use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    common::bytes32_to_string, drift::{UpdateUserReduceOnlyIxArgs, UpdateUserReduceOnlyIxData}, state::Vault
};

pub fn reset_delegate<'a>(_program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
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

    let mut vault = Vault::get(vault_account);

    if vault.manager != *manager.key {
        msg!("Invalid Manager Account");
        return Err(ProgramError::InvalidArgument);
    }

    if vault.delegate == Pubkey::default() {
        msg!("Delegate is not set yet");
        return Err(ProgramError::InvalidArgument);
    }

    vault.delegate = Pubkey::default();

    Vault::save(&vault, vault_account)?;

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
    let args = UpdateUserReduceOnlyIxArgs {
        sub_account_id: 0,
        reduce_only: false,
    };
    let data: UpdateUserReduceOnlyIxData = args.into();

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
        &[&[
            b"vault",
            bytes32_to_string(vault.name).as_ref(),
            &[vault.bump],
        ]],
    )?;

    Ok(())
}
