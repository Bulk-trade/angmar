use crate::common::{deserialize_zero_copy, log_accounts, transfer_fees, transfer_to_vault};
use crate::drift::{DepositIxArgs, DepositIxData};
use crate::error::wrap_drift_error;
use crate::state::{DepositInfo, Vault, VaultDepositor};
use drift::instructions::optional_accounts::{load_maps, AccountMaps};
use drift::state::spot_market_map::get_writable_spot_market_set;
use drift::state::user::User;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use std::collections::BTreeSet;

pub fn deposit<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    name: String,
    amount: u64,
) -> ProgramResult {
    msg!("Starting deposit...");
    msg!("name: {}", name);
    msg!("amount: {}", amount);

    let clock = &Clock::get()?;

    let mut account_info_iter = accounts.iter();

    let vault_account = next_account_info(&mut account_info_iter)?;
    let vault_depositor_account = next_account_info(&mut account_info_iter)?;
    let authority = next_account_info(&mut account_info_iter)?;
    let treasury = next_account_info(&mut account_info_iter)?;

    let drift_program = next_account_info(&mut account_info_iter)?;
    let drift_user = next_account_info(&mut account_info_iter)?;
    let drift_user_stats = next_account_info(&mut account_info_iter)?;
    let drift_state = next_account_info(&mut account_info_iter)?;
    let drift_spot_market_vault = next_account_info(&mut account_info_iter)?;
    let drift_oracle = next_account_info(&mut account_info_iter)?;
    let drift_spot_market = next_account_info(&mut account_info_iter)?;

    let user_token_account = next_account_info(&mut account_info_iter)?;
    let vault_token_account = next_account_info(&mut account_info_iter)?;
    let treasury_token_account = next_account_info(&mut account_info_iter)?;
    let mint = next_account_info(&mut account_info_iter)?;

    let token_program = next_account_info(&mut account_info_iter)?;
    let system_program = next_account_info(&mut account_info_iter)?;

    log_accounts(&[
        (vault_account, "Vault"),
        (vault_depositor_account, "Vault Depositor"),
        (authority, "Authority"),
        (treasury, "Treasury"),
        // Drift accounts
        (drift_program, "Drift Program"),
        (drift_user, "Drift User"),
        (drift_user_stats, "Drift User Stats"),
        (drift_state, "Drift State"),
        (drift_spot_market_vault, "Drift Spot Market Vault"),
        (drift_oracle, "Drift Oracle"),
        (drift_spot_market, "Drift Spot Market"),
        // Token accounts
        (user_token_account, "User Token Account"),
        (vault_token_account, "Vault Token Account"),
        (treasury_token_account, "Treasury Token Account"),
        (mint, "Mint"),
        // System accounts
        (token_program, "Token Program"),
    ]);

    if !authority.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if drift::ID != *drift_program.key {
        msg!("Invalid Drift Program");
        return Err(ProgramError::InvalidArgument);
    }

    let (vault_depositor_pda, _) =
        VaultDepositor::get_pda(&vault_account.key, &authority.key, program_id);

    if vault_depositor_pda != *vault_depositor_account.key {
        msg!("Invalid seeds for Vault Depositor PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let (vault_pda, vault_bump_seed) = Vault::get_pda(&name, program_id);

    if vault_pda != *vault_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let mut vault = Vault::get(vault_account);
    let mut vault_depositor = VaultDepositor::get(vault_depositor_account);

    let need_resize = vault_depositor.does_need_resize(vault_depositor_account.data.borrow().len());

    if need_resize {
        resize_vault_depositor_account(
            vault_depositor_account,
            authority,
            system_program,
            2, // Space for 2 additional DepositInfo
        )?;
    }

    let writable_spot_market_set = get_writable_spot_market_set(vault.spot_market_index);

    // Load maps with proper account references and types
    let AccountMaps {
        perp_market_map,
        spot_market_map,
        mut oracle_map,
    } = match load_maps(
        &mut accounts[9..11].iter().peekable(),
        &BTreeSet::new(),
        &writable_spot_market_set,
        clock.slot,
        None,
    ) {
        Ok(maps) => maps,
        Err(e) => return Err(ProgramError::Custom(e as u32)),
    };

    // User details
    let user = deserialize_zero_copy::<User>(&*drift_user.try_borrow_data()?);

    let vault_equity = vault
        .calculate_total_equity(&user, &perp_market_map, &spot_market_map, &mut oracle_map)
        .map_err(wrap_drift_error)?;

    let timestamp = clock.unix_timestamp;

    let (deposit_amount, fees) =
        vault_depositor.deposit(amount, vault_equity, &mut vault, timestamp)?;

    VaultDepositor::save(&vault_depositor, vault_depositor_account)?;
    Vault::save(&vault, vault_account)?;

    transfer_fees(
        fees,
        token_program,
        user_token_account,
        treasury_token_account,
        authority,
        mint,
    )?;

    transfer_to_vault(
        deposit_amount,
        token_program,
        user_token_account,
        vault_token_account,
        authority,
        mint,
    )?;

    drift_deposit(
        &vault,
        deposit_amount,
        name,
        vault_bump_seed,
        drift_state,
        drift_user,
        drift_user_stats,
        vault_account,
        drift_spot_market_vault,
        vault_token_account,
        token_program,
        drift_oracle,
        drift_spot_market,
        drift_program,
    )?;

    Ok(())
}

pub fn drift_deposit<'a>(
    vault: &Vault,
    amount: u64,
    name: String,
    vault_bump_seed: u8,
    // Individual accounts
    drift_state: &'a AccountInfo<'a>,
    drift_user: &'a AccountInfo<'a>,
    drift_user_stats: &'a AccountInfo<'a>,
    vault_account: &'a AccountInfo<'a>,
    drift_spot_market_vault: &'a AccountInfo<'a>,
    vault_token_account: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
    drift_oracle: &'a AccountInfo<'a>,
    drift_spot_market: &'a AccountInfo<'a>,
    drift_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    msg!("Transfering from Vault Pda to Drift Vault...");

    let accounts_meta = vec![
        AccountMeta {
            pubkey: *drift_state.key,
            is_signer: false,
            is_writable: false,
        },
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
            pubkey: *vault_account.key,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_spot_market_vault.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *vault_token_account.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *token_program.key,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *drift_oracle.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_spot_market.key,
            is_signer: false,
            is_writable: true,
        },
    ];

    let args = DepositIxArgs {
        market_index: vault.spot_market_index,
        amount,
        reduce_only: false,
    };

    let data: DepositIxData = args.into();

    let ix = Instruction {
        program_id: *drift_program.key,
        accounts: accounts_meta,
        data: data.try_to_vec()?,
    };

    invoke_signed(
        &ix,
        &[
            drift_state.clone(),
            drift_user.clone(),
            drift_user_stats.clone(),
            vault_account.clone(),
            drift_spot_market_vault.clone(),
            vault_token_account.clone(),
            token_program.clone(),
            drift_oracle.clone(),
            drift_spot_market.clone(),
            drift_program.clone(),
        ],
        &[&[b"vault", name.as_ref(), &[vault_bump_seed]]],
    )
}

