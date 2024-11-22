use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

pub fn initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
) -> ProgramResult {
    msg!("Initializing Vault...");
    msg!("Vault Id: {}", vault_id);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let treasury = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Print each variable
    msg!("initializer: {}", initializer.key);
    msg!("vault: {}", vault.key);
    msg!("treasury: {}", treasury.key);
    msg!("system_program: {}", system_program.key);

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[vault_id.as_bytes().as_ref()], program_id);

    if vault_pda != *vault.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(0);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            vault.key,
            rent_lamports,
            0,
            program_id,
        ),
        &[
            initializer.clone(),
            vault.clone(),
            system_program.clone(),
        ],
        &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    msg!("Vault created: {}", vault_pda);

    // Create Treasury PDA
    let (treasury_pda, treasury_bump_seed) =
        Pubkey::find_program_address(&[b"treasury", vault_id.as_bytes().as_ref()], program_id);

    if treasury_pda != *treasury.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            treasury.key,
            rent_lamports,
            0,
            program_id,
        ),
        &[
            initializer.clone(),
            treasury.clone(),
            system_program.clone(),
        ],
        &[&[
            b"treasury",
            vault_id.as_bytes().as_ref(),
            &[treasury_bump_seed],
        ]],
    )?;

    msg!("Treasury created: {}", treasury_pda);

    Ok(())
}
