use crate::{
    common::{log_accounts, log_params, string_to_bytes32},
    constants::PERCENTAGE_PRECISION,
    drift::{self, InitializeUserIxArgs, InitializeUserIxData, InitializeUserStatsIxData},
    error::ErrorCode,
    state::Treasury,
};
use serde::Serialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
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

pub fn initialize_drift_vault_with_bulk<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    params: &VaultParams,
) -> ProgramResult {
    msg!("Initializing Drift vault with bulk...");
    log_params(&params);

    let account_info_iter = &mut accounts.iter();

    let manager = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_token_account = next_account_info(account_info_iter)?;
    let treasury_account = next_account_info(account_info_iter)?;
    let treasury_token_account = next_account_info(account_info_iter)?;

    let drift_program = next_account_info(account_info_iter)?;
    let drift_user = next_account_info(account_info_iter)?;
    let drift_user_stats = next_account_info(account_info_iter)?;
    let drift_state = next_account_info(account_info_iter)?;

    let rent = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    log_accounts(&[
        (manager, "Manager"),
        (vault_account, "Vault Account"),
        (vault_token_account, "Vault Token Account"),
        (treasury_account, "Treasury"),
        (treasury_token_account, "Treasury Token Account"),
        (drift_program, "Drift Program"),
        (drift_user, "Drift User"),
        (drift_user_stats, "Drift User Stats"),
        (drift_state, "Drift State"),
        (rent, "Rent"),
        (system_program, "System Program"),
    ]);

    if !manager.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if drift::ID != *drift_program.key {
        msg!("Invalid Drift Program");
        return Err(ProgramError::InvalidArgument);
    }

    let account_len: usize = 1000;

    let rent_cal = Rent::get()?;
    let rent_lamports = rent_cal.minimum_balance(account_len);

    initialize_vault(
        program_id,
        manager,
        vault_account,
        vault_token_account,
        drift_user_stats,
        drift_user,
        &params,
        system_program,
        account_len,
        rent_lamports,
    )?;

    initialize_treasury(
        &program_id,
        manager,
        vault_account,
        treasury_account,
        treasury_token_account,
        system_program,
        &params,
        account_len,
        rent_lamports,
    )?;

    initialize_user_stats(
        &program_id,
        drift_program,
        drift_user_stats,
        drift_state,
        vault_account,
        manager,
        rent,
        system_program,
        &params,
    )?;

    initialize_user(
        program_id,
        drift_program,
        drift_user,
        drift_user_stats,
        drift_state,
        vault_account,
        manager,
        rent,
        system_program,
        &params,
    )?;

    Ok(())
}

#[derive(Serialize)]
pub struct VaultParams {
    pub name: String,
    pub redeem_period: u64,
    pub max_tokens: u64,
    pub management_fee: u64,
    pub min_deposit_amount: u64,
    pub profit_share: u32,
    pub hurdle_rate: u32,
    pub spot_market_index: u16,
    pub permissioned: bool,
}

