use crate::{
    drift::{self, InitializeUserIxArgs, InitializeUserIxData, InitializeUserStatsIxData},
    error::VaultError,
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_10::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::state::Vault;

pub fn initialize_drift_vault_with_bulk(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    management_fee: u64,
    min_deposit_amount: u64,
    profit_share: u32,
    spot_market_index: u16,
    permissioned: bool,
) -> ProgramResult {
    msg!("Initializing drift vault with bulk...");
    msg!("Vault Name: {}", name);
    msg!("Management Fee: {}", management_fee);
    msg!("Min Deposit Amount: {}", min_deposit_amount);
    msg!("Profit Share: {}", profit_share);
    msg!("Spot Market Index: {}", spot_market_index);
    msg!("Permissioned: {}", permissioned);

    let account_info_iter = &mut accounts.iter();

    let manager = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let vault_token_account = next_account_info(account_info_iter)?;
    let treasury = next_account_info(account_info_iter)?;

    let drift_program = next_account_info(account_info_iter)?;
    let drift_user = next_account_info(account_info_iter)?;
    let drift_user_stats = next_account_info(account_info_iter)?;
    let drift_state = next_account_info(account_info_iter)?;

    let rent = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    msg!("Manager: {}", manager.key);
    msg!("Vault: {}", vault.key);
    msg!("Vault Token Account: {}", vault_token_account.key);
    msg!("Treasury: {}", treasury.key);
    msg!("Drift Program: {}", drift_program.key);
    msg!("Drift User: {}", drift_user.key);
    msg!("Drift User Stats: {}", drift_user_stats.key);
    msg!("Drift State: {}", drift_state.key);
    msg!("Rent: {}", rent.key);
    msg!("System Program: {}", system_program.key);

    if !manager.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[b"vault", name.as_bytes().as_ref()], program_id);

    if vault_pda != *vault.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let account_len: usize = 1000;

    let rent_cal = Rent::get()?;
    let rent_lamports = rent_cal.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            manager.key,
            vault.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[manager.clone(), vault.clone(), system_program.clone()],
        &[&[b"vault", name.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    msg!("Vault created: {}", vault_pda);

    let mut data = try_from_slice_unchecked::<Vault>(&vault.data.borrow()).unwrap();
    msg!("borrowed new account data");

    let mut name_32 = [0u8; 32];
    let name_bytes = name.as_bytes();
    name_32[..name_bytes.len()].copy_from_slice(name_bytes);
    data.name = name_32;
    data.pubkey = *vault.key;
    data.manager = *manager.key;
    data.user_stats = *drift_user_stats.key;
    data.user = *drift_user.key;
    data.token_account = *vault_token_account.key;
    data.spot_market_index = spot_market_index;
    data.init_ts = Clock::get()?.unix_timestamp as u64;
    data.min_deposit_amount = min_deposit_amount;

    let percentage_precision = 1_000_000 as u64;

    if !management_fee < percentage_precision {
        msg!("management fee must be < 100%");
        return Err(VaultError::InvalidVaultInitialization.into());
    }

    data.management_fee = management_fee;

    if !profit_share < percentage_precision as u32 {
        msg!("profit share must be < 100%");
        return Err(VaultError::InvalidVaultInitialization.into());
    }
    data.profit_share = profit_share;
    data.bump = vault_bump_seed;
    data.permissioned = permissioned;

    msg!("serializing account");
    data.serialize(&mut &mut vault.data.borrow_mut()[..])?;
    msg!("state account serialized");

    // Create Treasury PDA
    let (treasury_pda, treasury_bump_seed) =
        Pubkey::find_program_address(&[b"treasury", name.as_bytes().as_ref()], program_id);

    if treasury_pda != *treasury.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            manager.key,
            treasury.key,
            rent_lamports,
            0,
            program_id,
        ),
        &[manager.clone(), treasury.clone(), system_program.clone()],
        &[&[b"treasury", name.as_bytes().as_ref(), &[treasury_bump_seed]]],
    )?;

    msg!("Treasury created: {}", treasury_pda);

    if drift::ID != *drift_program.key {
        msg!("Invalid Drift Program");
        return Err(ProgramError::InvalidArgument);
    }

    // initializeUserStats cpi
    let user_stats_accounts = vec![
        AccountMeta {
            pubkey: *drift_user_stats.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_state.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *vault.key,
            is_signer: true,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *manager.key,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *rent.key,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *system_program.key,
            is_signer: false,
            is_writable: false,
        },
    ];

    let user_stats_ix = Instruction {
        program_id: *drift_program.key,
        accounts: user_stats_accounts,
        data: InitializeUserStatsIxData.try_to_vec()?,
    };

    invoke_signed(
        &user_stats_ix,
        &[
            drift_program.clone(),
            drift_user_stats.clone(),
            drift_state.clone(),
            vault.clone(),
            manager.clone(),
            rent.clone(),
            system_program.clone(),
        ],
        &[&[b"vault", name.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    // initializeUser cpi

    let user_accounts = vec![
        AccountMeta {
            pubkey: *drift_user.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_user_stats.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_state.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *vault.key,
            is_signer: true,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *manager.key,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *rent.key,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *system_program.key,
            is_signer: false,
            is_writable: false,
        },
    ];

    let user_args = InitializeUserIxArgs {
        sub_account_id: 0,
        name: name_32,
    };

    let data: InitializeUserIxData = user_args.into();

    let user_ix = Instruction {
        program_id: *drift_program.key,
        accounts: user_accounts,
        data: data.try_to_vec()?,
    };

    invoke_signed(
        &user_ix,
        &[
            drift_program.clone(),
            drift_user.clone(),
            drift_user_stats.clone(),
            drift_state.clone(),
            vault.clone(),
            manager.clone(),
            rent.clone(),
            system_program.clone(),
        ],
        &[&[b"vault", name.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    Ok(())
}
