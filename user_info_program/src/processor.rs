use crate::error::InfoError;
use crate::instruction::VaultInstruction;
use crate::state::VaultAccountState;
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VaultInstruction::unpack(instruction_data)?;
    match instruction {
        VaultInstruction::AddUserInfo {
            user_pubkey,
            amount,
            fund_status,
            bot_status,
        } => add_user_info(
            program_id,
            accounts,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
        ),
        VaultInstruction::UpdateUserInfo {
            user_pubkey,
            amount,
            fund_status,
            bot_status,
        } => update_user_info(
            program_id,
            accounts,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
        ),
    }
}

pub fn add_user_info(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    user_pubkey: String,
    amount: f32,
    fund_status: String,
    bot_status: String,
) -> ProgramResult {
    msg!("Adding User info...");
    msg!("user_pubkey: {}", user_pubkey);
    msg!("amount: {}", amount);
    msg!("fund_status: {}", fund_status);
    msg!("bot_status: {}", bot_status);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), user_pubkey.as_bytes().as_ref()],
        program_id,
    );
    if pda != *pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // if rating > 5 || rating < 1 {
    //     msg!("Rating cannot be higher than 5");
    //     return Err(ReviewError::InvalidRating.into());
    // }

    let total_len: usize =
        1 + 4 + (4 + user_pubkey.len()) + (4 + fund_status.len()) + (4 + fund_status.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(InfoError::InvalidDataLength.into());
    }

    let account_len: usize = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[&[
            initializer.key.as_ref(),
            user_pubkey.as_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    msg!("PDA created: {}", pda);

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<VaultAccountState>(&pda_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if user account is already initialized");
    if account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.user_pubkey = user_pubkey;
    account_data.amount = amount;
    account_data.fund_status = fund_status;
    account_data.bot_status = bot_status;
    account_data.is_initialized = true;

    msg!("serializing account");
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    drift_interface::ID;


    Ok(())
}

pub fn update_user_info(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    user_pubkey: String,
    amount: f32,
    fund_status: String,
    bot_status: String,
) -> ProgramResult {
    msg!("Updating User info...");
    msg!("user_pubkey: {}", user_pubkey);
    msg!("amount: {}", amount);
    msg!("fund_status: {}", fund_status);
    msg!("bot_status: {}", bot_status);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;

    if pda_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<VaultAccountState>(&pda_account.data.borrow()).unwrap();
    msg!("user pubkey: {}", account_data.user_pubkey);

    let (pda, _bump_seed) = Pubkey::find_program_address(
        &[
            initializer.key.as_ref(),
            account_data.user_pubkey.as_bytes().as_ref(),
        ],
        program_id,
    );
    if pda != *pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(InfoError::InvalidPDA.into());
    }

    msg!("checking if user account is initialized");
    if !account_data.is_initialized() {
        msg!("Account is not initialized");
        return Err(InfoError::UninitializedAccount.into());
    }

    // if rating > 5 || rating < 1 {
    //     msg!("Invalid Rating");
    //     return Err(ReviewError::InvalidRating.into());
    // }

    let update_len: usize =
        1 + 4 + (4 + user_pubkey.len()) + (4 + fund_status.len()) + (4 + fund_status.len());
    if update_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(InfoError::InvalidDataLength.into());
    }

    msg!("UserInfo before update:");
    msg!("user_pubkey: {}", account_data.user_pubkey);
    msg!("amount: {}", account_data.amount);
    msg!("fund_status: {}", account_data.fund_status);
    msg!("bot_status: {}", account_data.bot_status);

    account_data.amount = amount;
    account_data.fund_status = fund_status;
    account_data.bot_status = bot_status;

    msg!("UserInfo after update:");
    msg!("user_pubkey: {}", account_data.user_pubkey);
    msg!("amount: {}", account_data.amount);
    msg!("fund_status: {}", account_data.fund_status);
    msg!("bot_status: {}", account_data.bot_status);

    msg!("serializing account");
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
}