fn initialize_vault<'a>(
    program_id: &Pubkey,
    manager: &'a AccountInfo<'a>,
    vault_account: &'a AccountInfo<'a>,
    vault_token_account: &'a AccountInfo<'a>,
    drift_user_stats: &'a AccountInfo<'a>,
    drift_user: &'a AccountInfo<'a>,
    params: &VaultParams,
    system_program: &'a AccountInfo<'a>,
    account_len: usize,
    rent_lamports: u64,
) -> ProgramResult {
    let (vault_pda, vault_bump_seed) = Vault::get_pda(&params.name, program_id);

    if vault_pda != *vault_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            manager.key,
            vault_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            manager.clone(),
            vault_account.clone(),
            system_program.clone(),
        ],
        &[&[
            b"vault",
            params.name.as_bytes().as_ref(),
            &[vault_bump_seed],
        ]],
    )?;

    msg!("Vault created: {}", vault_pda);

    let mut vault = Vault::get(vault_account);
    msg!("borrowed new account data");

    let name_32 = string_to_bytes32(&params.name);
    vault.name = name_32;
    vault.pubkey = *vault_account.key;
    vault.manager = *manager.key;
    vault.token_account = *vault_token_account.key;
    vault.user_stats = *drift_user_stats.key;
    vault.user = *drift_user.key;
    vault.total_shares = 0;
    vault.redeem_period = params.redeem_period;
    vault.max_tokens = params.max_tokens;

    if params.management_fee >= PERCENTAGE_PRECISION {
        msg!("management fee must be < 100%");
        return Err(ErrorCode::InvalidVaultInitialization.into());
    }

    vault.management_fee = params.management_fee;

    vault.init_ts = Clock::get()?.unix_timestamp as u64;
    vault.min_deposit_amount = params.min_deposit_amount;

    if params.profit_share >= PERCENTAGE_PRECISION as u32 {
        msg!("profit share must be < 100%");
        return Err(ErrorCode::InvalidVaultInitialization.into());
    }
    vault.profit_share = params.profit_share;

    vault.spot_market_index = params.spot_market_index;
    vault.bump = vault_bump_seed;
    vault.permissioned = params.permissioned;

    Vault::save(&vault, vault_account);

    msg!("Successfully initialized vault");
    Ok(())
}

fn initialize_treasury<'a>(
    program_id: &Pubkey,
    manager: &'a AccountInfo<'a>,
    vault_account: &'a AccountInfo<'a>,
    treasury_account: &'a AccountInfo<'a>,
    treasury_token_account: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    params: &VaultParams,
    account_len: usize,
    rent_lamports: u64,
) -> ProgramResult {
    let (treasury_pda, treasury_bump_seed) = Treasury::get_pda(&params.name, program_id);

    if treasury_pda != *treasury_account.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            manager.key,
            treasury_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            manager.clone(),
            treasury_account.clone(),
            system_program.clone(),
        ],
        &[&[
            b"treasury",
            params.name.as_bytes().as_ref(),
            &[treasury_bump_seed],
        ]],
    )?;

    msg!("Treasury created: {}", treasury_pda);

    let mut treasury = Treasury::get(treasury_account);
    treasury.pubkey = *treasury_account.key;
    treasury.manager = *manager.key;
    treasury.vault = *vault_account.key;
    treasury.token_account = *treasury_token_account.key;
    treasury.bump = treasury_bump_seed;

    Treasury::save(&treasury, treasury_account);

    Ok(())
}

fn initialize_user_stats<'a>(
    program_id: &Pubkey,
    drift_program: &'a AccountInfo<'a>,
    drift_user_stats: &'a AccountInfo<'a>,
    drift_state: &'a AccountInfo<'a>,
    vault_account: &'a AccountInfo<'a>,
    manager: &'a AccountInfo<'a>,
    rent: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    params: &VaultParams,
) -> ProgramResult {
    let (_, vault_bump_seed) = Vault::get_pda(&params.name, program_id);

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
            pubkey: *vault_account.key,
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
            vault_account.clone(),
            manager.clone(),
            rent.clone(),
            system_program.clone(),
        ],
        &[&[
            b"vault",
            params.name.as_bytes().as_ref(),
            &[vault_bump_seed],
        ]],
    )?;

    Ok(())
}

fn initialize_user<'a>(
    program_id: &Pubkey,
    drift_program: &'a AccountInfo<'a>,
    drift_user: &'a AccountInfo<'a>,
    drift_user_stats: &'a AccountInfo<'a>,
    drift_state: &'a AccountInfo<'a>,
    vault_account: &'a AccountInfo<'a>,
    manager: &'a AccountInfo<'a>,
    rent: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    params: &VaultParams,
) -> ProgramResult {
    let (_, vault_bump_seed) = Vault::get_pda(&params.name, program_id);

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
            pubkey: *vault_account.key,
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
        name: string_to_bytes32(&params.name),
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
            vault_account.clone(),
            manager.clone(),
            rent.clone(),
            system_program.clone(),
        ],
        &[&[
            b"vault",
            params.name.as_bytes().as_ref(),
            &[vault_bump_seed],
        ]],
    )?;

    Ok(())
}
