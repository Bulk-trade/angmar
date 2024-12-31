use serde::Serialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    common::{log_accounts, log_params},
    constants::{PERCENTAGE_PRECISION, PERCENTAGE_PRECISION_U64},
    error::VaultErrorCode,
    state::Vault,
};

pub fn update_vault<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    params: &UpdateVaultParams,
) -> ProgramResult {
    msg!("Updating vault...");
    log_params(&params);

    let account_info_iter = &mut accounts.iter();

    let manager = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;

    log_accounts(&[(manager, "Manager"), (vault_account, "Vault Account")]);

    if !manager.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut vault = Vault::get(vault_account);

    msg!("Before Updating vault...");
    log_params(&vault);

    if vault.manager != *manager.key {
        msg!("Invalid Vault Manager");
        return Err(ProgramError::InvalidArgument);
    }

    vault.lock_in_period = params.lock_in_period;
    vault.redeem_period = params.redeem_period;
    vault.max_tokens = params.max_tokens;

    if params.management_fee >= PERCENTAGE_PRECISION_U64 {
        msg!("management fee must be < 100%");
        return Err(VaultErrorCode::InvalidVaultInitialization.into());
    }

    vault.management_fee = params.management_fee;
    vault.min_deposit_amount = params.min_deposit_amount;

    if params.profit_share >= PERCENTAGE_PRECISION as u32 {
        msg!("profit share must be < 100%");
        return Err(VaultErrorCode::InvalidVaultInitialization.into());
    }
    vault.profit_share = params.profit_share;
    vault.permissioned = params.permissioned;

    Vault::save(&vault, vault_account)?;

    msg!("After Updating vault...");
    log_params(&vault);

    Ok(())
}

#[derive(Serialize)]
pub struct UpdateVaultParams {
    pub lock_in_period: u64,
    pub redeem_period: u64,
    pub max_tokens: u64,
    pub management_fee: u64,
    pub min_deposit_amount: u64,
    pub profit_share: u32,
    pub hurdle_rate: u32,
    pub permissioned: bool,
}