/// Resizes the vault depositor account to accommodate new deposits
///
/// # Arguments
/// * `vault_depositor_account` - Account to resize
/// * `authority` - Authority that pays for reallocation
/// * `system_program` - System program for CPI
/// * `additional_items` - Number of additional DepositInfo items to accommodate
fn resize_vault_depositor_account<'a>(
    vault_depositor_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    additional_items: usize,
) -> ProgramResult {
    msg!("Resizing vault depositor account...");

    // Calculate new size
    let current_size = vault_depositor_account.data.borrow().len();
    let new_size = current_size + DepositInfo::SIZE * additional_items;

    msg!("Current size: {}, New size: {}", current_size, new_size);

    // Calculate rent
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);
    let lamports_diff = new_minimum_balance.saturating_sub(vault_depositor_account.lamports());

    // Transfer additional rent if needed
    if lamports_diff > 0 {
        msg!("Transferring {} lamports for realloc rent", lamports_diff);
        invoke(
            &system_instruction::transfer(
                authority.key,
                vault_depositor_account.key,
                lamports_diff,
            ),
            &[
                authority.clone(),
                vault_depositor_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    // Reallocate account
    vault_depositor_account.realloc(new_size, false)?;

    msg!("Successfully resized account to {} bytes", new_size);
    Ok(())
}
