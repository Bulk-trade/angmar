use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

use crate::{
    common::{deserialize_zero_copy, log_accounts, transfer_to_user},
    error::wrap_drift_error,
    instructions::drift_withdraw,
    state::Vault,
};

pub fn manager_collect_fees<'info>(
    _program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    amount: u64,
) -> ProgramResult {
    msg!("Collecting fees...");
    msg!("amount: {}", amount);

    let clock = &Clock::get()?;

    let mut account_info_iter = &mut accounts.iter();

    Ok(())
}
