// src/processor.rs

use crate::{
    instruction::PoolInstruction,
    state::{PoolState, POOL_SEED},
    error::PoolError,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = PoolInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            PoolInstruction::InitializePool { vault } => {
                Self::process_initialize_pool(accounts, vault, program_id)
            }
            PoolInstruction::AllocateFunds { amount } => {
                Self::process_allocate_funds(accounts, amount, program_id)
            }
            // Handle additional instructions
        }
    }

    fn process_initialize_pool(
        accounts: &[AccountInfo],
        vault: Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let manager_account = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;

        if !manager_account.is_signer {
            return Err(PoolError::Unauthorized.into());
        }

        // Derive the Pool PDA
        let (pool_pda, _bump) = Pubkey::find_program_address(&[POOL_SEED], program_id);

        if pool_pda != *pool_account.key {
            return Err(PoolError::InvalidPoolAccount.into());
        }

        // Initialize the pool account data
        let mut pool_data = PoolState::try_from_slice(&pool_account.data.borrow())?;
        pool_data.manager = *manager_account.key;
        pool_data.vault = vault;
        pool_data.total_funds = 0;
        pool_data.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn process_allocate_funds(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let vault_program_account = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let pool_vault_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        // Verify that the caller is the Vault program
        let expected_vault_program_id = /* Insert the actual Vault Program ID */;
        if *vault_program_account.key != expected_vault_program_id {
            return Err(PoolError::Unauthorized.into());
        }

        // Derive the Pool PDA
        let (pool_pda, _bump) = Pubkey::find_program_address(&[POOL_SEED], program_id);
        if pool_pda != *pool_account.key {
            return Err(PoolError::InvalidPoolAccount.into());
        }

        // Update the PoolState
        let mut pool_data = PoolState::try_from_slice(&pool_account.data.borrow())?;
        pool_data.total_funds += amount;
        pool_data.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

        // Transfer funds from the Vault to the Pool's vault account
        invoke(
            &system_instruction::transfer(
                vault_program_account.key,
                pool_vault_account.key,
                amount,
            ),
            &[
                vault_program_account.clone(),
                pool_vault_account.clone(),
                system_program.clone(),
            ],
        )?;

        Ok(())
    }
}
