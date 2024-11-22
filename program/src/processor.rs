use crate::instruction::VaultInstruction;
use crate::instructions::{
    deposit, initialize_drift, initialize_drift_vault_with_bulk, initialize_vault, update_delegate,
    withdraw,
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VaultInstruction::unpack(instruction_data)?;
    match instruction {
        VaultInstruction::InitializeDriftWithBulk {
            name,
            management_fee,
            min_deposit_amount,
            profit_share,
            spot_market_index,
            permissioned,
        } => {
            initialize_drift_vault_with_bulk(
                program_id,
                accounts,
                name,
                management_fee,
                min_deposit_amount,
                profit_share,
                spot_market_index,
                permissioned,
            )
        }
        VaultInstruction::InitializeVault { vault_id } => {
            initialize_vault(program_id, accounts, vault_id)
        }
        VaultInstruction::Deposit {
            vault_id,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
            market_index,
        } => deposit(
            program_id,
            accounts,
            vault_id,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
            market_index,
        ),
        VaultInstruction::Withdraw {
            vault_id,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
            market_index,
        } => withdraw(
            program_id,
            accounts,
            vault_id,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
            market_index,
        ),
        VaultInstruction::InitializeDrift { vault_id } => {
            initialize_drift(program_id, accounts, vault_id)
        }
        VaultInstruction::UpdateDelegate {
            vault_id,
            delegate,
            sub_account,
            fund_status,
            bot_status,
        } => update_delegate(
            program_id,
            accounts,
            vault_id,
            delegate,
            sub_account,
            fund_status,
            bot_status,
        ),
    }
}
