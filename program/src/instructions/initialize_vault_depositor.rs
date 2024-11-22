use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_10::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use borsh::BorshSerialize;
use crate::{
    error::VaultError,
    state::{Vault, VaultDepositor},
};

pub fn initialize_vault_depositor(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Initializing Vault Depositor...");

    let account_info_iter = &mut accounts.iter();

    let vault = next_account_info(account_info_iter)?;
    let vault_depositor = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;

    let system_program = next_account_info(account_info_iter)?;

    msg!("Vault: {}", vault.key);
    msg!("Vault Depositor: {}", vault_depositor.key);
    msg!("Authority: {}", authority.key);
    msg!("System Program: {}", system_program.key);

    if !authority.is_signer {
        msg!("Vault depositor authority must pay to create account");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_depositor_pda, vault_depositor_bump_seed) = Pubkey::find_program_address(
        &[
            b"vault_depositor",
            vault.key.as_ref(),
            authority.key.as_ref(),
        ],
        program_id,
    );

    if vault_depositor_pda != *vault_depositor.key {
        msg!("Invalid seeds for Vault Depositor PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let account_len: usize = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            vault_depositor.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            authority.clone(),
            vault_depositor.clone(),
            system_program.clone(),
        ],
        &[&[
            b"vault_depositor",
            vault.key.as_ref(),
            authority.key.as_ref(),
            &[vault_depositor_bump_seed],
        ]],
    )?;

    msg!("Vault Depositor created: {}", vault_depositor_pda);

    let mut data =
        try_from_slice_unchecked::<VaultDepositor>(&vault_depositor.data.borrow()).unwrap();
    msg!("borrowed new account data");

    data.vault = *vault.key;
    data.pubkey = *vault_depositor.key;
    data.authority = *authority.key;

    msg!("Vault: {:?}", data.vault);
    msg!("Pubkey: {:?}", data.pubkey);
    msg!("Authority: {:?}", data.authority);

    msg!("unpacking vault state account");
    let vault_data = try_from_slice_unchecked::<Vault>(&vault.data.borrow())?;

    // Print the specified fields from vault_data
    msg!("Name: {:?}", vault_data.name);
    msg!("Pubkey: {:?}", vault_data.pubkey);
    msg!("Manager: {:?}", vault_data.manager);
    msg!("User Stats: {:?}", vault_data.user_stats);
    msg!("User: {:?}", vault_data.user);
    msg!("Token Account: {:?}", vault_data.token_account);
    msg!("Spot Market Index: {:?}", vault_data.spot_market_index);
    msg!("Init Timestamp: {:?}", vault_data.init_ts);
    msg!("Min Deposit Amount: {:?}", vault_data.min_deposit_amount);
    msg!("Management Fee: {:?}", vault_data.management_fee);
    msg!("Profit Share: {:?}", vault_data.profit_share);
    msg!("Bump: {:?}", vault_data.bump);
    msg!("Permissioned: {:?}", vault_data.permissioned);

    if vault_data.permissioned {
        if vault_data.manager != *authority.key {
            msg!("Vault depositor can only be created by vault manager");
            return Err(VaultError::PermissionedVault.into());
        }
    }

    // Serialize and save the data back to the account
    msg!("serializing account");
    data.serialize(&mut &mut vault_depositor.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
}
