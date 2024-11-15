use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use drift_interface::{
    initialize_user_ix, initialize_user_stats_ix, InitializeUserAccounts, InitializeUserIxArgs,
    InitializeUserKeys, InitializeUserStatsAccounts, InitializeUserStatsKeys,
};

pub fn initialize_drift(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
) -> ProgramResult {
    msg!("Initializing Drift Vault...");
    msg!("Vault Id: {}", vault_id);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let treasury = next_account_info(account_info_iter)?;

    let drift_program = next_account_info(account_info_iter)?;
    let drift_user = next_account_info(account_info_iter)?;
    let drift_user_stats = next_account_info(account_info_iter)?;
    let drift_state = next_account_info(account_info_iter)?;

    let rent = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // First batch - Main accounts
    msg!("1. initializer: {}", initializer.key);
    msg!("2. vault: {}", vault.key);
    msg!("3. treasury: {}", treasury.key);

    // Second batch - Drift accounts
    msg!("4. drift_program: {}", drift_program.key);
    msg!("5. drift_user: {}", drift_user.key);
    msg!("6. drift_user_stats: {}", drift_user_stats.key);
    msg!("7. drift_state: {}", drift_state.key);

    // Third batch - System accounts
    msg!("8. rent: {}", rent.key);
    msg!("9. system_program: {}", system_program.key);

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

    msg!("Vault PDA: {}", vault_pda);

    // Create Treasury PDA
    let (treasury_pda, _) =
        Pubkey::find_program_address(&[b"treasury", vault_id.as_bytes().as_ref()], program_id);

    if treasury_pda != *treasury.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Treasury PDA: {}", treasury_pda);

    if drift_interface::ID != *drift_program.key {
        msg!("Invalid Drift Program");
        return Err(ProgramError::InvalidArgument);
    }

    // initializeUserStats cpi
    let user_stats_accounts = InitializeUserStatsAccounts {
        user_stats: drift_user_stats,
        state: drift_state,
        authority: vault,
        payer: initializer,
        rent,
        system_program,
    };

    let user_stats_keys: InitializeUserStatsKeys =
        InitializeUserStatsKeys::from(user_stats_accounts);

    let user_stats_ix = initialize_user_stats_ix(user_stats_keys)?;

    invoke_signed(
        &user_stats_ix,
        &[
            drift_program.clone(),
            drift_user_stats.clone(),
            drift_state.clone(),
            vault.clone(),
            initializer.clone(),
            rent.clone(),
            system_program.clone(),
        ],
        &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    // initializeUser cpi
    let mut name = [0u8; 32];
    let bytes = vault_id.as_bytes();
    name[..bytes.len()].copy_from_slice(bytes);

    let user_accounts = InitializeUserAccounts {
        user: drift_user,
        user_stats: drift_user_stats,
        state: drift_state,
        authority: vault,
        payer: initializer,
        rent,
        system_program,
    };

    let user_keys: InitializeUserKeys = InitializeUserKeys::from(user_accounts);
    let user_args = InitializeUserIxArgs {
        sub_account_id: 0,
        name,
    };

    let user_ix = initialize_user_ix(user_keys, user_args)?;

    invoke_signed(
        &user_ix,
        &[
            drift_program.clone(),
            drift_user.clone(),
            drift_user_stats.clone(),
            drift_state.clone(),
            vault.clone(),
            initializer.clone(),
            rent.clone(),
            system_program.clone(),
        ],
        &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    Ok(())
}