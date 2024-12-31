use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

use crate::{
    common::{bytes32_to_string, log_accounts, transfer_to_user_from_treasury},
    state::{Treasury, Vault},
};

pub fn manager_collect_fees<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    amount: u64,
) -> ProgramResult {
    msg!("Collecting fees...");
    msg!("amount: {}", amount);

    let clock = &Clock::get()?;

    let mut account_info_iter = &mut accounts.iter();

    let manager = next_account_info(&mut account_info_iter)?;
    let vault_account = next_account_info(&mut account_info_iter)?;
    let treasury_account = next_account_info(&mut account_info_iter)?;

    let manager_token_account = next_account_info(&mut account_info_iter)?;
    let treasury_token_account = next_account_info(&mut account_info_iter)?;
    let mint = next_account_info(&mut account_info_iter)?;

    let token_program = next_account_info(&mut account_info_iter)?;

    log_accounts(&[
        // Vault accounts
        (manager, "Manager"),
        (vault_account, "Vault"),
        (treasury_account, "Treasury"),
        // Token accounts
        (manager_token_account, "Manager Token Account"),
        (treasury_token_account, "Treasury Token Account"),
        (mint, "Mint"),
        // System accounts
        (token_program, "Token Program"),
    ]);

    if !manager.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut vault = Vault::get(vault_account);

    if vault.manager != *manager.key {
        msg!("Invalid Vault Manager");
        return Err(ProgramError::InvalidArgument);
    }

    vault.manager_collect_fees(amount, clock.unix_timestamp)?;

    Vault::save(&vault, vault_account)?;

    let vault_name = bytes32_to_string(vault.name);

    let (treasury_pda, treasury_bump_seed) = Treasury::get_pda(&vault_name, program_id);

    if treasury_pda != *treasury_account.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let binding = [treasury_bump_seed];
    let signature_seeds = Treasury::get_treasury_signer_seeds(&vault_name, &binding);

    transfer_to_user_from_treasury(
        amount,
        token_program,
        manager_token_account,
        treasury_token_account,
        treasury_account,
        mint,
        signature_seeds,
    )?;

    Ok(())
}
