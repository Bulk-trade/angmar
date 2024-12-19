use super::{Vault, WithdrawRequest};
use crate::{
    common::{log_data, log_params},
    constants::PERCENTAGE_PRECISION,
    error::{wrap_drift_error, ErrorCode},
    state::{VaultDepositorAction, VaultDepositorRecord},
    validate,
};
use borsh::{BorshDeserialize, BorshSerialize};
use drift::math::{
    casting::Cast,
    insurance::{if_shares_to_vault_amount, vault_amount_to_if_shares},
    safe_math::SafeMath,
};
use solana_program::{
    account_info::AccountInfo, borsh0_10::try_from_slice_unchecked, entrypoint::ProgramResult, msg,
    program_error::ProgramError, program_pack::Sealed, pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultDepositor {
    /// The vault deposited into
    pub vault: Pubkey,
    /// The vault depositor account's pubkey. It is a pda of vault and authority
    pub pubkey: Pubkey,
    /// The authority is the address with permission to deposit/withdraw
    pub authority: Pubkey,
    /// share of vault owned by this depositor. vault_shares / vault.total_shares is depositor's ownership of vault_equity
    pub vault_shares: u128,
    /// last withdraw request
    pub last_withdraw_request: WithdrawRequest,
    /// Timestamp vault depositor initialized
    pub init_ts: u64,
    /// lifetime net deposits of vault depositor for the vault
    pub net_deposits: u64,
    /// lifetime total deposits
    pub total_deposits: u64,
    /// lifetime total withdraws
    pub total_withdraws: u64,
    /// the token amount of gains the vault depositor has paid performance fees on
    pub cumulative_profit_share_amount: u64,
    pub profit_share_fee_paid: u64,
    /// the exponent for vault_shares decimal places
    pub vault_shares_base: u32,
    pub padding1: u32,
    pub padding: [u64; 8],
}

impl Sealed for VaultDepositor {}

impl VaultDepositor {
    pub fn get_pda<'a>(vault: &Pubkey, authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"vault_depositor", vault.as_ref(), authority.as_ref()],
            program_id,
        )
    }

    pub fn get(account: &AccountInfo) -> Self {
        try_from_slice_unchecked::<VaultDepositor>(&account.data.borrow()).unwrap()
    }

    pub fn save(&self, account: &AccountInfo) -> ProgramResult {
        self.serialize(&mut &mut account.data.borrow_mut()[..])?;
        Ok(())
    }

    pub fn calculate_amount_to_shares(
        amount: u64,
        total_vault_shares: u128,
        total_value_locked: u64,
    ) -> Result<u128, ProgramError> {
        let shares = vault_amount_to_if_shares(amount, total_vault_shares, total_value_locked)
            .map_err(wrap_drift_error)?;

        Ok(shares)
    }

    pub fn calculate_shares_to_amount(
        n_shares: u128,
        total_vault_shares: u128,
        total_value_locked: u64,
    ) -> Result<u64, ProgramError> {
        let amount = if_shares_to_vault_amount(n_shares, total_vault_shares, total_value_locked)
            .map_err(wrap_drift_error)?;

        Ok(amount)
    }

    pub fn request_withdraw(
        &mut self,
        withdraw_amount: u64,
        vault_equity: u64,
        vault: &mut Vault,
        now: i64,
    ) -> ProgramResult {
        let shares = vault_amount_to_if_shares(withdraw_amount, vault.total_shares, vault_equity)
            .map_err(wrap_drift_error)?;

        validate!(
            shares > 0,
            ErrorCode::InvalidVaultWithdrawSize,
            "Requested shares = 0"
        )?;

        let vault_shares_before: u128 = self.vault_shares;
        let total_vault_shares_before = vault.total_shares;
        let user_vault_shares_before = vault.user_shares;

        self.last_withdraw_request.set(
            vault_shares_before,
            shares,
            withdraw_amount,
            vault_equity,
            now,
        )?;

        vault.total_withdraw_requested = vault
            .total_withdraw_requested
            .safe_add(withdraw_amount)
            .map_err(wrap_drift_error)?;

        msg!("Vault Withdraw Request Record");
        let record = VaultDepositorRecord {
            ts: now,
            vault: vault.pubkey,
            depositor_authority: self.authority,
            action: VaultDepositorAction::WithdrawRequest,
            amount: withdraw_amount,
            spot_market_index: vault.spot_market_index,
            vault_equity_before: vault_equity,
            vault_shares_before,
            user_vault_shares_before,
            total_vault_shares_before,
            vault_shares_after: self.vault_shares,
            total_vault_shares_after: vault.total_shares,
            user_vault_shares_after: vault.user_shares,
            profit_share: vault.profit_share,
            profit_share_amount: 0,
            management_fee: 0,
            management_fee_amount: vault.management_fee,
        };

        log_data(&record)?;

        log_params(&record);

        Ok(())
    }

    pub fn cancel_withdraw_request(
        &mut self,
        vault_equity: u64,
        vault: &mut Vault,
        now: i64,
    ) -> ProgramResult {
        let vault_shares_before: u128 = self.vault_shares;
        let total_vault_shares_before = vault.total_shares;
        let user_vault_shares_before = vault.user_shares;

        vault.total_withdraw_requested = vault
            .total_withdraw_requested
            .safe_sub(self.last_withdraw_request.value)
            .map_err(wrap_drift_error)?;

        self.last_withdraw_request.reset(now)?;

        msg!("Vault Withdraw Request Record");
        let record = VaultDepositorRecord {
            ts: now,
            vault: vault.pubkey,
            depositor_authority: self.authority,
            action: VaultDepositorAction::CancelWithdrawRequest,
            amount: 0,
            spot_market_index: vault.spot_market_index,
            vault_equity_before: vault_equity,
            vault_shares_before,
            user_vault_shares_before,
            total_vault_shares_before,
            vault_shares_after: self.vault_shares,
            total_vault_shares_after: vault.total_shares,
            user_vault_shares_after: vault.user_shares,
            profit_share: vault.profit_share,
            profit_share_amount: 0,
            management_fee: 0,
            management_fee_amount: vault.management_fee,
        };

        log_data(&record)?;

        log_params(&record);

        Ok(())
    }

    pub fn withdraw(&mut self, vault_equity: u64, vault: &mut Vault, now: i64) -> Result<(u64, u64), ProgramError> {
        self.last_withdraw_request
            .check_redeem_period_finished(vault, now)?;

        let vault_shares_before: u128 = self.vault_shares;
        let total_vault_shares_before = vault.total_shares;
        let user_vault_shares_before = vault.user_shares;

        let shares = self.last_withdraw_request.shares;

        validate!(
            shares > 0,
            ErrorCode::InvalidVaultWithdraw,
            "Must submit withdraw request and wait the redeem_period ({} seconds)",
            vault.redeem_period
        )?;

        validate!(
            vault_shares_before >= shares,
            ErrorCode::InsufficientVaultShares
        )?;

        let amount: u64 = if_shares_to_vault_amount(shares, vault.total_shares, vault_equity)
            .map_err(wrap_drift_error)?;
        let mut withdraw_amount = amount.min(self.last_withdraw_request.value);

        // Calculate fees and profit share
        let management_fee = vault.calculate_fees(withdraw_amount);
        let profit_share = self.calculate_profit_share(withdraw_amount, vault_equity, vault)?;

        msg!("Management fee: {}", management_fee);
        msg!("Profit share: {}", profit_share);

        // Calculate total deductions and final amount
        let total_deductions = management_fee
            .checked_add(profit_share)
            .ok_or(ErrorCode::MathError)?;

        withdraw_amount = withdraw_amount
            .checked_sub(total_deductions)
            .ok_or(ErrorCode::InsufficientWithdraw)?
            .min(self.last_withdraw_request.value);

        msg!("Total deductions: {}", total_deductions);
        msg!("Final withdraw amount: {}", withdraw_amount);
        msg!(
            "Last withdraw request value: {}",
            self.last_withdraw_request.value
        );

        msg!(
            "amount={}, last_withdraw_request_value={}",
            amount,
            self.last_withdraw_request.value
        );
        msg!(
            "vault_shares={}, last_withdraw_request_shares={}",
            self.vault_shares,
            self.last_withdraw_request.shares
        );

        self.profit_share_fee_paid = self
            .profit_share_fee_paid
            .safe_add(profit_share.cast().map_err(wrap_drift_error)?)
            .map_err(wrap_drift_error)?;

        self.vault_shares = self
            .vault_shares
            .safe_sub(shares)
            .map_err(wrap_drift_error)?;

        self.total_withdraws = self.total_withdraws.saturating_add(withdraw_amount);
        self.net_deposits = self
            .net_deposits
            .safe_sub(withdraw_amount.cast().map_err(wrap_drift_error)?)
            .map_err(wrap_drift_error)?;

        vault.manager_total_fee = vault
            .manager_total_fee
            .checked_add(management_fee)
            .ok_or(ErrorCode::MathError)?;

        vault.manager_total_profit_share = vault
            .manager_total_profit_share
            .checked_add(profit_share)
            .ok_or(ErrorCode::MathError)?;

        vault.total_withdraws = vault.total_withdraws.saturating_add(withdraw_amount);
        vault.net_deposits = vault
            .net_deposits
            .safe_sub(withdraw_amount.cast().map_err(wrap_drift_error)?)
            .map_err(wrap_drift_error)?;
        vault.total_shares = vault
            .total_shares
            .safe_sub(shares)
            .map_err(wrap_drift_error)?;
        vault.user_shares = vault
            .user_shares
            .safe_sub(shares)
            .map_err(wrap_drift_error)?;
        vault.total_withdraw_requested = vault
            .total_withdraw_requested
            .safe_sub(self.last_withdraw_request.value)
            .map_err(wrap_drift_error)?;

        self.last_withdraw_request.reset(now)?;

        let vault_shares_after = self.vault_shares;

        msg!("Vault Withdraw Record");
        let record = VaultDepositorRecord {
            ts: now,
            vault: vault.pubkey,
            depositor_authority: self.authority,
            action: VaultDepositorAction::Withdraw,
            amount: withdraw_amount,
            spot_market_index: vault.spot_market_index,
            vault_equity_before: vault_equity,
            vault_shares_before,
            user_vault_shares_before,
            total_vault_shares_before,
            vault_shares_after,
            total_vault_shares_after: vault.total_shares,
            user_vault_shares_after: vault.user_shares,
            profit_share: vault.profit_share,
            profit_share_amount: profit_share,
            management_fee: vault.management_fee,
            management_fee_amount: management_fee,
        };

        log_data(&record)?;

        log_params(&record);

        Ok((withdraw_amount, total_deductions))
    }

    pub fn calculate_profit_share(
        &self,
        amount: u64,
        vault_equity: u64,
        vault: &mut Vault,
    ) -> Result<u64, ProgramError> {
        // Calculate total value of shares
        let total_amount =
            if_shares_to_vault_amount(self.vault_shares, vault.total_shares, vault_equity)
                .map_err(wrap_drift_error)?;

        // Calculate total profit/loss
        let total_profit = total_amount
            .cast::<i64>()
            .map_err(wrap_drift_error)?
            .safe_sub(self.net_deposits.cast::<i64>().map_err(wrap_drift_error)?)
            .map_err(wrap_drift_error)?;

        // Only take profit share if profitable
        if total_profit > 0 {
            let profit_share_amount = total_profit
                .cast::<u128>()
                .map_err(wrap_drift_error)?
                .safe_mul(vault.profit_share.into())
                .map_err(wrap_drift_error)?
                .safe_div(PERCENTAGE_PRECISION.into())
                .map_err(wrap_drift_error)?
                .cast::<u64>()
                .map_err(wrap_drift_error)?;

            // Scale profit share by withdrawal amount
            let scaled_profit_share = profit_share_amount
                .safe_mul(amount)
                .map_err(wrap_drift_error)?
                .safe_div(total_amount)
                .map_err(wrap_drift_error)?;

            Ok(scaled_profit_share)
        } else {
            Ok(0)
        }
    }
}
