use borsh::{BorshDeserialize, BorshSerialize};
use drift::error::ErrorCode;
use drift::math::casting::Cast;
use drift::math::margin::calculate_user_equity;
use drift::math::safe_math::SafeMath;
use drift::state::oracle_map::OracleMap;
use drift::state::perp_market_map::PerpMarketMap;
use drift::state::spot_market_map::SpotMarketMap;
use drift::state::user::User;
use drift::validate;
use solana_program::account_info::AccountInfo;
use solana_program::borsh0_10::try_from_slice_unchecked;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_pack::Sealed;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use std::result::Result;

use crate::common::{calculate_amount_to_shares, log_data, log_params};
use crate::constants::PERCENTAGE_PRECISION_U64;
use crate::custom_validate;
use crate::error::{wrap_drift_error, VaultErrorCode};
use crate::state::{VaultDepositorAction, VaultDepositorRecord};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Vault {
    /// The name of the vault. Vault pubkey is derived from this name.
    pub name: [u8; 32],
    /// The vault's pubkey. It is a pda of name and also used as the authority for drift user
    pub pubkey: Pubkey,
    /// The manager of the vault who has ability to update vault params
    pub manager: Pubkey,
    /// The vaults token account. Used to receive tokens between deposits and withdrawals
    pub token_account: Pubkey,
    /// The drift user stats account for the vault
    pub user_stats: Pubkey,
    /// The drift user account for the vault
    pub user: Pubkey,
    /// The vaults designated delegate for drift user account
    /// can differ from actual user delegate if vault is in liquidation
    pub delegate: Pubkey,
    /// The delegate handling liquidation for depositor
    pub liquidation_delegate: Pubkey,
    /// The sum of all shares held by the users (vault depositors)
    pub user_shares: u128,
    /// The sum of all shares: deposits from users, manager deposits, manager profit/fee, and protocol profit/fee.
    /// The manager deposits are total_shares - user_shares - protocol_profit_and_fee_shares.
    pub total_shares: u128,
    /// Last fee update unix timestamp
    pub last_fee_update_ts: u64,
    /// When the liquidation starts
    pub liquidation_start_ts: u64,
    /// The period (in seconds) that deposits must remain in the vault before withdrawal can be requested.
    pub lock_in_period: u64,
    /// The period (in seconds) that a vault depositor must wait after requesting a withdrawal to finalize withdrawal.
    pub redeem_period: u64,
    /// The sum of all outstanding withdraw requests
    pub total_withdraw_requested: u64,
    /// Max token capacity, once hit/passed vault will reject new deposits (updatable)
    pub max_tokens: u64,
    /// The annual fee charged on deposits by the manager.
    /// Traditional funds typically charge 2% per year on assets under management.
    pub management_fee: u64,
    /// Timestamp vault initialized
    pub init_ts: u64,
    /// The net deposits for the vault
    pub net_deposits: u64,
    /// The net deposits for the manager
    pub manager_net_deposits: u64,
    /// Total deposits
    pub total_deposits: u64,
    /// Total withdraws
    pub total_withdraws: u64,
    /// Total deposits for the manager
    pub manager_total_deposits: u64,
    /// Total withdraws for the manager
    pub manager_total_withdraws: u64,
    /// Total management fee accrued by the manager
    pub manager_total_fee: u64,
    /// Total profit share accrued by the manager
    pub manager_total_profit_share: u64,
    /// The minimum deposit amount
    pub min_deposit_amount: u64,
    /// The base 10 exponent of the shares (given massive share inflation can occur at near zero vault equity)
    pub shares_base: u32,
    /// Percentage the manager charges on all profits realized by depositors: PERCENTAGE_PRECISION
    pub profit_share: u32,
    /// Vault manager only collect incentive fees during periods when returns are higher than this amount: PERCENTAGE_PRECISION
    pub hurdle_rate: u32,
    /// The spot market index the vault deposits into/withdraws from
    pub spot_market_index: u16,
    /// The bump for the vault pda
    pub bump: u8,
    /// Whether anybody can be a depositor
    pub permissioned: bool,
}

impl Sealed for Vault {}

impl Vault {
    pub const SIZE: usize = std::mem::size_of::<Vault>() + 8;

