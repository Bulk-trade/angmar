use crate::instruction::VaultInstruction;
use crate::state::UserInfoAccountState;
use crate::{error::InfoError, state::VaultAccountState};
use borsh::BorshSerialize;
use solana_program::program::invoke;
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
        VaultInstruction::InitializeVault { vault_id } => {
            initialize_vault(program_id, accounts, vault_id)
        }
        VaultInstruction::Deposit {
            vault_id,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
        } => deposit(
            program_id,
            accounts,
            vault_id,
            user_pubkey,
            amount,
            fund_status,
            bot_status,
        ),
        VaultInstruction::UpdateUserInfo {
            vault_id,
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

pub fn initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
) -> ProgramResult {
    msg!("Initializing Vault...");
    msg!("Vault Id: {}", vault_id);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let vault_pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_pda, vault_bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), vault_id.as_bytes().as_ref()],
        program_id,
    );

    if vault_pda != *vault_pda_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let total_len: usize = 1 + 4 + (4 + vault_id.len());
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
            vault_pda_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            vault_pda_account.clone(),
            system_program.clone(),
        ],
        &[&[
            initializer.key.as_ref(),
            vault_id.as_bytes().as_ref(),
            &[vault_bump_seed],
        ]],
    )?;

    msg!("PDA created: {}", vault_pda);

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<VaultAccountState>(&vault_pda_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if user account is already initialized");
    if account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.vault_id = vault_id.clone();
    account_data.is_initialized = true;

    msg!("serializing account");
    account_data.serialize(&mut &mut vault_pda_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
}

pub fn deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
    user_pubkey: String,
    amount: f32,
    fund_status: String,
    bot_status: String,
) -> ProgramResult {
    msg!("Starting deposit...");
    msg!("vault_id: {}", vault_id);
    msg!("user_pubkey: {}", user_pubkey);
    msg!("amount: {}", amount);
    msg!("fund_status: {}", fund_status);
    msg!("bot_status: {}", bot_status);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_info_pda_account = next_account_info(account_info_iter)?;
    let vault_pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // if user_info_pda_account.owner != program_id {
    //     return Err(ProgramError::IllegalOwner);
    // }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (user_pda, user_bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), user_pubkey.as_bytes().as_ref()],
        program_id,
    );

    if user_pda != *user_info_pda_account.key {
        msg!("Invalid seeds for User PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let total_len: usize =
        1 + 4 + (4 + user_pubkey.len()) + (4 + fund_status.len()) + (4 + fund_status.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(InfoError::InvalidDataLength.into());
    }

    let account_len: usize = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    msg!("checking if user account is initialized");

    msg!("unpacking state account");
    let account_data_result =
        try_from_slice_unchecked::<UserInfoAccountState>(&user_info_pda_account.data.borrow());

    let account_data = match account_data_result {
        Ok(data) => {
            msg!("user pubkey: {}", data.user_pubkey);

            msg!("Account is already initialized");
            msg!("Updating the acccount");

            msg!("UserInfo before update:");
            msg!("vault_id: {}", data.vault_id);
            msg!("user_pubkey: {}", data.user_pubkey);
            msg!("amount: {}", data.amount);
            msg!("fund_status: {}", data.fund_status);
            msg!("bot_status: {}", data.bot_status);

            let mut updated_data = data;
            updated_data.vault_id = vault_id.clone();
            updated_data.amount += amount;
            updated_data.fund_status = fund_status;
            updated_data.bot_status = bot_status;

            msg!("UserInfo after update:");
            msg!("vault_id: {}", updated_data.vault_id);
            msg!("user_pubkey: {}", updated_data.user_pubkey);
            msg!("amount: {}", updated_data.amount);
            msg!("fund_status: {}", updated_data.fund_status);
            msg!("bot_status: {}", updated_data.bot_status);

            updated_data
        }
        Err(e) => {
            msg!("Error unpacking account data: {:?}", e);
            msg!("Account is not initialized");
            msg!("Creating the acccount");
            invoke_signed(
                &system_instruction::create_account(
                    initializer.key,
                    user_info_pda_account.key,
                    rent_lamports,
                    account_len.try_into().unwrap(),
                    program_id,
                ),
                &[
                    initializer.clone(),
                    user_info_pda_account.clone(),
                    system_program.clone(),
                ],
                &[&[
                    initializer.key.as_ref(),
                    user_pubkey.as_bytes().as_ref(),
                    &[user_bump_seed],
                ]],
            )?;

            msg!("new User PDA created: {}", user_pda);

            msg!("unpacking new state account");
            let mut data = try_from_slice_unchecked::<UserInfoAccountState>(
                &user_info_pda_account.data.borrow(),
            )
            .unwrap();
            msg!("borrowed new account data");

            msg!("checking if user account is already initialized");
            if data.is_initialized() {
                msg!("Account already initialized");
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            data.vault_id = vault_id.clone();
            data.user_pubkey = user_pubkey.clone();
            data.amount = amount;
            data.fund_status = fund_status;
            data.bot_status = bot_status;
            data.is_initialized = true;

            msg!("New UserInfo: ");
            msg!("vault_id: {}", data.vault_id);
            msg!("user_pubkey: {}", data.user_pubkey);
            msg!("amount: {}", data.amount);
            msg!("fund_status: {}", data.fund_status);
            msg!("bot_status: {}", data.bot_status);

            data
        }
    };

    msg!("serializing account");
    account_data.serialize(&mut &mut user_info_pda_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    //drift_interface::ID;

    let (vault_pda, vault_bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), vault_id.as_bytes().as_ref()],
        program_id,
    );

    msg!("Vault PDA: {}", vault_pda);

    if vault_pda != *vault_pda_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Depositing to Vault Pda...");
    invoke(
        &system_instruction::transfer(
            initializer.key,
            vault_pda_account.key,
            (amount * 1_000_000_000.0) as u64,
        ),
        &[
            initializer.clone(),
            vault_pda_account.clone(),
            system_program.clone(),
        ]
    )?;

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
        try_from_slice_unchecked::<UserInfoAccountState>(&pda_account.data.borrow()).unwrap();
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
