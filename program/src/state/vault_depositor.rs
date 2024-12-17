use super::{Vault, WithdrawRequest};
use crate::{
    common::{log_data, log_params},
    error::{wrap_drift_error, ErrorCode},
    state::{VaultDepositorAction, VaultDepositorRecord},
    validate,
};
use borsh::{BorshDeserialize, BorshSerialize};
use drift::math::{insurance::vault_amount_to_if_shares, safe_math::SafeMath};
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
    /// creation ts of vault depositor
    pub last_valid_ts: u64,
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

    pub fn calculate_shares_for_deposit(
        amount: u64,
        total_vault_shares: u128,
        total_value_locked: u64,
    ) -> Result<u128, ProgramError> {
        let shares = vault_amount_to_if_shares(
            amount.try_into().unwrap(),
            total_vault_shares,
            total_value_locked.try_into().unwrap(),
        )
        .map_err(|e| ProgramError::Custom(e as u32))?;

        Ok(shares)
    }

    pub fn request_withdraw(
        &mut self,
        withdraw_amount: u64,
        vault_equity: u64,
        vault: &mut Vault,
        now: i64,
    ) -> ProgramResult {
        let shares = vault_amount_to_if_shares(withdraw_amount, vault.total_shares, vault_equity)
            .map_err(|e| ProgramError::Custom(e as u32))?;

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
            management_fee: 0,
            management_fee_shares: vault.management_fee,
        };

        log_data(&record)?;

        log_params(&record);

        Ok(())
    }
}
