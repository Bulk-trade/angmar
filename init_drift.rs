use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use drift_interface::{
    initialize_user_invoke_signed,
    initialize_user_stats_invoke,
    InitializeUserAccounts,
    InitializeUserIxArgs,
    InitializeUserStatsAccounts,
};

pub fn initialize_drift(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
) -> ProgramResult {
    msg!("Initializing Drift User for Vault...");
    msg!("Vault Id: {}", vault_id);

    let account_info_iter = &mut accounts.iter();

    // Accounts expected by the function
    let initializer = next_account_info(account_info_iter)?;          // Account 0
    let vault_pda_account = next_account_info(account_info_iter)?;    // Account 1
    let treasury_pda_account = next_account_info(account_info_iter)?; // Account 2
    let user = next_account_info(account_info_iter)?;                 // Account 3
    let user_stats = next_account_info(account_info_iter)?;           // Account 4
    let state = next_account_info(account_info_iter)?;                // Account 5
    let authority = vault_pda_account;                                // Use vault_pda_account as authority
    let payer = next_account_info(account_info_iter)?;                // Account 6
    let rent = next_account_info(account_info_iter)?;                 // Account 7
    let system_program = next_account_info(account_info_iter)?;       // Account 8
    let drift_signer = next_account_info(account_info_iter)?;         // Account 9

    // Verify PDAs
    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[vault_id.as_bytes()], program_id);

    if vault_pda != *vault_pda_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    // Seeds for signing
    let vault_seeds: &[&[u8]] = &[vault_id.as_bytes(), &[vault_bump_seed]];

    // Prepare the 'name' variable
    let mut name = [0u8; 32];
    let vault_id_bytes = vault_id.as_bytes();
    let name_slice = &mut name[..vault_id_bytes.len()];
    name_slice.copy_from_slice(vault_id_bytes);

    // Initialize user stats (if necessary)
    initialize_user_stats_invoke(
        InitializeUserStatsAccounts {
            user_stats,
            state,
            authority,
            payer,
            rent,
            system_program,
        },
        // Removed the second argument as the function only takes one argument
    )?;

    // Initialize user
    initialize_user_invoke_signed(
        InitializeUserAccounts {
            user,
            user_stats,
            state,
            authority,
            payer,
            rent,
            system_program,
        },
        InitializeUserIxArgs {
            sub_account_id: 0,
            name,
        },
        &[vault_seeds], // Use vault_seeds for signing
    )?;

    Ok(())
}
