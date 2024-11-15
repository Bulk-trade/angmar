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

pub fn update_delegate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
    delegate: String,
    sub_account: u16,
    fund_status: String,
    bot_status: String,
) -> ProgramResult {
    msg!("Updating delegate...");
    msg!("vault_id: {}", vault_id);
    msg!("delegate: {}", delegate);
    msg!("sub account: {}", sub_account);
    msg!("fund_status: {}", fund_status);
    msg!("bot_status: {}", bot_status);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;

    let drift_program = next_account_info(account_info_iter)?;
    let drift_user = next_account_info(account_info_iter)?;
    // First batch - Main accounts
    msg!("1. initializer: {}", initializer.key);
    msg!("2. vault: {}", vault.key);

    // Second batch - Drift accounts
    msg!("3. drift_program: {}", drift_program.key);
    msg!("4. drift_user: {}", drift_user.key);

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Cpi to Drift");
    let accounts_meta = vec![
        AccountMeta {
            pubkey: *drift_user.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *vault.key,
            is_signer: true,
            is_writable: true,
        },
    ];

    let delegate_pubkey = match Pubkey::from_str(&delegate) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            panic!("Delegate string is not a valid Pubkey: {}", delegate);
        }
    };

    let args = UpdateUserDelegateIxArgs {
        sub_account_id: sub_account,
        delegate: delegate_pubkey,
    };

    let data: UpdateUserDelegateIxData = args.into();

    let ix = Instruction {
        program_id: *drift_program.key,
        accounts: accounts_meta,
        data: data.try_to_vec()?,
    };

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[vault_id.as_bytes().as_ref()], program_id);

    msg!("Vault PDA: {}", vault_pda);

    if vault_pda != *vault.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &ix,
        &[drift_user.clone(), vault.clone(), drift_program.clone()],
        &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    Ok(())
}
