use crate::{custom_validate, error::VaultErrorCode};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::sysvar::slot_history::ProgramError;

use super::Vault;

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct WithdrawRequest {
    /// request shares of vault withdraw
    pub shares: u128,
    /// requested value (in vault spot_market_index) of shares for withdraw
    pub value: u64,
    /// request ts of vault withdraw
    pub ts: i64,
}

impl WithdrawRequest {
    pub fn set(
        &mut self,
        current_shares: u128,
        withdraw_shares: u128,
        withdraw_amount: u64,
        vault_equity: u64,
        now: i64,
    ) -> ProgramResult {
        custom_validate!(
            self.value == 0,
            VaultErrorCode::VaultWithdrawRequestInProgress,
            "withdraw request is already in progress"
        )?;

        custom_validate!(
            withdraw_shares <= current_shares,
            VaultErrorCode::InvalidVaultWithdrawSize,
            "shares requested exceeds vault_shares {} > {}",
            withdraw_shares,
            current_shares
        )?;

        self.shares = withdraw_shares;

        custom_validate!(
            withdraw_amount == 0 || withdraw_amount <= vault_equity,
            VaultErrorCode::InvalidVaultWithdrawSize,
            "Requested withdraw value {} is not equal or below vault_equity {}",
            withdraw_amount,
            vault_equity
        )?;

        self.value = withdraw_amount;

        self.ts = now;

        Ok(())
    }

    pub fn reset(&mut self, now: i64) -> ProgramResult {
        // reset vault_depositor withdraw request info
        self.shares = 0;
        self.value = 0;
        self.ts = now;

        Ok(())
    }

    pub fn check_redeem_period_finished(&self, vault: &Vault, now: i64) -> ProgramResult {
        let time_since_withdraw_request = now.saturating_sub(self.ts);

        custom_validate!(
            time_since_withdraw_request >= vault.redeem_period as i64,
            VaultErrorCode::CannotWithdrawBeforeRedeemPeriodEnd
        )?;

        Ok(())
    }
}
