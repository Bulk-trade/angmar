// use std::cell::Ref;
// use std::mem;
// use bytemuck::{Pod, Zeroable};
// use borsh::{BorshDeserialize, BorshSerialize};
// use drift::state::user::{Order, PerpPosition, SpotPosition};
// use solana_program::account_info::AccountInfo;
// use solana_program::borsh0_10::try_from_slice_unchecked;
// use solana_program::{pubkey::Pubkey, program_error::ProgramError};


// #[repr(C)]
// #[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
// pub struct User {
//     /// The owner/authority of the account
//     pub authority: Pubkey,
//     /// An addresses that can control the account on the authority's behalf. Has limited power, cant withdraw
//     pub delegate: Pubkey,
//     /// Encoded display name e.g. "toly"
//     pub name: [u8; 32],
//     /// The user's spot positions
//     pub spot_positions: [SpotPosition; 8],
//     /// The user's perp positions
//     pub perp_positions: [PerpPosition; 8],
//     /// The user's orders
//     pub orders: [Order; 32],
//     /// The last time the user added perp lp positions
//     pub last_add_perp_lp_shares_ts: i64,
//     /// The total values of deposits the user has made
//     /// precision: QUOTE_PRECISION
//     pub total_deposits: u64,
//     /// The total values of withdrawals the user has made
//     /// precision: QUOTE_PRECISION
//     pub total_withdraws: u64,
//     /// The total socialized loss the users has incurred upon the protocol
//     /// precision: QUOTE_PRECISION
//     pub total_social_loss: u64,
//     /// Fees (taker fees, maker rebate, referrer reward, filler reward) and pnl for perps
//     /// precision: QUOTE_PRECISION
//     pub settled_perp_pnl: i64,
//     /// Fees (taker fees, maker rebate, filler reward) for spot
//     /// precision: QUOTE_PRECISION
//     pub cumulative_spot_fees: i64,
//     /// Cumulative funding paid/received for perps
//     /// precision: QUOTE_PRECISION
//     pub cumulative_perp_funding: i64,
//     /// The amount of margin freed during liquidation. Used to force the liquidation to occur over a period of time
//     /// Defaults to zero when not being liquidated
//     /// precision: QUOTE_PRECISION
//     pub liquidation_margin_freed: u64,
//     /// The last slot a user was active. Used to determine if a user is idle
//     pub last_active_slot: u64,
//     /// Every user order has an order id. This is the next order id to be used
//     pub next_order_id: u32,
//     /// Custom max initial margin ratio for the user
//     pub max_margin_ratio: u32,
//     /// The next liquidation id to be used for user
//     pub next_liquidation_id: u16,
//     /// The sub account id for this user
//     pub sub_account_id: u16,
//     /// Whether the user is active, being liquidated or bankrupt
//     pub status: u8,
//     /// Whether the user has enabled margin trading
//     pub is_margin_trading_enabled: bool,
//     /// User is idle if they haven't interacted with the protocol in 1 week and they have no orders, perp positions or borrows
//     /// Off-chain keeper bots can ignore users that are idle
//     pub idle: bool,
//     /// number of open orders
//     pub open_orders: u8,
//     /// Whether or not user has open order
//     pub has_open_order: bool,
//     /// number of open orders with auction
//     pub open_auctions: u8,
//     /// Whether or not user has open order with auction
//     pub has_open_auction: bool,
//     pub padding1: [u8; 5],
//     pub last_fuel_bonus_update_ts: u32,
//     pub padding: [u8; 12],
// }

// impl User {
//     // pub fn get(account: &AccountInfo) -> Self {
//     //     try_from_slice_unchecked::<User>(&account.data.borrow()).unwrap()
//     // }

//     pub fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
//         let data = account.try_borrow_data()?;

//         let user = Ref::map(data, |data| {
//             bytemuck::from_bytes(&data[8..mem::size_of::<User>() + 8])
//         });

//         Ok(*user)
//     }
// }
