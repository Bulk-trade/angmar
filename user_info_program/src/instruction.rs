use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum VaultInstruction {
    AddUserInfo {
        user_pubkey: String,
        amount: u32,
        fund_status: String,
        bot_status: String,
    },
    UpdateUserInfo {
        user_pubkey: String,
        amount: u32,
        fund_status: String,
        bot_status: String,
    },
}

#[derive(BorshDeserialize)]
struct UserInfoPayload {
    user_pubkey: String,
    amount: u32,
    fund_status: String,
    bot_status: String,
}

impl VaultInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        let payload = UserInfoPayload::try_from_slice(rest).unwrap();
        Ok(match variant {
            0 => Self::AddUserInfo {
                user_pubkey: payload.user_pubkey,
                amount: payload.amount,
                fund_status: payload.fund_status,
                bot_status: payload.bot_status,
            },
            1 => Self::UpdateUserInfo {
                 user_pubkey: payload.user_pubkey,
                amount: payload.amount,
                fund_status: payload.fund_status,
                bot_status: payload.bot_status,
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
