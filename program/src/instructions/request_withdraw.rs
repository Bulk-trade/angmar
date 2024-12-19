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
    common::{bytes32_to_string, deserialize_zero_copy, log_accounts},
    error::wrap_drift_error,
    state::{Vault, VaultDepositor},
};

pub fn request_withdraw<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    withdraw_amount: u64,
) -> ProgramResult {
    let clock = &Clock::get()?;

    let mut iter = accounts.iter();

    let vault_account = next_account_info(&mut iter)?;
    let vault_depositor_account = next_account_info(&mut iter)?;
    let authority = next_account_info(&mut iter)?;

    let drift_user = next_account_info(&mut iter)?;
    let drift_user_stats = next_account_info(&mut iter)?;
    let drift_state = next_account_info(&mut iter)?;
    let drift_oracle = next_account_info(&mut iter)?;
    let drift_spot_market = next_account_info(&mut iter)?;

    log_accounts(&[
        (vault_account, "Vault"),
        (vault_depositor_account, "Vault Depositor"),
        (authority, "Authority"),
        // Drift accounts
        (drift_user, "Drift User"),
        (drift_user_stats, "Drift User Stats"),
        (drift_state, "Drift State"),
        (drift_oracle, "Drift Oracle"),
        (drift_spot_market, "Drift Spot Market"),
    ]);

    if !authority.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
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
        &mut accounts[6..8].iter().peekable(),
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

    let vault_equity =
        vault.calculate_total_equity(&user, &perp_market_map, &spot_market_map, &mut oracle_map).map_err(wrap_drift_error)?;

    msg!("vault_equity: {:?}", vault_equity);

    vault_depositor.request_withdraw(
        withdraw_amount,
        vault_equity,
        &mut vault,
        clock.unix_timestamp,
    )?;

    Vault::save(&vault, vault_account)?;
    VaultDepositor::save(&vault_depositor, vault_depositor_account)?;

    Ok(())
}
