use std::collections::BTreeSet;

use crate::common::{bytes32_to_string, deserialize_zero_copy, PERCENTAGE_PRECISION};
use crate::drift::{DepositIxArgs, DepositIxData};
use crate::error::VaultError;
use crate::state::{Vault, VaultDepositor};
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
use std::slice;

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

    // First batch - Main accounts
    msg!("vault: {}", vault_account.key);
    msg!("vault_depositor: {}", vault_depositor_account.key);
    msg!("authority: {}", authority.key);
    msg!("treasury: {}", treasury.key);

    // Second batch - Drift accounts
    msg!("drift_program: {}", drift_program.key);
    msg!("drift_user: {}", drift_user.key);
    msg!("drift_user_stats: {}", drift_user_stats.key);
    msg!("drift_state: {}", drift_state.key);
    msg!("drift_spot_market_vault: {}", drift_spot_market_vault.key);
    msg!("drift_oracle: {}", drift_oracle.key);
    msg!("drift_spot_market: {}", drift_spot_market.key);

    // Third batch - Token accounts
    msg!("user_token_account: {}", user_token_account.key);
    msg!("vault_token_account: {}", vault_token_account.key);
    msg!("treasury_token_account: {}", treasury_token_account.key);
    msg!("mint: {}", mint.key);

    // Fourth batch - System accounts
    msg!("token_program: {}", token_program.key);
    msg!("system_program: {}", system_program.key);

    if !authority.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_depositor_pda, _) = Pubkey::find_program_address(
        &[
            b"vault_depositor",
            vault_account.key.as_ref(),
            authority.key.as_ref(),
        ],
        program_id,
    );

    if vault_depositor_pda != *vault_depositor_account.key {
        msg!("Invalid seeds for Vault Depositor PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("unpacking vault state account");
    let mut vault = Vault::get(vault_account);

    // Print the specified fields from vault_data
    msg!("Name: {:?}", vault.name);
    msg!("Pubkey: {:?}", vault.pubkey);
    msg!("Manager: {:?}", vault.manager);
    msg!("User Stats: {:?}", vault.user_stats);
    msg!("User: {:?}", vault.user);
    msg!("Token Account: {:?}", vault.token_account);
    msg!("Spot Market Index: {:?}", vault.spot_market_index);
    msg!("Init Timestamp: {:?}", vault.init_ts);
    msg!("Min Deposit Amount: {:?}", vault.min_deposit_amount);
    msg!("Management Fee: {:?}", vault.management_fee);
    msg!("Profit Share: {:?}", vault.profit_share);
    msg!("Bump: {:?}", vault.bump);
    msg!("Permissioned: {:?}", vault.permissioned);

    {
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

        msg!("  oracle_map: {}", oracle_map.slot);
        // msg!("  perp_market_map: {}", perp_market_map.);
        // Print spot market map details with error handling
        let spot_market = spot_market_map
            .get_ref(&vault.spot_market_index)
            .map_err(|e| ProgramError::Custom(e as u32))?;
        msg!("  spot_market_map: {}", spot_market.get_precision());

        // User details
        let user = deserialize_zero_copy::<User>(&*drift_user.try_borrow_data()?);
        msg!("User Details:");
        msg!("  Authority: {}", user.authority);
        msg!("  Name: {}", bytes32_to_string(user.name));

        let vault_equity = vault
            .calculate_equity(&user, &perp_market_map, &spot_market_map, &mut oracle_map)
            .map_err(|e| ProgramError::Custom(e as u32))?;

        msg!("vault_equity: {:?}", vault_equity);
    }
    msg!("Getting Vault Depositor");
    let mut vault_depositor = VaultDepositor::get(vault_depositor_account);

    msg!("Vault Depositor Pubkey: {:?}", vault_depositor.pubkey);

    if amount < vault.min_deposit_amount {
        msg!("Deposit can't be less then {}", vault.min_deposit_amount);
        return Err(VaultError::InvalidInput.into());
    }

    let vault_fee = vault.management_fee;
    let fees = (amount * vault_fee + 99) / PERCENTAGE_PRECISION;
    amount -= fees;

    msg!("Fees: {}", fees);
    msg!("Amount after fees: {}", amount);

    msg!("Issuing user shares: {}", amount);
    vault.net_deposits += amount;
    vault.total_deposits += amount;
    vault.total_shares += amount as u128;

    Vault::save(&vault, vault_account);

    vault_depositor.vault_shares += amount as u128;
    vault_depositor.total_deposits += amount;
    vault_depositor.net_deposits += amount;

    VaultDepositor::save(vault_depositor, vault_depositor_account);

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
    )?;

    if vault.pubkey != *vault_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

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
    )?;

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[b"vault", name.as_ref()], program_id);

    if vault_pda != *vault_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Transfering from Vault Pda to Drift Vault...");
    // Drift Deposit Cpi
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
    )?;

    Ok(())
}