    pub fn get_vault_signer_seeds<'a>(name: &'a str, bump: &'a [u8]) -> [&'a [u8]; 3] {
        [b"vault", name.as_bytes(), bump]
    }

    pub fn get_pda<'a>(name: &String, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vault", name.as_bytes()], program_id)
    }

    pub fn get(account: &AccountInfo) -> Self {
        try_from_slice_unchecked::<Vault>(&account.data.borrow()).unwrap()
    }

    pub fn save(vault: &Vault, vault_account: &AccountInfo) -> ProgramResult {
        vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;
        Ok(())
    }

    pub fn get_manager_shares(&self) -> u128 {
        self.total_shares.saturating_sub(self.user_shares)
    }

    pub fn calculate_total_equity(
        &self,
        user: &User,
        perp_market_map: &PerpMarketMap,
        spot_market_map: &SpotMarketMap,
        oracle_map: &mut OracleMap,
    ) -> Result<u64, drift::error::ErrorCode> {
        let (vault_equity, all_oracles_valid) =
            calculate_user_equity(user, perp_market_map, spot_market_map, oracle_map)?;

        validate!(
            all_oracles_valid,
            drift::error::ErrorCode::InvalidOracle,
            "oracle invalid"
        )?;

        validate!(
            vault_equity >= 0,
            drift::error::ErrorCode::InvalidOracle,
            "vault equity negative"
        )?;

        let spot_market = spot_market_map.get_ref(&self.spot_market_index)?;
        let spot_market_precision = spot_market.get_precision().cast::<i128>()?;
        let oracle_price = oracle_map
            .get_price_data(&spot_market.oracle)?
            .price
            .cast::<i128>()?;

        Ok(vault_equity
            .safe_mul(spot_market_precision)?
            .safe_div(oracle_price)?
            .cast::<u64>()?)
    }

    pub fn calculate_fees(&self, amount: u64) -> u64 {
        let numerator = amount
            .checked_mul(self.management_fee)
            .expect("Fee calculation overflow");
        (numerator + PERCENTAGE_PRECISION_U64 - 1) / PERCENTAGE_PRECISION_U64
    }

    pub fn manager_deposit(
        &mut self,
        amount: u64,
        vault_equity: u64,
        now: i64,
    ) -> Result<(), ProgramError> {
        custom_validate!(
            self.max_tokens == 0 || self.max_tokens > vault_equity.saturating_add(amount),
            VaultErrorCode::VaultIsAtCapacity,
            "after deposit vault equity is {} > {}",
            vault_equity + amount,
            self.max_tokens
        )?;

        let vault_shares_before = self.get_manager_shares();
        let total_vault_shares_before = self.total_shares;
        let user_vault_shares_before = self.user_shares;

        let new_shares =
            calculate_amount_to_shares(amount, total_vault_shares_before, vault_equity)?;
        msg!("Issuing user shares: {}", new_shares);

        self.total_deposits = self.total_deposits.saturating_add(amount);
        self.manager_total_deposits = self.manager_total_deposits.saturating_add(amount);
        self.net_deposits = self.net_deposits.saturating_add(amount);
        self.manager_net_deposits = self.manager_net_deposits.saturating_add(amount);

        self.total_shares = self.total_shares.saturating_add(new_shares);
        let vault_shares_after = self.get_manager_shares();

        msg!("Vault Deposit Record");
        let record = VaultDepositorRecord {
            ts: now,
            vault: self.pubkey,
            depositor_authority: self.manager,
            action: VaultDepositorAction::Deposit,
            amount,
            spot_market_index: self.spot_market_index,
            vault_equity_before: vault_equity,
            vault_shares_before,
            user_vault_shares_before,
            total_vault_shares_before,
            vault_shares_after,
            total_vault_shares_after: self.total_shares,
            user_vault_shares_after: self.user_shares,
            profit_share: self.profit_share,
            profit_share_amount: 0,
            management_fee: self.management_fee,
            management_fee_amount: 0,
        };

        log_data(&record)?;

        log_params(&record);

        Ok(())
    }

    pub fn manager_withdraw(
        &mut self,
        amount: u64,
        vault_equity: u64,
        now: i64,
    ) -> Result<(), ProgramError> {
        let vault_shares_before = self.get_manager_shares();
        let total_vault_shares_before = self.total_shares;
        let user_vault_shares_before = self.user_shares;

        let shares = calculate_amount_to_shares(amount, self.total_shares, vault_equity)?;

        custom_validate!(
            shares > 0,
            VaultErrorCode::InvalidVaultWithdrawSize,
            "Requested shares = 0"
        )?;

        custom_validate!(
            vault_shares_before >= shares,
            VaultErrorCode::InsufficientVaultShares
        )?;

        self.total_withdraws = self.total_withdraws.saturating_add(amount);
        self.manager_total_withdraws = self.manager_total_withdraws.saturating_add(amount);
        self.net_deposits = self.net_deposits.saturating_sub(amount);
        self.manager_net_deposits = self.manager_net_deposits.saturating_sub(amount);

        self.total_shares = self.total_shares.saturating_sub(shares);
        let vault_shares_after = self.get_manager_shares();

        msg!("Vault Deposit Record");
        let record = VaultDepositorRecord {
            ts: now,
            vault: self.pubkey,
            depositor_authority: self.manager,
            action: VaultDepositorAction::Withdraw,
            amount,
            spot_market_index: self.spot_market_index,
            vault_equity_before: vault_equity,
            vault_shares_before,
            user_vault_shares_before,
            total_vault_shares_before,
            vault_shares_after,
            total_vault_shares_after: self.total_shares,
            user_vault_shares_after: self.user_shares,
            profit_share: self.profit_share,
            profit_share_amount: 0,
            management_fee: self.management_fee,
            management_fee_amount: 0,
        };

        log_data(&record)?;

        log_params(&record);

        Ok(())
    }
}
