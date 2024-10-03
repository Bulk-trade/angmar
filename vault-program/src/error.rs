// error.rs
use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum VaultError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Invalid Vault Account")]
    InvalidVaultAccount,
    #[error("Invalid User Account")]
    InvalidUserAccount,
    #[error("Insufficient Funds")]
    InsufficientFunds,
    #[error("Signature Missing")]
    SignatureMissing,
    #[error("Unauthorized")]
    Unauthorized,
}

impl From<VaultError> for ProgramError {
    fn from(e: VaultError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
