use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, borsh0_10::try_from_slice_unchecked, program_error::ProgramError,
    program_pack::Sealed, pubkey::Pubkey,
};

use crate::{
    constants::{DECIMALS_SHARES, DECIMALS_USDC},
    error::ErrorCode,
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

    pub fn save(vault_depositor: &VaultDepositor, account: &AccountInfo) {
        let _ = vault_depositor.serialize(&mut &mut account.data.borrow_mut()[..]);
    }

    pub fn calculate_shares_for_deposit(
        total_vault_shares: u128,
        total_value_locked: u128,
        deposit_usdc: u128,
    ) -> Result<u128, ProgramError> {
        if total_vault_shares == 0 {
            // First deposit case: issue shares equivalent to the deposit amount in 18 decimals
            return Ok(deposit_usdc * DECIMALS_SHARES / DECIMALS_USDC);
        }
        // Calculate shares proportional to NAV (using scaled decimals)
        let scaled_deposit = deposit_usdc
            .checked_mul(Self::PRECISION_FACTOR)
            .ok_or(ErrorCode::Overflow)?;

        let proportion = scaled_deposit
            .checked_div(total_value_locked)
            .ok_or(ErrorCode::Overflow)?;

        let shares = proportion
            .checked_mul(total_vault_shares)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(Self::PRECISION_FACTOR)
            .ok_or(ErrorCode::Overflow)?;

        Ok(shares)
    }

    const PRECISION_FACTOR: u128 = 1_000_000; // 6 decimal places for precision
}
