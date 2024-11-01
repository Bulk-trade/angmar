use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::AccountMeta,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use drift_interface::{
    initialize_user_invoke_signed, initialize_user_ix, initialize_user_stats_invoke, initialize_user_stats_invoke_signed, initialize_user_stats_ix, InitializeUserAccounts, InitializeUserIxArgs, InitializeUserKeys, InitializeUserStatsAccounts, InitializeUserStatsKeys, ID
};

pub fn initialize_drift(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
) -> ProgramResult {
    msg!("Initializing Vault...");
    msg!("Vault Id: {}", vault_id);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let vault_pda_account = next_account_info(account_info_iter)?;
    let treasury_pda_account = next_account_info(account_info_iter)?;
    let drift_program = next_account_info(account_info_iter)?;
    let user = next_account_info(account_info_iter)?;
    let user_stats = next_account_info(account_info_iter)?;
    let state = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let rent = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Print each variable
    msg!("initializer: {}", initializer.key);
    msg!("vault_pda_account: {}", vault_pda_account.key);
    msg!("treasury_pda_account: {}", treasury_pda_account.key);
    msg!("drift_program: {}", drift_program.key);
    msg!("user: {}", user.key);
    msg!("user_stats: {}", user_stats.key);
    msg!("state: {}", state.key);
    msg!("authority: {}", authority.key);
    msg!("payer: {}", payer.key);
    msg!("user_rent: {}", rent.key);
    msg!("system_program: {}", system_program.key);

    // if !initializer.is_signer {
    //     msg!("Missing required signature");
    //     return Err(ProgramError::MissingRequiredSignature);
    // }

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[vault_id.as_bytes().as_ref()], program_id);

    if vault_pda != *vault_pda_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Vault PDA: {}", vault_pda);

    // Create Treasury PDA
    let (treasury_pda, _treasury_bump_seed) =
        Pubkey::find_program_address(&[b"treasury", vault_id.as_bytes().as_ref()], program_id);

    if treasury_pda != *treasury_pda_account.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Treasury PDA: {}", treasury_pda);

    //Initialize user stats
    // initialize_user_stats_invoke_signed(
    //     InitializeUserStatsAccounts {
    //         user_stats,
    //         state,
    //         authority,
    //         payer,
    //         rent,
    //         system_program,
    //     },
    //     &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]],
    // )?;

    // let (authority_pda, signer_bump) = Pubkey::find_program_address(&[b"signer", vault_id.as_bytes().as_ref()], program_id);

    // if authority_pda != *authority.key {
    //     msg!("Invalid seeds for Authority PDA");
    //     return Err(ProgramError::InvalidArgument);
    // }

    if drift_interface::ID != *drift_program.key {
        msg!("Invalid Drift Program");
        return Err(ProgramError::InvalidArgument);
    }

    let user_stats_accounts = InitializeUserStatsAccounts {
        user_stats,
        state,
        authority,
        payer,
        rent,
        system_program,
    };

    let user_stats_keys: InitializeUserStatsKeys =
        InitializeUserStatsKeys::from(user_stats_accounts);

    let user_stats_ix = initialize_user_stats_ix(user_stats_keys)?;

    invoke(
        &user_stats_ix,
        &[
            drift_program.clone(),
            user_stats.clone(),
            state.clone(),
            authority.clone(),
            payer.clone(),
            rent.clone(),
            system_program.clone(),
        ],
    )?;

     let user_accounts = InitializeUserAccounts {
        user,
        user_stats,
        state,
        authority,
        payer,
        rent,
        system_program,
    };

    let user_keys: InitializeUserKeys = InitializeUserKeys::from(user_accounts);
    let user_args = InitializeUserIxArgs {
        sub_account_id: 0,
        name: [1; 32],
    };

    let user_ix = initialize_user_ix(user_keys, user_args)?;

      invoke(
        &user_ix,
        &[
            drift_program.clone(),
            user.clone(),
            user_stats.clone(),
            state.clone(),
            authority.clone(),
            payer.clone(),
            rent.clone(),
            system_program.clone(),
        ]
    )?;

    //    let ix =  solana_program::instruction::Instruction{
    //     program_id: *drift_program.key,
    //     accounts: vec![
    //         // AccountMeta::new(*drift_program.key, false),
    //         AccountMeta::new(*user_stats.key, false),
    //         AccountMeta::new(*state.key, false),
    //         AccountMeta::new(*authority.key, true),
    //         AccountMeta::new(*payer.key, true),
    //         AccountMeta::new_readonly(*rent.key, false),
    //         AccountMeta::new_readonly(*system_program.key, false)
    //     ],
    //     data: Vec::new()
    //    };

    // invoke_signed(
    //     &ix,
    //     &[
    //         drift_program.clone(),
    //         user_stats.clone(),
    //         state.clone(),
    //         authority.clone(),
    //         payer.clone(),
    //         rent.clone(),
    //         system_program.clone(),
    //     ],
    //     &[&[
    //         b"signer",
    //         vault_id.as_bytes().as_ref(),
    //         &[signer_bump],
    //     ]],
    // )?;

    // initialize_user_stats_invoke(InitializeUserStatsAccounts {
    //     user_stats,
    //     state,
    //     authority,
    //     payer,
    //     rent,
    //     system_program,
    // })?;

    // let mut name = [0u8; 32];
    // let bytes = vault_id.as_bytes();
    // name[..bytes.len()].copy_from_slice(bytes);

    // //Initialize user
    // initialize_user_invoke_signed(
    //     InitializeUserAccounts {
    //         user,
    //         user_stats,
    //         state,
    //         authority,
    //         payer,
    //         rent: user_rent,
    //         system_program,
    //     },
    //     InitializeUserIxArgs {
    //         sub_account_id: 0,
    //         name: name,
    //     },
    //     &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]],
    // )?;

    Ok(())
}
