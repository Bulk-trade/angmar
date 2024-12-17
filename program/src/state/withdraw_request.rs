use crate::{error::ErrorCode, validate};
use borsh::{BorshDeserialize, BorshSerialize};
use drift::math::safe_math::SafeMath;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::sysvar::slot_history::ProgramError;

use super::Vault;

#[derive(BorshSerialize, BorshDeserialize)]
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
        validate!(
            self.value == 0,
            ErrorCode::VaultWithdrawRequestInProgress,
            "withdraw request is already in progress"
        )?;

        validate!(
            withdraw_shares <= current_shares,
            ErrorCode::InvalidVaultWithdrawSize,
            "shares requested exceeds vault_shares {} > {}",
            withdraw_shares,
            current_shares
        )?;

        self.shares = withdraw_shares;

        validate!(
            withdraw_amount == 0 || withdraw_amount <= vault_equity,
            ErrorCode::InvalidVaultWithdrawSize,
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
        let time_since_withdraw_request = now
            .safe_sub(self.ts)
            .map_err(|e| ProgramError::Custom(e as u32))?;

        validate!(
            time_since_withdraw_request >= vault.redeem_period as i64,
            ErrorCode::CannotWithdrawBeforeRedeemPeriodEnd
        )?;

        Ok(())
    }
}
