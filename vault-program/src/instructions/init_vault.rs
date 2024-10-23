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
    let vault_pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_pda, vault_bump_seed) = Pubkey::find_program_address(
        &[vault_id.as_bytes().as_ref()],
        program_id,
    );

    if vault_pda != *vault_pda_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(0);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            vault_pda_account.key,
            rent_lamports,
            0,
            program_id,
        ),
        &[
            initializer.clone(),
            vault_pda_account.clone(),
            system_program.clone(),
        ],
        &[&[
            vault_id.as_bytes().as_ref(),
            &[vault_bump_seed],
        ]],
    )?;

    msg!("PDA created: {}", vault_pda);

    Ok(())
}
