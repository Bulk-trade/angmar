use anchor_lang::{Owner, ZeroCopy};
use arrayref::array_ref;
use borsh::BorshSerialize;
use bytemuck::from_bytes;
use drift::math::insurance::vault_amount_to_if_shares;
use serde::Serialize;
use serde_json::to_string;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    log::sol_log_data,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
};
use spl_token::instruction;

use crate::{error::wrap_drift_error, state::Vault};

/// Deserializes a zero-copy account from the given account data.
///
/// # Arguments
///
/// * `account_data` - A byte slice containing the account data.
///
/// # Returns
///
/// A boxed instance of the deserialized account.
pub fn deserialize_zero_copy<T: ZeroCopy + Owner>(account_data: &[u8]) -> Box<T> {
    let disc_bytes = array_ref![account_data, 0, 8];
    assert_eq!(disc_bytes, &T::discriminator());
    Box::new(*from_bytes::<T>(
        &account_data[8..std::mem::size_of::<T>() + 8],
    ))
}

/// Converts a string to a [u8; 32] array.
///
/// # Arguments
///
/// * `str` - The input string to be converted.
///
/// # Returns
///
/// A [u8; 32] array containing the string bytes.
pub fn string_to_bytes32(str: &str) -> [u8; 32] {
    let mut str_32 = [0u8; 32];
    let str_bytes = str.as_bytes();
    str_32[..str_bytes.len()].copy_from_slice(str_bytes);
    str_32
}

/// Converts a [u8; 32] array to a string.
///
/// # Arguments
///
/// * `bytes` - The input [u8; 32] array to be converted.
///
/// # Returns
///
/// A string representation of the byte array.
pub fn bytes32_to_string(bytes: [u8; 32]) -> String {
    String::from_utf8(
        bytes
            .iter()
            .take_while(|&&c| c != 0)
            .cloned()
            .collect::<Vec<u8>>(),
    )
    .unwrap_or_else(|_| String::from("Invalid UTF-8"))
}

/// Logs the fields and values of a struct in JSON format.
///
/// # Arguments
///
/// * `params` - A reference to the struct to be logged.
pub fn log_params<T: Serialize>(params: &T) {
    match to_string(params) {
        Ok(json) => msg!("{}", json),
        Err(e) => msg!("Failed to serialize params: {}", e),
    }
}

/// Logs account information to the program log
///
/// # Arguments
/// * `accounts` - Slice of tuples containing account info and account name
///
/// # Example
/// ```rust
/// log_accounts(&[
///     (vault_account, "Vault"),
///     (authority, "Authority")
/// ]);
/// ```
pub fn log_accounts(accounts: &[(&AccountInfo, &str)]) {
    for (account, name) in accounts {
        msg!("{}: {}", name, account.key);
    }
}

/// Logs serializable data to the program log using Solana's data logging syscall
///
/// # Arguments
/// * `record` - Any type that implements the Serialize trait
///
/// # Returns
/// * `Result<(), ProgramError>` - Success or serialization error
///
/// # Example
/// ```rust
/// let record = VaultDepositorRecord { /* fields */ };
/// log_data(&record)?;
/// ```
///
/// # Errors
/// Returns error if serialization fails
pub fn log_data<T: Serialize + BorshSerialize>(record: &T) -> Result<(), ProgramError> {
    sol_log_data(&[&record.try_to_vec()?]);
    Ok(())
}

pub fn calculate_amount_to_shares(
    amount: u64,
    total_vault_shares: u128,
    total_value_locked: u64,
) -> Result<u128, ProgramError> {
    let shares = vault_amount_to_if_shares(amount, total_vault_shares, total_value_locked)
        .map_err(wrap_drift_error)?;

    Ok(shares)
}

pub fn transfer_fees<'a>(
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

pub fn transfer_fees_from_vault<'a>(
    vault: &Vault,
    fees: u64,
    token_program: &AccountInfo<'a>,
    user_token_account: &AccountInfo<'a>,
    treasury_token_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("Depositing Fees to Treasury Pda...");
    invoke_signed(
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
        &[&[
            b"vault",
            bytes32_to_string(vault.name).as_ref(),
            &[vault.bump],
        ]],
    )
}

pub fn transfer_to_vault<'a>(
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

pub fn transfer_to_user_from_vault<'a>(
    vault: &Vault,
    amount: u64,
    token_program: &AccountInfo<'a>,
    destination_token_account: &AccountInfo<'a>,
    source_token_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("Transfering to User....");
    invoke_signed(
        &instruction::transfer(
            &token_program.key,
            &source_token_account.key,
            &destination_token_account.key,
            &authority.key,
            &[authority.key],
            amount,
        )?,
        &[
            mint.clone(),
            source_token_account.clone(),
            destination_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
        &[&[
            b"vault",
            bytes32_to_string(vault.name).as_ref(),
            &[vault.bump],
        ]],
    )
}

pub fn transfer_to_user_from_treasury<'a>(
    amount: u64,
    token_program: &AccountInfo<'a>,
    destination_token_account: &AccountInfo<'a>,
    source_token_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    signature_seeds: [&[u8]; 3],
) -> ProgramResult {
    msg!("Transfering to User....");

    invoke_signed(
        &instruction::transfer(
            &token_program.key,
            &source_token_account.key,
            &destination_token_account.key,
            &authority.key,
            &[authority.key],
            amount,
        )?,
        &[
            mint.clone(),
            source_token_account.clone(),
            destination_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
        &[&signature_seeds],
    )
}
