use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

pub fn manager_deposit<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    name: String,
    amount: u64,
) -> ProgramResult {
    msg!("Starting deposit...");
    msg!("name: {}", name);
    msg!("amount: {}", amount);

    let clock = &Clock::get()?;

    let mut account_info_iter = &mut accounts.iter();

    let manager = next_account_info(&mut account_info_iter)?;
    let vault_account = next_account_info(&mut account_info_iter)?;
    let vault_token_account = next_account_info(&mut account_info_iter)?;

    let drift_program = next_account_info(&mut account_info_iter)?;
    let drift_user = next_account_info(&mut account_info_iter)?;
    let drift_user_stats = next_account_info(&mut account_info_iter)?;
    let drift_state = next_account_info(&mut account_info_iter)?;
    let drift_spot_market_vault = next_account_info(&mut account_info_iter)?;
    let drift_oracle = next_account_info(&mut account_info_iter)?;
    let drift_spot_market = next_account_info(&mut account_info_iter)?;

    Ok(())
}
