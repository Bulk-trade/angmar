import { DRIFT_PROGRAM_ID, getDriftStateAccountPublicKey, getUserAccountPublicKeySync, getUserStatsAccountPublicKey, sigNum } from "@drift-labs/sdk";
import {
    PublicKey,
    SystemProgram,
    AccountMeta,
} from "@solana/web3.js";
// import {
//     DRIFT_PROGRAM_ID,
//     getDriftStateAccountPublicKey,
//     getUserAccountPublicKeySync,
//     getUserStatsAccountPublicKey,
//     MarketType,
//     OrderParams,
//     DriftClient as _DriftClient,
//     initialize as _initialize,
//     PositionDirection,
//     BulkAccountLoader,
//     decodeUser,
// } from "@drift-labs/sdk";

// import { BaseClient, ApiTxOptions } from "./base";
const DRIFT_PROGRAM = new PublicKey(DRIFT_PROGRAM_ID);
const DRIFT_VAULT = new PublicKey(
    "JCNCMFXo5M5qwUPg2Utu1u6YWp3MbygxqBsBeXXJfrw"
);
const DRIFT_MARGIN_PRECISION = 10_000;

const remainingAccountsForOrders = [
    {
        pubkey: new PublicKey("BAtFj4kQttZRVep3UZS2aZRDixkGYgWsbqTBVDbnSsPF"), // sol pricing oracle
        isWritable: false,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("En8hkHLkRe9d9DraYmBTrus518BvmVH448YcvmrFM6Ce"), // usdc pricing oracle
        isWritable: false,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("3x85u7SWkmmr7YQGYhtjARgxwegTLJgkSLRprfXod6rh"), // sol spot market account
        isWritable: true,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("6gMq3mRCKf8aP3ttTyYhuijVZ2LGi14oDsBbkgubfLB3"), // usdc spot market
        isWritable: true,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("8UJgxaiQx5nTrdDgph5FiahMmzduuLTLf5WmsPegYA6W"), // sol perp market account
        isWritable: true,
        isSigner: false,
    },
];

export async function getInitializeDriftKeys(
    signer: PublicKey, programId: PublicKey, vaultId: String,
): Promise<AccountMeta[]> {
    const [vault_signer] = PublicKey.findProgramAddressSync(
        [Buffer.from("signer"), Buffer.from(vaultId)],
        programId
    );

    console.log(`Vault Signer PDA is: ${vault_signer}`);

    const vault = getVaultPda(programId, vaultId);
    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);
   

    return [
        {
            pubkey: new PublicKey('dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH'),
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: user,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: userStats,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: state,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: signer,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: new PublicKey('SysvarRent111111111111111111111111111111111'),
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
        },
    ]
}

export async function getDriftKeys(
    spotMarketIndex: number,
    signer: PublicKey,
    programId: PublicKey,
    vaultId: String,
) {

    const vault = getVaultPda(programId, vaultId);

    const driftState = await this.driftClient.getStatePublicKey();
    const spotMarket = this.driftClient.getSpotMarketAccount(
        spotMarketIndex
    );
    if (!spotMarket) {
        throw new Error(
            `Spot market ${spotMarketIndex} not found on driftClient`
        );
    }

    const userStatsKey = getUserStatsAccountPublicKey(
        this.driftClient.program.programId,
        vault
    );
    const userKey = getUserAccountPublicKeySync(
        this.driftClient.program.programId,
        vault
    );

    const accounts = {
        driftSpotMarket: spotMarket.pubkey,
        driftSpotMarketMint: spotMarket.mint,
        driftUserStats: userStatsKey,
        driftUser: userKey,
        driftState,
        vault,
    };

    return accounts;
}

function getDriftUser(vault: PublicKey, subAccountId: number = 0): PublicKey[] {
    return [
        getUserAccountPublicKeySync(DRIFT_PROGRAM, vault),
        getUserStatsAccountPublicKey(DRIFT_PROGRAM, vault),
    ];
}

function getVaultPda(programId: PublicKey, vaultId: String) {
    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from(vaultId)],
        programId
    );

    return pda;
}