use crate::drift::{DepositIxArgs, DepositIxData};
use crate::error::VaultError;
use crate::state::UserInfoAccountState;
use borsh::BorshSerialize;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_10::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
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
    market_index: u16,
) -> ProgramResult {
    msg!("Starting deposit...");
    msg!("vault_id: {}", vault_id);
    msg!("user_pubkey: {}", user_pubkey);
    msg!("amount: {}", amount);
    msg!("fund_status: {}", fund_status);
    msg!("bot_status: {}", bot_status);
    msg!("market_index: {}", market_index);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_info = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let treasury = next_account_info(account_info_iter)?;

    let drift_program = next_account_info(account_info_iter)?;
    let drift_user = next_account_info(account_info_iter)?;
    let drift_user_stats = next_account_info(account_info_iter)?;
    let drift_state = next_account_info(account_info_iter)?;
    let drift_spot_market_vault = next_account_info(account_info_iter)?;
    let drift_oracle = next_account_info(account_info_iter)?;
    let drift_spot_market = next_account_info(account_info_iter)?;

    let user_token_account = next_account_info(account_info_iter)?;
    let vault_token_account = next_account_info(account_info_iter)?;
    let treasury_token_account = next_account_info(account_info_iter)?;
    let mint = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // First batch - Main accounts
    msg!("1. initializer: {}", initializer.key);
    msg!("2. user_info: {}", user_info.key);
    msg!("3. vault: {}", vault.key);
    msg!("4. treasury: {}", treasury.key);

    // Second batch - Drift accounts
    msg!("5. drift_program: {}", drift_program.key);
    msg!("6. drift_user: {}", drift_user.key);
    msg!("7. drift_user_stats: {}", drift_user_stats.key);
    msg!("8. drift_state: {}", drift_state.key);
    msg!(
        "9. drift_spot_market_vault: {}",
        drift_spot_market_vault.key
    );
    msg!("10. drift_oracle: {}", drift_oracle.key);
    msg!("11. drift_spot_market: {}", drift_spot_market.key);

    // Third batch - Token accounts
    msg!("12. user_token_account: {}", user_token_account.key);
    msg!("13. vault_token_account: {}", vault_token_account.key);
    msg!("14. treasury_token_account: {}", treasury_token_account.key);
    msg!("15. mint: {}", mint.key);

    // Fourth batch - System accounts
    msg!("16. token_program: {}", token_program.key);
    msg!("17. system_program: {}", system_program.key);

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (user_pda, user_bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), user_pubkey.as_bytes().as_ref()],
        program_id,
    );

    if user_pda != *user_info.key {
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

    const FEE_PERCENTAGE: u64 = 2;
    let fees = (amount * FEE_PERCENTAGE + 99) / 100;
    amount -= fees;

    msg!("checking if user account is initialized");

    msg!("unpacking state account");
    let account_data_result =
        try_from_slice_unchecked::<UserInfoAccountState>(&user_info.data.borrow());

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
                    user_info.key,
                    rent_lamports,
                    account_len.try_into().unwrap(),
                    program_id,
                ),
                &[
                    initializer.clone(),
                    user_info.clone(),
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
            let mut data =
                try_from_slice_unchecked::<UserInfoAccountState>(&user_info.data.borrow()).unwrap();
            msg!("borrowed new account data");

            msg!("checking if user account is already initialized");
            if data.is_initialized {
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
    account_data.serialize(&mut &mut user_info.data.borrow_mut()[..])?;
    msg!("state account serialized");

    let (treasury_pda, _treasury_bump_seed) =
        Pubkey::find_program_address(&[b"treasury", vault_id.as_bytes().as_ref()], program_id);

    msg!("Treasury PDA: {}", treasury_pda);

    if treasury_pda != *treasury.key {
        msg!("Invalid seeds for Treasury PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Depositing Fees to Treasury Pda...");
    invoke(
        &instruction::transfer(
            &token_program.key,
            &user_token_account.key,
            &treasury_token_account.key,
            &initializer.key,
            &[initializer.key],
            fees,
        )?,
        &[
            mint.clone(),
            user_token_account.clone(),
            treasury_token_account.clone(),
            initializer.clone(),
            token_program.clone(),
        ],
    )?;

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[vault_id.as_bytes().as_ref()], program_id);

    msg!("Vault PDA: {}", vault_pda);

    if vault_pda != *vault.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Transfering to Vault Pda...");
    invoke(
        &instruction::transfer(
            &token_program.key,
            &user_token_account.key,
            &vault_token_account.key,
            &initializer.key,
            &[initializer.key],
            amount,
        )?,
        &[
            mint.clone(),
            user_token_account.clone(),
            vault_token_account.clone(),
            initializer.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("Transfering from Vault Pda to Drift Vault...");
    // Drift Deposit Cpi
    let accounts_meta = vec![
        AccountMeta {
            pubkey: *drift_state.key,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *drift_user.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_user_stats.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *vault.key,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_spot_market_vault.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *vault_token_account.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *token_program.key,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *drift_spot_market.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_oracle.key,
            is_signer: false,
            is_writable: true,
        },
    ];

    let args = DepositIxArgs {
        market_index,
        amount,
        reduce_only: false,
    };

    let data: DepositIxData = args.into();

    let ix = Instruction {
        program_id: *drift_program.key,
        accounts: accounts_meta,
        data: data.try_to_vec()?,
    };

    invoke_signed(
        &ix,
        &[
            drift_state.clone(),
            drift_user.clone(),
            drift_user_stats.clone(),
            vault.clone(),
            drift_spot_market_vault.clone(),
            vault_token_account.clone(),
            token_program.clone(),
            drift_spot_market.clone(),
            drift_oracle.clone(),
            drift_program.clone(),
        ],
        &[&[vault_id.as_bytes().as_ref(), &[vault_bump_seed]]],
    )?;

    Ok(())
}
