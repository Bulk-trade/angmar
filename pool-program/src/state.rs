// src/state.rs

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const POOL_SEED: &[u8] = b"pool";

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolState {
    pub manager: Pubkey,
    pub vault: Pubkey,
    pub total_funds: u64,
    // Additional fields for strategy parameters, etc.
}
