use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum VaultInstruction {
    InitializeVault {
        vault_id: String,
    },
    Deposit {
        vault_id: String,
        user_pubkey: String,
        amount: u64,
        fund_status: String,
        bot_status: String,
    },
    Withdraw {
        vault_id: String,
        user_pubkey: String,
        amount: u64,
        fund_status: String,
        bot_status: String,
    },
    InitializeDrift {
        vault_id: String,
    },
}

#[derive(BorshDeserialize)]
struct VaultPayload {
    vault_id: String,
    user_pubkey: String,
    amount: u64,
    fund_status: String,
    bot_status: String,
}

impl VaultInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        let payload = VaultPayload::try_from_slice(rest).unwrap();
        Ok(match variant {
            0 => Self::InitializeVault {
                vault_id: payload.vault_id,
            },
            1 => Self::Deposit {
                vault_id: payload.vault_id,
                user_pubkey: payload.user_pubkey,
                amount: payload.amount,
                fund_status: payload.fund_status,
                bot_status: payload.bot_status,
            },
            2 => Self::Withdraw {
                vault_id: payload.vault_id,
                user_pubkey: payload.user_pubkey,
                amount: payload.amount,
                fund_status: payload.fund_status,
                bot_status: payload.bot_status,
            },
            3 => Self::InitializeDrift {
                vault_id: payload.vault_id,
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
