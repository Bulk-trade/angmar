use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::borsh0_10::try_from_slice_unchecked;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Treasury {
    /// The treasury's pubkey.
    pub pubkey: Pubkey,
    /// The manager of the treasury who has ability to update vault params
    pub manager: Pubkey,
    /// The vault's pubkey. It is a pda of name and also used as the authority for drift user
    pub vault: Pubkey,
    /// The treasury token account. Used to receive tokens for deposits and withdrawals fees
    pub token_account: Pubkey,
    /// The bump for the treasury pda
    pub bump: u8,
}

impl Treasury {
    pub const SIZE: usize = std::mem::size_of::<Treasury>() + 8;

    pub fn get_treasury_signer_seeds<'a>(name: &'a str, bump: &'a [u8]) -> [&'a [u8]; 3] {
        [b"treasury", name.as_bytes(), bump]
    }

    pub fn get_pda<'a>(name: &String, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"treasury", name.as_bytes()], program_id)
    }

    pub fn get(account: &AccountInfo) -> Self {
        try_from_slice_unchecked::<Treasury>(&account.data.borrow()).unwrap()
    }

    pub fn save(treasury: &Treasury, account: &AccountInfo) {
        let _ = treasury.serialize(&mut &mut account.data.borrow_mut()[..]);
    }
}
