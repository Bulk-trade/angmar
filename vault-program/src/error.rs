use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfoError {
    #[error("Account not initialized yet")]
    UninitializedAccount,

    #[error("PDA derived does not equal PDA passed in")]
    InvalidPDA,

    #[error("Input data exceeds max length")]
    InvalidDataLength,

    #[error("Input data is invalid")]
    InvalidInput,
}

impl From<InfoError> for ProgramError {
    fn from(e: InfoError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
