use std::collections::BTreeSet;

use crate::{
    common::{
        bytes32_to_string, deserialize_zero_copy, log_accounts, transfer_fees_from_vault,
        transfer_to_user,
    },
    drift::{WithdrawIxArgs, WithdrawIxData},
    error::wrap_drift_error,
    state::{Vault, VaultDepositor},
};
use drift::{
    instructions::optional_accounts::{load_maps, AccountMaps},
    state::{spot_market_map::get_writable_spot_market_set, user::User},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

pub fn withdraw<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
) -> ProgramResult {
    let clock = &Clock::get()?;

    let mut iter = accounts.iter();

    let vault_account = next_account_info(&mut iter)?;
    let vault_depositor_account = next_account_info(&mut iter)?;
    let authority = next_account_info(&mut iter)?;
    let treasury = next_account_info(&mut iter)?;

    let drift_program = next_account_info(&mut iter)?;
    let drift_user = next_account_info(&mut iter)?;
    let drift_user_stats = next_account_info(&mut iter)?;
    let drift_state = next_account_info(&mut iter)?;
    let drift_spot_market_vault = next_account_info(&mut iter)?;
    let drift_oracle = next_account_info(&mut iter)?;
    let drift_spot_market = next_account_info(&mut iter)?;
    let drift_signer = next_account_info(&mut iter)?;

    let user_token_account = next_account_info(&mut iter)?;
    let vault_token_account = next_account_info(&mut iter)?;
    let treasury_token_account = next_account_info(&mut iter)?;
    let mint = next_account_info(&mut iter)?;

    let token_program = next_account_info(&mut iter)?;

    log_accounts(&[
        // Vault accounts
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
        (drift_signer, "Drift Signer"),
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

    let mut vault = Vault::get(vault_account);
    let mut vault_depositor = VaultDepositor::get(vault_depositor_account);

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
    msg!("User Details:");
    msg!("  Authority: {}", user.authority);
    msg!("  Name: {}", bytes32_to_string(user.name));

    let vault_equity = vault
        .calculate_total_equity(&user, &perp_market_map, &spot_market_map, &mut oracle_map)
        .map_err(wrap_drift_error)?;

    msg!("vault_equity: {:?}", vault_equity);

    let (user_withdraw_amount, total_deductions) =
        vault_depositor.withdraw(vault_equity, &mut vault, clock.unix_timestamp)?;

    msg!("user_withdraw_amount: {}", user_withdraw_amount);

    Vault::save(&vault, vault_account)?;
    VaultDepositor::save(&vault_depositor, vault_depositor_account)?;

    drift_withdraw(
        &vault,
        user_withdraw_amount + total_deductions,
        drift_program,
        drift_state,
        drift_user,
        drift_user_stats,
        vault_account,
        drift_spot_market_vault,
        drift_signer,
        vault_token_account,
        token_program,
        drift_oracle,
        drift_spot_market,
    )?;

    transfer_fees_from_vault(
        &vault,
        total_deductions,
        token_program,
        user_token_account,
        treasury_token_account,
        authority,
        mint,
    )?;

    transfer_to_user(
        &vault,
        user_withdraw_amount,
        token_program,
        user_token_account,
        vault_token_account,
        vault_account,
        mint,
    )?;

    Ok(())
}

/// Executes withdraw from Drift protocol
fn drift_withdraw<'a>(
    vault: &Vault,
    amount: u64,
    // Individual accounts
    drift_program: &AccountInfo<'a>,
    drift_state: &AccountInfo<'a>,
    drift_user: &AccountInfo<'a>,
    drift_user_stats: &AccountInfo<'a>,
    vault_account: &AccountInfo<'a>,
    drift_spot_market_vault: &AccountInfo<'a>,
    drift_signer: &AccountInfo<'a>,
    vault_token_account: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    drift_oracle: &AccountInfo<'a>,
    drift_spot_market: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("Withdrawing from Drift to Vault...");

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
            pubkey: *drift_signer.key,
            is_signer: false,
            is_writable: false,
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

    let args = WithdrawIxArgs {
        market_index: vault.spot_market_index,
        amount,
        reduce_only: false,
    };

    let data: WithdrawIxData = args.into();

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
            drift_signer.clone(),
            vault_token_account.clone(),
            token_program.clone(),
            drift_oracle.clone(),
            drift_spot_market.clone(),
            drift_program.clone(),
        ],
        &[&[
            b"vault",
            bytes32_to_string(vault.name).as_ref(),
            &[vault.bump],
        ]],
    )
}
