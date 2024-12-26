use std::mem;

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
    /// Record of Deposits
    pub deposits: Vec<DepositInfo>,
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
    pub const INITIAL_SIZE: usize = mem::size_of::<Self>() + mem::size_of::<DepositInfo>() * 10 + 8;

    pub fn get_vault_depositor_signer_seeds<'a>(
        vault: &'a [u8],
        authority: &'a [u8],
        bump: &'a [u8],
    ) -> [&'a [u8]; 4] {
        [b"vault_depositor", vault, authority, bump]
    }

    pub fn get_current_size(&self) -> usize {
        // Fixed struct size
        let base_size = mem::size_of::<Self>();

        // Dynamic size from deposits Vec
        let deposits_size = self.deposits.len() * DepositInfo::SIZE;

        base_size + deposits_size
    }

    pub fn does_need_resize(&self, account_size: usize) -> bool {
        account_size < self.get_current_size() + DepositInfo::SIZE //Current size plus one more DepositInfo struct
    }

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

    pub fn deposit(
        &mut self,
        mut amount: u64,
        vault_equity: u64,
        vault: &mut Vault,
        now: i64,
    ) -> Result<(u64, u64), ProgramError> {
        validate!(
            vault.max_tokens == 0
                || vault.max_tokens > vault_equity.safe_add(amount).map_err(wrap_drift_error)?,
            ErrorCode::VaultIsAtCapacity,
            "after deposit vault equity is {} > {}",
            vault_equity + amount,
            vault.max_tokens
        )?;

        validate!(
            vault.min_deposit_amount == 0 || amount >= vault.min_deposit_amount,
            ErrorCode::InvalidVaultDeposit,
            "deposit amount {} is below vault min_deposit_amount {}",
            amount,
            vault.min_deposit_amount
        )?;

        let vault_shares_before = self.vault_shares;
        let total_vault_shares_before = vault.total_shares;
        let user_vault_shares_before = vault.user_shares;

        let management_fee = vault.calculate_fees(amount);
        amount -= management_fee;
        msg!("Fees: {}", management_fee);
        msg!("Deposit amount after fees: {}", amount);

        let new_shares =
            Self::calculate_amount_to_shares(amount, total_vault_shares_before, vault_equity)?;
        msg!("Issuing user shares: {}", new_shares);

        self.total_deposits = self.total_deposits.saturating_add(amount);
        self.net_deposits = self.net_deposits.saturating_add(amount);
        self.vault_shares = self.vault_shares.saturating_add(new_shares);
        self.deposits.push(DepositInfo::new(now, new_shares));

        vault.manager_total_fee = vault.manager_total_fee.saturating_add(management_fee);
        vault.total_deposits = vault.total_deposits.saturating_add(amount);
        vault.net_deposits = vault.net_deposits.saturating_add(amount);
        vault.total_shares = vault
            .total_shares
            .safe_add(new_shares)
            .map_err(wrap_drift_error)?;
        vault.user_shares = vault
            .user_shares
            .safe_add(new_shares)
            .map_err(wrap_drift_error)?;

        msg!("Vault Deposit Record");
        let record = VaultDepositorRecord {
            ts: now,
            vault: vault.pubkey,
            depositor_authority: self.authority,
            action: VaultDepositorAction::Deposit,
            amount,
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
            management_fee,
            management_fee_amount: vault.management_fee,
        };

        log_data(&record)?;

        log_params(&record);

        Ok((amount, management_fee))
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

        validate!(
            vault_shares_before >= shares,
            ErrorCode::InsufficientVaultShares
        )?;

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

    pub fn withdraw(
        &mut self,
        vault_equity: u64,
        vault: &mut Vault,
        now: i64,
    ) -> Result<(u64, u64), ProgramError> {
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

        let mut withdraw_amount: u64 =
            if_shares_to_vault_amount(shares, vault.total_shares, vault_equity)
                .map_err(wrap_drift_error)?;
        // let mut withdraw_amount = amount.min(self.last_withdraw_request.value);

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
            .ok_or(ErrorCode::InsufficientWithdraw)?;

        msg!("Total deductions: {}", total_deductions);
        msg!("Final withdraw amount: {}", withdraw_amount);
        msg!(
            "Last withdraw request value: {}",
            self.last_withdraw_request.value
        );

        msg!(
            "amount={}, last_withdraw_request_value={}",
            withdraw_amount,
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

#[derive(BorshSerialize, BorshDeserialize)]
pub struct DepositInfo {
    /// Timestamp of Deposit
    pub ts: i64,
    /// Shares allocated at Deposit
    pub shares: u128,
}

impl DepositInfo {
    pub const SIZE: usize = mem::size_of::<DepositInfo>();

    pub fn new(ts: i64, shares: u128) -> Self {
        Self { ts, shares }
    }
}
