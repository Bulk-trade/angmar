use crate::common::{bytes32_to_string, deserialize_zero_copy, log_accounts, log_data, log_params};
use crate::drift::{DepositIxArgs, DepositIxData};
use crate::error::{wrap_drift_error, ErrorCode};
use crate::state::{Vault, VaultDepositor, VaultDepositorAction, VaultDepositorRecord};
use crate::validate;
use drift::instructions::optional_accounts::{load_maps, AccountMaps};
use drift::state::spot_market_map::get_writable_spot_market_set;
use drift::state::user::User;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::instruction;
use std::collections::BTreeSet;

pub fn deposit<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    name: String,
    mut amount: u64,
) -> ProgramResult {
    msg!("Starting deposit...");
    msg!("name: {}", name);
    msg!("amount: {}", amount);

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

    let user_token_account = next_account_info(&mut iter)?;
    let vault_token_account = next_account_info(&mut iter)?;
    let treasury_token_account = next_account_info(&mut iter)?;
    let mint = next_account_info(&mut iter)?;

    let token_program = next_account_info(&mut iter)?;
    let system_program = next_account_info(&mut iter)?;

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
        (system_program, "System Program"),
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

    msg!("unpacking vault state account");
    let mut vault = Vault::get(vault_account);

    // Create array of references to input accounts
    //let spot_markets = slice::from_ref(drift_oracle);
    // Directly slice the necessary multiple accounts example  &mut accounts[8..10].iter().peekable()
    //let mut remaining_accounts_iter = spot_markets.iter().peekable();
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

    msg!("Getting Vault Depositor");
    let mut vault_depositor = VaultDepositor::get(vault_depositor_account);

    msg!("Vault Depositor Pubkey: {:?}", vault_depositor.pubkey);

    validate!(
        vault.max_tokens == 0 || vault.max_tokens > vault_equity + amount,
        ErrorCode::VaultIsAtCapacity,
        "after deposit vault equity is {} > {}",
        vault_equity + amount,
        vault.max_tokens
    )?;

    validate!(
        vault.min_deposit_amount == 0 || amount >= vault.min_deposit_amount,
        ErrorCode::InvalidVaultDeposit,
        "deposit amount {} is below vault min_deposit_amount {}",
        amount,
        vault.min_deposit_amount
    )?;

    let vault_shares_before = vault_depositor.vault_shares;
    let total_vault_shares_before = vault.total_shares;
    let user_vault_shares_before = vault.user_shares;

    let management_fee = vault.management_fee;
    let fees = Vault::calculate_fees(amount, management_fee);
    amount -= fees;
    msg!("Fees: {}", fees);
    msg!("Deposit amount after fees: {}", amount);

    let new_shares = VaultDepositor::calculate_shares_for_deposit(
        amount,
        total_vault_shares_before,
        vault_equity,
    )?;
    msg!("Issuing user shares: {}", new_shares);

    vault_depositor.total_deposits += amount;
    vault_depositor.net_deposits += amount;
    vault_depositor.vault_shares += new_shares;

    VaultDepositor::save(&vault_depositor, vault_depositor_account)?;

    vault.manager_total_fee += fees;
    vault.total_deposits += amount;
    vault.net_deposits += amount;
    vault.user_shares += new_shares as u128;
    vault.total_shares += new_shares as u128;

    Vault::save(&vault, vault_account)?;

    msg!("Vault Deposit Record");
    let record = VaultDepositorRecord {
        ts: clock.unix_timestamp,
        vault: vault.pubkey,
        depositor_authority: *authority.key,
        action: VaultDepositorAction::Deposit,
        amount,
        spot_market_index: vault.spot_market_index,
        vault_equity_before: vault_equity,
        vault_shares_before,
        user_vault_shares_before,
        total_vault_shares_before,
        vault_shares_after: vault_depositor.vault_shares,
        total_vault_shares_after: vault.total_shares,
        user_vault_shares_after: vault.user_shares,
        profit_share: vault.profit_share,
        management_fee: fees,
        management_fee_shares: vault.management_fee,
    };

    log_data(&record)?;

    log_params(&record);

    transfer_fees(
        fees,
        token_program,
        user_token_account,
        treasury_token_account,
        authority,
        mint,
    )?;

    transfer_to_vault(
        amount,
        token_program,
        user_token_account,
        vault_token_account,
        authority,
        mint,
    )?;

    drift_deposit(
        &vault,
        amount,
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

fn transfer_fees<'a>(
    fees: u64,
    token_program: &AccountInfo<'a>,
    user_token_account: &AccountInfo<'a>,
    treasury_token_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("Depositing Fees to Treasury Pda...");
    invoke(
        &instruction::transfer(
            &token_program.key,
            &user_token_account.key,
            &treasury_token_account.key,
            &authority.key,
            &[authority.key],
            fees,
        )?,
        &[
            mint.clone(),
            user_token_account.clone(),
            treasury_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
    )
}

fn transfer_to_vault<'a>(
    amount: u64,
    token_program: &AccountInfo<'a>,
    user_token_account: &AccountInfo<'a>,
    vault_token_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("Transfering to Vault Pda...");
    invoke(
        &instruction::transfer(
            &token_program.key,
            &user_token_account.key,
            &vault_token_account.key,
            &authority.key,
            &[authority.key],
            amount,
        )?,
        &[
            mint.clone(),
            user_token_account.clone(),
            vault_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
    )
}

fn drift_deposit<'a>(
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
