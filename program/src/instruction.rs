use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum VaultInstruction {
    InitializeVault {
        vault_id: String,
    },
    InitializeDrift {
        vault_id: String,
    },
    InitializeDriftWithBulk {
        name: String,
        lock_in_period: u64,
        redeem_period: u64,
        max_tokens: u64,
        management_fee: u64,
        min_deposit_amount: u64,
        profit_share: u32,
        hurdle_rate: u32,
        spot_market_index: u16,
        permissioned: bool,
    },
    InitializeVaultDepositor {},
    Deposit {
        name: String,
        amount: u64,
    },
    DepositOld {
        vault_id: String,
        user_pubkey: String,
        amount: u64,
        fund_status: String,
        bot_status: String,
        market_index: u16,
    },
    WithdrawRequest {
        amount: u64,
    },
    CancelWithdrawRequest {},
    Withdraw {},
    UpdateDelegate {
        name: String,
        delegate: String,
        sub_account: u16,
    },
}

// #[derive(BorshDeserialize)]
// struct BaseVaultPayload {
//     name: String,
//     user_pubkey: String,
//     amount: u64,
//     fund_status: String,
//     bot_status: String,
//     market_index: u16,
// }

#[derive(BorshDeserialize)]
struct InitVaultPayload {
    name: String,
    lock_in_period: u64,
    redeem_period: u64,
    max_tokens: u64,
    management_fee: u64,
    min_deposit_amount: u64,
    profit_share: u32,
    hurdle_rate: u32,
    spot_market_index: u16,
    permissioned: bool,
}

#[derive(BorshDeserialize)]
struct DepositPayload {
    name: String,
    amount: u64,
}

#[derive(BorshDeserialize)]
struct UpdateDelegatePayload {
    name: String,
    delegate: String,
    sub_account: u16,
}

#[derive(BorshDeserialize)]
struct WithdrawRequestPayload {
    amount: u64,
}

impl VaultInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match variant {
            0 => {
                let payload = InitVaultPayload::try_from_slice(rest).unwrap();
                Self::InitializeDriftWithBulk {
                    name: payload.name,
                    lock_in_period: payload.lock_in_period,
                    redeem_period: payload.redeem_period,
                    max_tokens: payload.max_tokens,
                    management_fee: payload.management_fee,
                    min_deposit_amount: payload.min_deposit_amount,
                    profit_share: payload.profit_share,
                    hurdle_rate: payload.hurdle_rate,
                    spot_market_index: payload.spot_market_index,
                    permissioned: payload.permissioned,
                }
            }
            1 => Self::InitializeVaultDepositor {},
            2 => {
                let payload = UpdateDelegatePayload::try_from_slice(rest).unwrap();
                Self::UpdateDelegate {
                    name: payload.name,
                    delegate: payload.delegate,
                    sub_account: payload.sub_account,
                }
            }
            3 => {
                let payload = DepositPayload::try_from_slice(rest).unwrap();
                Self::Deposit {
                    name: payload.name,
                    amount: payload.amount,
                }
            }
            4 => {
                let payload = WithdrawRequestPayload::try_from_slice(rest).unwrap();
                Self::WithdrawRequest {
                    amount: payload.amount,
                }
            }
            5 => Self::CancelWithdrawRequest {},
            6 => Self::Withdraw {},
            // 2 => {
            //     let payload = BaseVaultPayload::try_from_slice(rest).unwrap();
            //     Self::Withdraw {
            //         vault_id: payload.vault_id,
            //         user_pubkey: payload.user_pubkey,
            //         amount: payload.amount,
            //         fund_status: payload.fund_status,
            //         bot_status: payload.bot_status,
            //         market_index: payload.market_index,
            //     }
            // }
            // 3 => {
            //     let payload = BaseVaultPayload::try_from_slice(rest).unwrap();
            //     Self::InitializeDrift {
            //         vault_id: payload.vault_id,
            //     }
            // }

            // 7 => {
            //     let payload = BaseVaultPayload::try_from_slice(rest).unwrap();
            //     Self::DepositOld {
            //         vault_id: payload.vault_id,
            //         user_pubkey: payload.user_pubkey,
            //         amount: payload.amount,
            //         fund_status: payload.fund_status,
            //         bot_status: payload.bot_status,
            //         market_index: payload.market_index,
            //     }
            // }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
