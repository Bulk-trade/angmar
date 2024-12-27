use crate::instruction::VaultInstruction;
use crate::instructions::{
    cancel_withdraw_request, deposit, initialize_drift_vault_with_bulk, initialize_vault_depositor,
    request_withdraw, update_vault_delegate, withdraw, VaultParams,
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VaultInstruction::unpack(instruction_data)?;
    match instruction {
        VaultInstruction::InitializeDriftWithBulk {
            name,
            lock_in_period,
            redeem_period,
            max_tokens,
            management_fee,
            min_deposit_amount,
            profit_share,
            hurdle_rate,
            spot_market_index,
            permissioned,
        } => initialize_drift_vault_with_bulk(
            program_id,
            accounts,
            &VaultParams {
                name,
                lock_in_period,
                redeem_period,
                max_tokens,
                management_fee,
                min_deposit_amount,
                profit_share,
                hurdle_rate,
                spot_market_index,
                permissioned,
            },
        ),
        VaultInstruction::InitializeVaultDepositor {} => {
            initialize_vault_depositor(program_id, accounts)
        }
        VaultInstruction::Deposit { name, amount } => deposit(program_id, accounts, name, amount),
        VaultInstruction::WithdrawRequest { amount } => {
            request_withdraw(program_id, accounts, amount)
        }
        VaultInstruction::CancelWithdrawRequest {} => cancel_withdraw_request(program_id, accounts),
        VaultInstruction::Withdraw {} => withdraw(program_id, accounts),
        VaultInstruction::UpdateDelegate {
            name: vault_id,
            delegate,
            sub_account,
        } => update_vault_delegate(program_id, accounts, vault_id, delegate, sub_account),
    }
}
