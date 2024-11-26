use crate::drift::{DepositIxArgs, DepositIxData};
use crate::state::Vault;
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
};
use spl_token::instruction;

pub fn deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    mut amount: u64,
) -> ProgramResult {
    msg!("Starting deposit...");
    msg!("name: {}", name);
    msg!("amount: {}", amount);

    let account_info_iter = &mut accounts.iter();

    let vault = next_account_info(account_info_iter)?;
    let vault_depositor = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;
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
    msg!("vault: {}", vault.key);
    msg!("vault_depositor: {}", vault_depositor.key);
    msg!("authority: {}", authority.key);
    msg!("treasury: {}", treasury.key);

    // Second batch - Drift accounts
    msg!("drift_program: {}", drift_program.key);
    msg!("drift_user: {}", drift_user.key);
    msg!("drift_user_stats: {}", drift_user_stats.key);
    msg!("drift_state: {}", drift_state.key);
    msg!("drift_spot_market_vault: {}", drift_spot_market_vault.key);
    msg!("drift_oracle: {}", drift_oracle.key);
    msg!("drift_spot_market: {}", drift_spot_market.key);

    // Third batch - Token accounts
    msg!("user_token_account: {}", user_token_account.key);
    msg!("vault_token_account: {}", vault_token_account.key);
    msg!("treasury_token_account: {}", treasury_token_account.key);
    msg!("mint: {}", mint.key);

    // Fourth batch - System accounts
    msg!("token_program: {}", token_program.key);
    msg!("system_program: {}", system_program.key);

    if !authority.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (vault_depositor_pda, _) = Pubkey::find_program_address(
        &[
            b"vault_depositor",
            vault.key.as_ref(),
            authority.key.as_ref(),
        ],
        program_id,
    );

    if vault_depositor_pda != *vault_depositor.key {
        msg!("Invalid seeds for Vault Depositor PDA");
        return Err(ProgramError::InvalidArgument);
    }

    const FEE_PERCENTAGE: u64 = 2;
    let fees = (amount * FEE_PERCENTAGE + 99) / 100;
    amount -= fees;

    msg!("unpacking vault state account");
    let vault_data = try_from_slice_unchecked::<Vault>(&vault.data.borrow())?;

    // Print the specified fields from vault_data
    msg!("Name: {:?}", vault_data.name);
    msg!("Pubkey: {:?}", vault_data.pubkey);
    msg!("Manager: {:?}", vault_data.manager);
    msg!("User Stats: {:?}", vault_data.user_stats);
    msg!("User: {:?}", vault_data.user);
    msg!("Token Account: {:?}", vault_data.token_account);
    msg!("Spot Market Index: {:?}", vault_data.spot_market_index);
    msg!("Init Timestamp: {:?}", vault_data.init_ts);
    msg!("Min Deposit Amount: {:?}", vault_data.min_deposit_amount);
    msg!("Management Fee: {:?}", vault_data.management_fee);
    msg!("Profit Share: {:?}", vault_data.profit_share);
    msg!("Bump: {:?}", vault_data.bump);
    msg!("Permissioned: {:?}", vault_data.permissioned);

    let clock = &Clock::get()?;

    let spot_market_index = vault_data.spot_market_index;
    let AccountMaps {
        perp_market_map,
        spot_market_map,
        mut oracle_map,
    } = load_maps(clock.slot, Some(spot_market_index), vp.is_some())?;

    let vault_equity =
        vault_data.calculate_equity(&user, &perp_market_map, &spot_market_map, &mut oracle_map)?;

    // vault_depositor.deposit(
    //     amount,
    //     vault_equity,
    //     &mut vault,
    //     &mut vp,
    //     clock.unix_timestamp,
    // )?;

    msg!("Depositing Fees to Treasury Pda...");
    invoke(
        &instruction::transfer(
            &token_program.key,
            &user_token_account.key,
            &treasury_token_account.key,
            &authority.key,
            &[authority.key],
            fees,
        )?,
        &[
            mint.clone(),
            user_token_account.clone(),
            treasury_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
    )?;

    if vault_data.pubkey != *vault.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Transfering to Vault Pda...");
    invoke(
        &instruction::transfer(
            &token_program.key,
            &user_token_account.key,
            &vault_token_account.key,
            &authority.key,
            &[authority.key],
            amount,
        )?,
        &[
            mint.clone(),
            user_token_account.clone(),
            vault_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
    )?;

    let (vault_pda, vault_bump_seed) =
        Pubkey::find_program_address(&[b"vault", name.as_ref()], program_id);

    if vault_pda != *vault.key {
        msg!("Invalid seeds for Vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

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
            pubkey: *drift_oracle.key,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *drift_spot_market.key,
            is_signer: false,
            is_writable: true,
        },
    ];

    let args = DepositIxArgs {
        market_index: vault_data.spot_market_index,
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
            drift_oracle.clone(),
            drift_spot_market.clone(),
            drift_program.clone(),
        ],
        &[&[b"vault", name.as_ref(), &[vault_bump_seed]]],
    )?;

    Ok(())
}

fn trim_trailing_zeros(input: &[u8]) -> &[u8] {
    let end = input.iter().position(|&x| x == 0).unwrap_or(input.len());
    &input[..end]
}

fn load_maps(slot: u64, writable_spot_market_index: Option<u16>, has_vault_protocol: bool) {
    //https://github.com/drift-labs/drift-vaults/blob/827b6746c327e620a784a4163c9453c2176f7b72/programs/drift_vaults/src/state/account_maps.rs#L19
}
