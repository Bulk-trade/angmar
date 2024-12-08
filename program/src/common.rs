use anchor_lang::{Owner, ZeroCopy};
use arrayref::array_ref;
use bytemuck::from_bytes;

pub const PERCENTAGE_PRECISION: u64 = 1_000_000;

pub fn deserialize_zero_copy<T: ZeroCopy + Owner>(account_data: &[u8]) -> Box<T> {
    let disc_bytes = array_ref![account_data, 0, 8];
    assert_eq!(disc_bytes, &T::discriminator());
    Box::new(*from_bytes::<T>(
        &account_data[8..std::mem::size_of::<T>() + 8],
    ))
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