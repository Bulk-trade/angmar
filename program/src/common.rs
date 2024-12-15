use anchor_lang::{Owner, ZeroCopy};
use arrayref::array_ref;
use bytemuck::from_bytes;
use serde::Serialize;
use serde_json::to_string;
use solana_program::{account_info::AccountInfo, log::sol_log_data, msg, program_error::ProgramError};
use borsh::BorshSerialize;

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
