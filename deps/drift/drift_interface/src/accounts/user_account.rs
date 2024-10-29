use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

impl UserAccount {
    pub fn deserialize(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        let user_account = Self::try_from_slice(buf)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(user_account)
    }
}
