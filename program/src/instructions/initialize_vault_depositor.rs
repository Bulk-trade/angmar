use crate::{
    error::ErrorCode,
    state::{Vault, VaultDepositor},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};

pub fn initialize_vault_depositor<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    msg!("Initializing Vault Depositor...");

    let account_info_iter = &mut accounts.iter();

    let vault_account = next_account_info(account_info_iter)?;
    let vault_depositor_account = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;

    let system_program = next_account_info(account_info_iter)?;

    msg!("Vault: {}", vault_account.key);
    msg!("Vault Depositor: {}", vault_depositor_account.key);
    msg!("Authority: {}", authority.key);
    msg!("System Program: {}", system_program.key);

    if !authority.is_signer {
        msg!("Vault depositor authority must pay to create account");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_depositor_pda, vault_depositor_bump_seed) =
        VaultDepositor::get_pda(vault_account.key, authority.key, program_id);

    if vault_depositor_pda != *vault_depositor_account.key {
        msg!("Invalid seeds for Vault Depositor PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Verify vault permissions
    let vault = Vault::get(vault_account);
    if vault.permissioned {
        if vault.manager != *authority.key {
            msg!("Vault depositor can only be created by vault manager");
            return Err(ErrorCode::PermissionedVault.into());
        }
    }

    if vault.pubkey != *vault_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    initialize_depositor(
        vault_account.key,
        vault_depositor_account,
        authority,
        system_program,
        program_id,
        vault_depositor_bump_seed,
    )?;
    Ok(())
}

fn initialize_depositor<'a>(
    vault_account_pubkey: &Pubkey,
    vault_depositor_account: &'a AccountInfo<'a>,
    authority: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    program_id: &Pubkey,
    vault_depositor_bump_seed: u8,
) -> Result<VaultDepositor, ProgramError> {
    //Create depositor pda
    let account_len: usize = 1000;

    // Calculate rent
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            vault_depositor_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            authority.clone(),
            vault_depositor_account.clone(),
            system_program.clone(),
        ],
        &[&[
            b"vault_depositor",
            vault_account_pubkey.as_ref(),
            authority.key.as_ref(),
            &[vault_depositor_bump_seed],
        ]],
    )?;

    msg!("Vault Depositor created: {}", vault_depositor_account.key);

    // Create and initialize vault depositor
    let mut vault_depositor = VaultDepositor::get(vault_depositor_account);

    // Set initial state
    vault_depositor.vault = *vault_account_pubkey;
    vault_depositor.pubkey = *vault_depositor_account.key;
    vault_depositor.authority = *authority.key;
    vault_depositor.init_ts = Clock::get()?.unix_timestamp as u64;

    // Save state
    VaultDepositor::save(&vault_depositor, vault_depositor_account);

    msg!("Successfully initialized vault depositor state");
    Ok(vault_depositor)
}
