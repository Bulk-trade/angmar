use crate::error::VaultError;
use crate::state::UserInfoAccountState;
use borsh::BorshSerialize;
use drift_interface::{deposit_ix, DepositAccounts, DepositIxArgs, DepositKeys};
use solana_program::program::invoke;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh1::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::instruction;
use std::convert::TryInto;

pub fn deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
    user_pubkey: String,
    mut amount: u64,
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
    let treasury_pda_account = next_account_info(account_info_iter)?;
    let drift_program = next_account_info(account_info_iter)?;
    let user = next_account_info(account_info_iter)?;
    let user_stats = next_account_info(account_info_iter)?;
    let state = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;
    let spot_market_vault = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

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
        return Err(VaultError::InvalidDataLength.into());
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

    let (treasury_pda, _treasury_bump_seed) =
        Pubkey::find_program_address(&[b"treasury", vault_id.as_bytes().as_ref()], program_id);

    msg!("Treasury PDA: {}", treasury_pda);

    if treasury_pda != *treasury_pda_account.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    const FEE_PERCENTAGE: u64 = 2;
    let fees = (amount * FEE_PERCENTAGE + 99) / 100;
    amount -= fees;

    msg!("Depositing Fees to Treasury Pda...");
    invoke(
        &system_instruction::transfer(initializer.key, treasury_pda_account.key, fees),
        &[
            initializer.clone(),
            treasury_pda_account.clone(),
            system_program.clone(),
        ],
    )?;

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[vault_id.as_bytes().as_ref()], program_id);

    msg!("Vault PDA: {}", vault_pda);

    if vault_pda != *vault_pda_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Depositing to Vault Pda...");
    // invoke(
    //     &system_instruction::transfer(
    //         initializer.key,
    //         vault_pda_account.key,
    //         (amount * 1_000_000_000.0) as u64,
    //     ),
    //     &[
    //         initializer.clone(),
    //         vault_pda_account.clone(),
    //         system_program.clone(),
    //     ],
    // )?;

    let ix = &instruction::transfer(
        token_program.key,
        user_token_account.key,
        vault_pda_account.key,
        initializer.key,
        &[initializer.key],
        amount,
    )?;

    invoke(
        &ix,
        &[
            initializer.clone(),
            vault_pda_account.clone(),
            system_program.clone(),
        ],
    )?;

    let accounts = DepositAccounts {
        state,
        user,
        user_stats,
        authority: vault_pda_account,
        spot_market_vault,
        user_token_account,
        token_program,
    };

    let keys = DepositKeys::from(accounts);
    let args = DepositIxArgs {
        market_index: 0,
        amount: amount,
        reduce_only: false,
    };

    let ix = deposit_ix(keys, args)?;

    invoke_signed(
        &ix,
        &[
            drift_program.clone(),
            state.clone(),
            user.clone(),
            user_stats.clone(),
            vault_pda_account.clone(),
            spot_market_vault.clone(),
            user_token_account.clone(),
            token_program.clone(),
        ],
        &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]]
    )?;

    Ok(())
}
