// src/instruction.rs

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum PoolInstruction {
    /// Initializes a new pool
    /// Accounts:
    /// - [signer] Manager
    /// - [writable] Pool Account (PDA)
    /// - [] Vault Program ID
    InitializePool {
        vault: Pubkey,
    },

    /// Allocates funds to the pool
    /// Accounts:
    /// - [signer] Vault Program
    /// - [writable] Pool Account (PDA)
    /// - [writable] Pool Vault Account (PDA)
    AllocateFunds {
        amount: u64,
    },

    /// Placeholder for additional instructions
}
