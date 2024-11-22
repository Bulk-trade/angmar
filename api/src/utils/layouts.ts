import { struct, u8, str, u64, u16, bool, u32 } from "@coral-xyz/borsh";

export const vaultInstructionLayout = struct([
    u8("variant"),
    str("vault_id"),
    str("user_pubkey"),
    u64("amount"),
    str("fund_status"),
    str("bot_status"),
    u16("market_index"),
    str("delegate"),
    u16("sub_account")
]);

export const initVaultInstuctionLayout = struct([
    u8("variant"),
    str("name"),
    u64("management_fee"),
    u64("min_deposit_amount"),
    u32("profit_share"),
    u16("spot_market_index"),
    bool("permissioned"),
]);