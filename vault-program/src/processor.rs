// processor.rs
use crate::{
    instruction::VaultInstruction,
    state::{VaultState, UserAccount, VAULT_SEED},
    error::VaultError,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = VaultInstruction::try_from_slice(instruction_data)?;
        match instruction {
            VaultInstruction::InitializeVault => Self::process_initialize_vault(accounts, program_id),
            VaultInstruction::Deposit { amount } => Self::process_deposit(accounts, amount, program_id),
            VaultInstruction::Withdraw { amount } => Self::process_withdraw(accounts, amount, program_id),
        }
    }

    fn process_initialize_vault(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_account = next_account_info(account_info_iter)?;
        let vault_account = next_account_info(account_info_iter)?;

        // Verify that the vault account is owned by the program
        let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[VAULT_SEED], program_id);
        if vault_pda != *vault_account.key {
            return Err(VaultError::InvalidVaultAccount.into());
        }

        // Initialize the vault account data
        let mut vault_data = VaultState::try_from_slice(&vault_account.data.borrow())?;
        vault_data.admin = *admin_account.key;
        vault_data.total_deposits = 0;
        vault_data.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn process_deposit(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;
        let user_vault_account = next_account_info(account_info_iter)?;
        let vault_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        // Verify that the user is signer
        if !user_account.is_signer {
            return Err(VaultError::SignatureMissing.into());
        }

        // Derive the user vault account PDA
        let (user_vault_pda, user_vault_bump) = Pubkey::find_program_address(
            &[b"user_account", user_account.key.as_ref()],
            program_id,
        );
        if user_vault_pda != *user_vault_account.key {
            return Err(VaultError::InvalidUserAccount.into());
        }

        // Transfer lamports from user to vault
        invoke(
            &system_instruction::transfer(user_account.key, vault_account.key, amount),
            &[
                user_account.clone(),
                vault_account.clone(),
                system_program.clone(),
            ],
        )?;

        // Update user's balance
        let mut user_data = if user_vault_account.data_len() == 0 {
            // Create user account data
            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(std::mem::size_of::<UserAccount>());
            invoke_signed(
                &system_instruction::create_account(
                    user_account.key,
                    user_vault_account.key,
                    required_lamports,
                    std::mem::size_of::<UserAccount>() as u64,
                    program_id,
                ),
                &[
                    user_account.clone(),
                    user_vault_account.clone(),
                    system_program.clone(),
                ],
                &[&[b"user_account", user_account.key.as_ref(), &[user_vault_bump]]],
            )?;
            UserAccount {
                owner: *user_account.key,
                balance: 0,
            }
        } else {
            UserAccount::try_from_slice(&user_vault_account.data.borrow())?
        };

        user_data.balance += amount;
        user_data.serialize(&mut &mut user_vault_account.data.borrow_mut()[..])?;

        // Update vault's total deposits
        let mut vault_data = VaultState::try_from_slice(&vault_account.data.borrow())?;
        vault_data.total_deposits += amount;
        vault_data.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn process_withdraw(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;
        let user_vault_account = next_account_info(account_info_iter)?;
        let vault_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        // Verify that the user is signer
        if !user_account.is_signer {
            return Err(VaultError::SignatureMissing.into());
        }

        // Derive the user vault account PDA
        let (user_vault_pda, user_vault_bump) = Pubkey::find_program_address(
            &[b"user_account", user_account.key.as_ref()],
            program_id,
        );
        if user_vault_pda != *user_vault_account.key {
            return Err(VaultError::InvalidUserAccount.into());
        }

        // Verify user's balance
        let mut user_data = UserAccount::try_from_slice(&user_vault_account.data.borrow())?;
        if user_data.balance < amount {
            return Err(VaultError::InsufficientFunds.into());
        }

        // Transfer lamports from vault to user
        invoke(
            &system_instruction::transfer(vault_account.key, user_account.key, amount),
            &[
                vault_account.clone(),
                user_account.clone(),
                system_program.clone(),
            ],
        )?;

        // Update user's balance
        user_data.balance -= amount;
        user_data.serialize(&mut &mut user_vault_account.data.borrow_mut()[..])?;

        // Update vault's total deposits
        let mut vault_data = VaultState::try_from_slice(&vault_account.data.borrow())?;
        vault_data.total_deposits -= amount;
        vault_data.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

        Ok(())
    }
}
