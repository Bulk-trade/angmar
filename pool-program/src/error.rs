// src/error.rs

use thiserror::Error;
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum PoolError {
    #[error("Invalid Instruction")]
    InvalidInstruction,

    #[error("Invalid Pool Account")]
    InvalidPoolAccount,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Insufficient Funds")]
    InsufficientFunds,

    // Add more errors as needed
}

impl From<PoolError> for ProgramError {
    fn from(e: PoolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
