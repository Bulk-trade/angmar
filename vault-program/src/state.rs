// state.rs
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const VAULT_SEED: &[u8] = b"vault";

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VaultState {
    pub admin: Pubkey,
    pub total_deposits: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub owner: Pubkey,
    pub balance: u64,
}
