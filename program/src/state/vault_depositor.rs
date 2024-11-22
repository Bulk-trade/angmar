use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::Sealed, pubkey::Pubkey};


#[derive(BorshSerialize, BorshDeserialize)]
pub struct VaultDepositor {
    /// The vault deposited into
    pub vault: Pubkey,
    /// The vault depositor account's pubkey. It is a pda of vault and authority
    pub pubkey: Pubkey,
    /// The authority is the address with permission to deposit/withdraw
    pub authority: Pubkey,
    /// share of vault owned by this depositor. vault_shares / vault.total_shares is depositor's ownership of vault_equity
    vault_shares: u128,
    /// creation ts of vault depositor
    pub last_valid_ts: i64,
    /// lifetime net deposits of vault depositor for the vault
    pub net_deposits: i64,
    /// lifetime total deposits
    pub total_deposits: u64,
    /// lifetime total withdraws
    pub total_withdraws: u64,
    /// the token amount of gains the vault depositor has paid performance fees on
    pub cumulative_profit_share_amount: i64,
    pub profit_share_fee_paid: u64,
    /// the exponent for vault_shares decimal places
    pub vault_shares_base: u32,
    pub padding1: u32,
    pub padding: [u64; 8],
}

impl Sealed for VaultDepositor {}