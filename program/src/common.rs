use anchor_lang::{Owner, ZeroCopy};
use borsh::BorshDeserialize;
use bytemuck::from_bytes;
use drift::state::user::User;
use solana_program::{
    account_info::AccountInfo, borsh0_10::try_from_slice_unchecked, program_error::ProgramError,
};
use std::{
    cell::{Ref, RefMut},
    mem,
};
use arrayref::array_ref;

pub const PERCENTAGE_PRECISION: u64 = 1_000_000;

pub fn deserialize_zero_copy<T: ZeroCopy + Owner>(account_data: &[u8]) -> Box<T> {
    let disc_bytes = array_ref![account_data, 0, 8];
    assert_eq!(disc_bytes, &T::discriminator());
    Box::new(*from_bytes::<T>(
        &account_data[8..std::mem::size_of::<T>() + 8],
    ))
}

/// Common function to load and deserialize an account
pub fn drift_user_loader(account: &AccountInfo) -> Result<User, ProgramError> {
    // Borrow the account data
    let data = account.try_borrow_data()?;

    Ok(*Ref::map(data, |data| {
        bytemuck::from_bytes(&data[8..mem::size_of::<User>() + 8])
    }))

    // // Check if the data length is sufficient to contain the discriminator
    // if data.len() < expected_discriminator.len() {
    //     return Err(ProgramError::InvalidAccountData);
    // }

    // // Verify the discriminator
    // let disc_bytes = &data[0..8];
    // if disc_bytes != expected_discriminator {
    //     return Err(ProgramError::InvalidAccountData);
    // }

    // Deserialize the account data
    // let account: T = T::try_from_slice(&data[8..])?;
    // Ok(account)

    // Ok(try_from_slice_unchecked::<T>(&account_info.data.borrow()).unwrap())
}

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
// /// Common function to load and deserialize an account mutably
// pub fn account_loader_mut<'a, T: BorshDeserialize + Sized>(
//     account_info: &'a AccountInfo<'a>,
//     expected_discriminator: &[u8; 8],
// ) -> Result<RefMut<'a, T>, ProgramError> {
//     // Borrow the account data mutably
//     let data = account_info.try_borrow_mut_data()?;

//     // Check if the data length is sufficient to contain the discriminator
//     if data.len() < expected_discriminator.len() {
//         return Err(ProgramError::InvalidAccountData);
//     }

//     // Verify the discriminator
//     let disc_bytes = &data[0..8];
//     if disc_bytes != expected_discriminator {
//         return Err(ProgramError::InvalidAccountData);
//     }

//     // Deserialize the account data
//     Ok(RefMut::map(data, |data| {
//         bytemuck::from_bytes_mut(&mut data[8..std::mem::size_of::<T>() + 8])
//     }))
// }
