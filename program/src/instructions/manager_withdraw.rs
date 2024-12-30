use std::collections::BTreeSet;

use drift::{
    instructions::optional_accounts::{load_maps, AccountMaps},
    state::{spot_market_map::get_writable_spot_market_set, user::User},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

use crate::{
    common::{deserialize_zero_copy, log_accounts, transfer_to_user},
    error::wrap_drift_error,
    instructions::drift_withdraw,
    state::Vault,
};

pub fn manager_withdraw<'info>(
    _program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    amount: u64,
) -> ProgramResult {
    msg!("Starting manager withdraw...");
    msg!("amount: {}", amount);

    let clock = &Clock::get()?;

    let mut account_info_iter = &mut accounts.iter();

    let manager = next_account_info(&mut account_info_iter)?;
    let vault_account = next_account_info(&mut account_info_iter)?;

    let drift_program = next_account_info(&mut account_info_iter)?;
    let drift_user = next_account_info(&mut account_info_iter)?;
    let drift_user_stats = next_account_info(&mut account_info_iter)?;
    let drift_state = next_account_info(&mut account_info_iter)?;
    let drift_spot_market_vault = next_account_info(&mut account_info_iter)?;
    let drift_oracle = next_account_info(&mut account_info_iter)?;
    let drift_spot_market = next_account_info(&mut account_info_iter)?;
    let drift_signer = next_account_info(&mut account_info_iter)?;

    let manager_token_account = next_account_info(&mut account_info_iter)?;
    let vault_token_account = next_account_info(&mut account_info_iter)?;
    let mint = next_account_info(&mut account_info_iter)?;

    let token_program = next_account_info(&mut account_info_iter)?;

    log_accounts(&[
        (manager, "Manager"),
        (vault_account, "Vault Account"),
        // Drift accounts
        (drift_program, "Drift Program"),
        (drift_user, "Drift User"),
        (drift_user_stats, "Drift User Stats"),
        (drift_state, "Drift State"),
        (drift_spot_market_vault, "Drift Spot Market Vault"),
        (drift_oracle, "Drift Oracle"),
        (drift_spot_market, "Drift Spot Market"),
        // Token accounts
        (manager_token_account, "Manager Token Account"),
        (vault_token_account, "Vault Token Account"),
        (mint, "Mint"),
        // System accounts
        (token_program, "Token Program"),
    ]);

    if !manager.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if drift::ID != *drift_program.key {
        msg!("Invalid Drift Program");
        return Err(ProgramError::InvalidArgument);
    }

    let mut vault = Vault::get(vault_account);

    let writable_spot_market_set = get_writable_spot_market_set(vault.spot_market_index);

    // Load maps with proper account references and types
    let AccountMaps {
        perp_market_map,
        spot_market_map,
        mut oracle_map,
    } = match load_maps(
        &mut accounts[7..9].iter().peekable(),
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

    vault.manager_withdraw(amount, vault_equity, clock.unix_timestamp)?;

    Vault::save(&vault, vault_account)?;

    drift_withdraw(
        &vault,
        amount,
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

    transfer_to_user(
        &vault,
        amount,
        token_program,
        manager_token_account,
        vault_token_account,
        vault_account,
        mint,
    )?;

    Ok(())
}
