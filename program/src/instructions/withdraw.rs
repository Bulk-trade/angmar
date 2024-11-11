use crate::error::VaultError;
use crate::state::UserInfoAccountState;
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_10::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_id: String,
    user_pubkey: String,
    mut amount: u64,
    fund_status: String,
    bot_status: String,
) -> ProgramResult {
    msg!("Starting withdraw...");
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

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (user_pda, _user_bump_seed) = Pubkey::find_program_address(
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

    msg!("checking if user account is initialized");

    msg!("unpacking state account");
    let account_data_result =
        try_from_slice_unchecked::<UserInfoAccountState>(&user_info_pda_account.data.borrow());

    let account_data = match account_data_result {
        Ok(data) => {
            msg!("user pubkey: {}", data.user_pubkey);

            msg!("Account is already initialized");

            msg!("Checking Balance before withdraw");
            if data.amount < amount {
                msg!(
                    "Error: Withdrawal amount ({}) is greater than the available balance ({})",
                    amount,
                    data.amount
                );
                return Err(ProgramError::InsufficientFunds);
            }

            msg!("Updating the acccount");

            msg!("UserInfo before update:");
            msg!("vault_id: {}", data.vault_id);
            msg!("user_pubkey: {}", data.user_pubkey);
            msg!("amount: {}", data.amount);
            msg!("fund_status: {}", data.fund_status);
            msg!("bot_status: {}", data.bot_status);

            let mut updated_data = data;
            updated_data.vault_id = vault_id.clone();
            updated_data.amount -= amount;
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
            
            return Err(VaultError::UninitializedAccount.into());
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
    **vault_pda_account.lamports.borrow_mut() -= fees;
    **treasury_pda_account.lamports.borrow_mut() += fees;

    let (vault_pda, _vault_bump_seed) = Pubkey::find_program_address(
        &[vault_id.as_bytes().as_ref()],
        program_id,
    );

    msg!("Vault PDA: {}", vault_pda);

    if vault_pda != *vault_pda_account.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Withdrawing from Vault Pda to {}", initializer.key);

    **vault_pda_account.lamports.borrow_mut() -= amount;
    **initializer.lamports.borrow_mut() += amount;

    Ok(())
}
