// instruction.rs
use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum VaultInstruction {
    /// Initializes the vault
    /// Accounts:
    /// - [signer] Admin
    /// - [writable] Vault Account (PDA)
    InitializeVault,

    /// Deposits funds into the vault
    /// Accounts:
    /// - [signer] User
    /// - [writable] User Account (PDA)
    /// - [writable] Vault Account (PDA)
    /// - [] System Program
    Deposit { amount: u64 },

    /// Withdraws funds from the vault
    /// Accounts:
    /// - [signer] User
    /// - [writable] User Account (PDA)
    /// - [writable] Vault Account (PDA)
    /// - [] System Program
    Withdraw { amount: u64 },
}
