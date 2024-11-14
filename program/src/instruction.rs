use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum VaultInstruction {
    InitializeVault {
        vault_id: String,
    },
    InitializeDrift {
        vault_id: String,
    },
    Deposit {
        vault_id: String,
        user_pubkey: String,
        amount: u64,
        fund_status: String,
        bot_status: String,
        market_index: u16,
    },
    Withdraw {
        vault_id: String,
        user_pubkey: String,
        amount: u64,
        fund_status: String,
        bot_status: String,
        market_index: u16,
    },
    UpdateDelegate {
        vault_id: String,
        delegate: String,
        sub_account: u16,
        fund_status: String,
        bot_status: String,
    },
}

#[derive(BorshDeserialize)]
struct VaultPayload {
    vault_id: String,
    user_pubkey: String,
    amount: u64,
    fund_status: String,
    bot_status: String,
    market_index: u16,
    delegate: String,
    sub_account: u16,
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
                market_index: payload.market_index,
            },
            2 => Self::Withdraw {
                vault_id: payload.vault_id,
                user_pubkey: payload.user_pubkey,
                amount: payload.amount,
                fund_status: payload.fund_status,
                bot_status: payload.bot_status,
                market_index: payload.market_index,
            },
            3 => Self::InitializeDrift {
                vault_id: payload.vault_id,
            },
            4 => Self::UpdateDelegate {
                vault_id: payload.vault_id,
                delegate: payload.delegate,
                sub_account: payload.sub_account,
                fund_status: payload.fund_status,
                bot_status: payload.bot_status,
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
