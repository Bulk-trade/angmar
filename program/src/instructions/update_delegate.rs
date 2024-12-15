use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::str::FromStr;

use crate::drift::{UpdateUserDelegateIxArgs, UpdateUserDelegateIxData};
use crate::state::Vault;

pub fn update_vault_delegate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    name: String,
    delegate: String,
    sub_account: u16,
) -> ProgramResult {
    // Log instruction parameters
    msg!("Updating delegate...");
    msg!("vault_id: {}", name);
    msg!("delegate: {}", delegate);
    msg!("sub account: {}", sub_account);

    // Extract accounts in expected order
    let account_info_iter = &mut accounts.iter();
    let manager = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let drift_program = next_account_info(account_info_iter)?;
    let drift_user = next_account_info(account_info_iter)?;

    // Log account addresses for debugging
    msg!("1. initializer: {}", manager.key);
    msg!("2. vault: {}", vault_account.key);
    msg!("3. drift_program: {}", drift_program.key);
    msg!("4. drift_user: {}", drift_user.key);

    // Verify manager signature
    if !manager.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify vault PDA
    let (vault_pda, vault_bump_seed) = Vault::get_pda(&name, program_id);
    if vault_pda != *vault_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Parse delegate public key
    let delegate_pubkey = match Pubkey::from_str(&delegate) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            panic!("Delegate string is not a valid Pubkey: {}", delegate);
        }
    };


    // Update vault state
    let mut vault = Vault::get(vault_account);

    if vault.manager != *manager.key {
        msg!("Invalid Manager Account");
        return Err(ProgramError::InvalidArgument);
    }

    vault.delegate = delegate_pubkey;
    Vault::save(&vault, vault_account);

    // Update delegate through CPI
    update_delegate(
        drift_program,
        drift_user,
        vault_account,
        name,
        sub_account,
        delegate_pubkey,
        vault_bump_seed,
    )?;

    Ok(())
}

fn update_delegate<'a>(
    drift_program: &'a AccountInfo<'a>,
    drift_user: &'a AccountInfo<'a>,
    vault_account: &'a AccountInfo<'a>,
    name: String,
    sub_account: u16,
    delegate_pubkey: Pubkey,
    vault_bump_seed: u8,
) -> ProgramResult {
    msg!("Update delegate CPI to Drift...");

    // Prepare CPI accounts
    let accounts_meta = vec![
        AccountMeta {
            pubkey: *drift_user.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *vault_account.key,
            is_signer: true,
            is_writable: true,
        },
    ];

    // Prepare instruction data
    let args = UpdateUserDelegateIxArgs {
        sub_account_id: sub_account,
        delegate: delegate_pubkey,
    };
    let data: UpdateUserDelegateIxData = args.into();

    // Create and execute CPI
    let ix = Instruction {
        program_id: *drift_program.key,
        accounts: accounts_meta,
        data: data.try_to_vec()?,
    };

    invoke_signed(
        &ix,
        &[
            drift_user.clone(),
            vault_account.clone(),
            drift_program.clone(),
        ],
        &[&[b"vault", name.as_bytes(), &[vault_bump_seed]]],
    )
}
