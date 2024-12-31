use std::mem;

use super::{Vault, WithdrawRequest};
use crate::{
    common::{calculate_amount_to_shares, log_data, log_params},
    constants::PERCENTAGE_PRECISION,
    custom_validate,
    error::{wrap_drift_error, VaultErrorCode},
    state::{VaultDepositorAction, VaultDepositorRecord},
};
use borsh::{BorshDeserialize, BorshSerialize};
use drift::math::{casting::Cast, insurance::if_shares_to_vault_amount, safe_math::SafeMath};
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
        custom_validate!(
            vault.max_tokens == 0 || vault.max_tokens > vault_equity.saturating_add(amount),
            VaultErrorCode::VaultIsAtCapacity,
            "after deposit vault equity is {} > {}",
            vault_equity + amount,
            vault.max_tokens
        )?;

        custom_validate!(
            vault.min_deposit_amount == 0 || amount >= vault.min_deposit_amount,
            VaultErrorCode::InvalidVaultDeposit,
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
            calculate_amount_to_shares(amount, total_vault_shares_before, vault_equity)?;
        msg!("Issuing user shares: {}", new_shares);

        self.total_deposits = self.total_deposits.saturating_add(amount);
        self.net_deposits = self.net_deposits.saturating_add(amount);
        self.vault_shares = self.vault_shares.saturating_add(new_shares);
        self.deposits.push(DepositInfo::new(now, new_shares));

        vault.manager_total_fee = vault.manager_total_fee.saturating_add(management_fee);
        vault.manager_total_net_fee = vault.manager_total_net_fee.saturating_add(management_fee);
        vault.total_deposits = vault.total_deposits.saturating_add(amount);
        vault.net_deposits = vault.net_deposits.saturating_add(amount);
        vault.total_shares = vault.total_shares.saturating_add(new_shares);
        vault.user_shares = vault.user_shares.saturating_add(new_shares);

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
            management_fee: vault.management_fee,
            management_fee_amount: management_fee,
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
        let shares = calculate_amount_to_shares(withdraw_amount, vault.total_shares, vault_equity)?;

        custom_validate!(
            shares > 0,
            VaultErrorCode::InvalidVaultWithdrawSize,
            "Requested shares = 0"
        )?;

        let withdrawable_shares = self.calculate_withdrawable_shares(now, vault.lock_in_period)?;

        custom_validate!(
            shares <= withdrawable_shares,
            VaultErrorCode::InvalidVaultWithdrawSize,
            "Requested shares greater then withdrawable_shares"
        )?;

        let vault_shares_before: u128 = self.vault_shares;
        let total_vault_shares_before = vault.total_shares;
        let user_vault_shares_before = vault.user_shares;

        custom_validate!(
            vault_shares_before >= shares,
            VaultErrorCode::InsufficientVaultShares
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
            .saturating_add(withdraw_amount);

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
            .saturating_sub(self.last_withdraw_request.value);

        self.last_withdraw_request.reset(now)?;

        msg!("Vault Cancel Withdraw Request Record");
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

        custom_validate!(
            shares > 0,
            VaultErrorCode::InvalidVaultWithdraw,
            "Must submit withdraw request and wait the redeem_period ({} seconds)",
            vault.redeem_period
        )?;

        custom_validate!(
            vault_shares_before >= shares,
            VaultErrorCode::InsufficientVaultShares
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
        let total_deductions = management_fee.saturating_add(profit_share);

        withdraw_amount = withdraw_amount.saturating_sub(total_deductions);

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

        self.profit_share_fee_paid = self.profit_share_fee_paid.saturating_add(profit_share);

        self.vault_shares = self.vault_shares.saturating_sub(shares);

        self.total_withdraws = self.total_withdraws.saturating_add(withdraw_amount);
        self.net_deposits = self.net_deposits.saturating_sub(withdraw_amount);

        self.remove_shares(shares)?;

        vault.manager_total_fee = vault.manager_total_fee.saturating_add(management_fee);
        vault.manager_total_profit_share = vault
            .manager_total_profit_share
            .saturating_add(profit_share);
        vault.manager_total_net_fee = vault.manager_total_net_fee.saturating_add(total_deductions);

        vault.total_withdraws = vault.total_withdraws.saturating_add(withdraw_amount);
        vault.net_deposits = vault.net_deposits.saturating_sub(withdraw_amount);
        vault.total_shares = vault.total_shares.saturating_sub(shares);
        vault.user_shares = vault.user_shares.saturating_sub(shares);
        vault.total_withdraw_requested = vault
            .total_withdraw_requested
            .saturating_sub(self.last_withdraw_request.value);

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
        let total_profit = total_amount.saturating_sub(self.net_deposits);

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

    /// Calculates withdrawable shares based on lock-in period
    ///
    /// # Arguments
    /// * `now` - Current Unix timestamp
    /// * `lock_in_period` - Period in seconds that deposits must be locked
    ///
    /// # Returns
    /// * `Result<u128, ProgramError>` - Total withdrawable shares or error
    pub fn calculate_withdrawable_shares(
        &self,
        current_timestamp: i64,
        lock_in_period: u64,
    ) -> Result<u128, ProgramError> {
        msg!(
            "Calculating withdrawable shares at timestamp {}",
            current_timestamp
        );

        // Sum shares from deposits that have passed their lock-in period
        let withdrawable_shares = self
            .deposits
            .iter()
            .filter_map(|deposit| {
                // Calculate unlock time for this deposit
                let unlock_time = deposit
                    .ts
                    .checked_add(lock_in_period as i64)
                    .ok_or(VaultErrorCode::MathError)
                    .ok()?;

                // Include shares if unlock time has passed
                if current_timestamp >= unlock_time {
                    msg!(
                        "Deposit of {} shares unlocked at {} (deposited at {})",
                        deposit.shares,
                        unlock_time,
                        deposit.ts
                    );
                    Some(deposit.shares)
                } else {
                    msg!(
                        "Deposit of {} shares still locked until {}",
                        deposit.shares,
                        unlock_time
                    );
                    None
                }
            })
            .sum();

        msg!(
            "Total withdrawable shares: {} at timestamp {}",
            withdrawable_shares,
            current_timestamp
        );

        Ok(withdrawable_shares)
    }

    pub fn remove_shares(&mut self, shares_to_remove: u128) -> Result<(), ProgramError> {
        // Calculate and log total shares before removal
        let total_shares_before: u128 = self.deposits.iter().map(|d| d.shares).sum();

        msg!(
            "Removing {} shares from total {} shares",
            shares_to_remove,
            total_shares_before
        );

        let mut remaining_shares = shares_to_remove;

        // Sort deposits by timestamp (oldest first)
        self.deposits.sort_by_key(|d| d.ts);

        // Track indices to remove
        let mut indices_to_remove = Vec::new();

        // Process deposits from oldest to newest
        for (i, deposit) in self.deposits.iter_mut().enumerate() {
            if remaining_shares == 0 {
                break;
            }

            if deposit.shares <= remaining_shares {
                // Remove entire deposit
                remaining_shares = remaining_shares
                    .checked_sub(deposit.shares)
                    .ok_or(VaultErrorCode::MathError)?;
                indices_to_remove.push(i);
            } else {
                // Partial removal
                deposit.shares = deposit
                    .shares
                    .checked_sub(remaining_shares)
                    .ok_or(VaultErrorCode::MathError)?;
                remaining_shares = 0;
            }
        }

        // Remove fully withdrawn deposits
        for &index in indices_to_remove.iter().rev() {
            self.deposits.remove(index);
        }

        // Validate all shares were removed
        if remaining_shares > 0 {
            msg!("Not enough shares in deposits to remove");
            return Err(VaultErrorCode::InsufficientShares.into());
        }

        // Calculate and log remaining shares after removal
        let total_shares_after: u128 = self.deposits.iter().map(|d| d.shares).sum();

        msg!(
            "After removal: {} shares remaining (removed {})",
            total_shares_after,
            total_shares_before - total_shares_after
        );

        Ok(())
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
