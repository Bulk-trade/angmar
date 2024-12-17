import { struct, u8, str, u64, u16, bool, u32 } from "@coral-xyz/borsh";

export const vaultInstructionLayout = struct([
    u8("variant"),
    str("vault_id"),
    str("user_pubkey"),
    u64("amount"),
    str("fund_status"),
    str("bot_status"),
    u16("market_index"),
]);

export const initVaultInstuctionLayout = struct([
    u8("variant"),
    str("name"),
    u64("redeem_period"),
    u64("max_tokens"),
    u64("management_fee"),
    u64("min_deposit_amount"),
    u32("profit_share"),
    u32("hurdle_rate"),
    u16("spot_market_index"),
    bool("permissioned"),
]);

export const depositInstuctionLayout = struct([
    u8("variant"),
    str("name"),
    u64("amount"),
]);

export const requestWithdrawInstuctionLayout = struct([
    u8("variant"),
    u64("amount"),
]);

export const updateDelegateInstuctionLayout = struct([
    u8("variant"),
    str("name"),
    str("delegate"),
    u16("sub_account"),
]);