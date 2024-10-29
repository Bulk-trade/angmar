use borsh::BorshDeserialize;
use solana_program::{msg, program_error::ProgramError};

pub enum VaultInstruction {
    InitializeVault {
        vault_id: String,
    },
    Deposit {
        vault_id: String,
        user_pubkey: String,
        amount: f32,
        fund_status: String,
        bot_status: String,
    },
    Withdraw {
        vault_id: String,
        user_pubkey: String,
        amount: f32,
        fund_status: String,
        bot_status: String,
    },
    InitializeDrift {
        vault_id: String,
    },
}

impl VaultInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let mut data = input;

        // Read the variant
        let variant = u8::deserialize(&mut data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match variant {
            0 => {
                // InitializeVault expects vault_id
                let vault_id = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(VaultInstruction::InitializeVault { vault_id })
            }
            1 => {
                // Deposit expects vault_id, user_pubkey, amount, fund_status, bot_status
                let vault_id = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let user_pubkey = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let amount = f32::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let fund_status = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let bot_status = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(VaultInstruction::Deposit {
                    vault_id,
                    user_pubkey,
                    amount,
                    fund_status,
                    bot_status,
                })
            }
            2 => {
                // Withdraw expects vault_id, user_pubkey, amount, fund_status, bot_status
                let vault_id = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let user_pubkey = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let amount = f32::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let fund_status = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let bot_status = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(VaultInstruction::Withdraw {
                    vault_id,
                    user_pubkey,
                    amount,
                    fund_status,
                    bot_status,
                })
            }
            3 => {
                // InitializeDrift expects vault_id
                let vault_id = String::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(VaultInstruction::InitializeDrift { vault_id })
            }
            _ => {
                msg!("Invalid instruction variant: {}", variant);
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}
