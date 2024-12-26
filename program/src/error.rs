use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorCode {
    #[error("Account not initialized yet")]
    UninitializedAccount,

    #[error("PDA derived does not equal PDA passed in")]
    InvalidPDA,

    #[error("Input data exceeds max length")]
    InvalidDataLength,

    #[error("Input data is invalid")]
    InvalidInput,

    #[error("Input data for initialization is invalid")]
    InvalidVaultInitialization,

    #[error("PermissionedVault")]
    PermissionedVault,

    #[error("VaultIsAtCapacity")]
    VaultIsAtCapacity,

    #[error("Overflow")]
    Overflow,

    #[error("InvalidVaultDeposit")]
    InvalidVaultDeposit,

    #[error("VaultWithdrawRequestInProgress")]
    VaultWithdrawRequestInProgress,

    #[error("InvalidVaultWithdrawSize")]
    InvalidVaultWithdrawSize,

    #[error("CannotWithdrawBeforeRedeemPeriodEnd")]
    CannotWithdrawBeforeRedeemPeriodEnd,

    #[error("InvalidEquityValue")]
    InvalidEquityValue,

    #[error("InsufficientVaultShares")]
    InsufficientVaultShares,

    #[error("InvalidVaultWithdraw")]
    InvalidVaultWithdraw,

    #[error("MathError")]
    MathError,

    #[error("InsufficientWithdraw")]
    InsufficientWithdraw,

    #[error("InsufficientShares")]
    InsufficientShares,
}

impl From<ErrorCode> for ProgramError {
    fn from(e: ErrorCode) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// Create wrapper type in our crate
#[derive(Debug)]
pub struct DriftErrorWrapper(drift::error::ErrorCode);

// Implement From for our wrapper
impl From<DriftErrorWrapper> for ProgramError {
    fn from(e: DriftErrorWrapper) -> Self {
        ProgramError::Custom(e.0 as u32)
    }
}

// Helper function to wrap drift errors
pub fn wrap_drift_error(e: drift::error::ErrorCode) -> ProgramError {
    DriftErrorWrapper(e).into()
}
