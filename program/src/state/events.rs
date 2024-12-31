use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Serialize, Deserialize)]
pub struct VaultDepositorRecord {
    pub ts: i64,
    pub vault: Pubkey,
    pub depositor_authority: Pubkey,
    pub action: VaultDepositorAction,
    pub amount: u64,

    pub spot_market_index: u16,
    pub vault_shares_before: u128,
    pub vault_shares_after: u128,

    pub vault_equity_before: u64,

    pub user_vault_shares_before: u128,
    pub total_vault_shares_before: u128,

    pub user_vault_shares_after: u128,
    pub total_vault_shares_after: u128,

    pub profit_share: u32,
    pub profit_share_amount: u64,
    pub management_fee: u64,
    pub management_fee_amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Serialize, Deserialize)]
pub enum VaultDepositorAction {
    Deposit,
    WithdrawRequest,
    CancelWithdrawRequest,
    Withdraw,
    CollectFees,
}
