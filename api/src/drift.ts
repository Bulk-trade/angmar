import { DRIFT_PROGRAM_ID, getDriftStateAccountPublicKey, getSpotMarketPublicKey, getUserAccountPublicKeySync, getUserStatsAccountPublicKey } from "@drift-labs/sdk";
import { getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
    PublicKey,
    SystemProgram,
    AccountMeta,
    Connection,
} from "@solana/web3.js";
import { Keypair } from "@solana/web3.js";
import { getVaultPDA } from "./util";

// import { BaseClient, ApiTxOptions } from "./base";
export const DRIFT_PROGRAM = new PublicKey(DRIFT_PROGRAM_ID);
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
    signer: PublicKey, programId: PublicKey, vault: PublicKey,
): Promise<AccountMeta[]> {

    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);


    return [
        {
            pubkey: DRIFT_PROGRAM,
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

export async function getDriftDepositKeys(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
    userTokenAccount: PublicKey,
    treasuryTokenAccount: PublicKey,
    vaultName: string,
    spotMarket: PublicKey,
    spotMarketVault: PublicKey,
    oracle: PublicKey,
    mint: PublicKey
): Promise<AccountMeta[]> {

    const vault = getVaultPDA(vaultName, programId);
    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);

    const vaultTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        signer,
        mint,
        vault,
        true
    )).address;

    return [
        {
            pubkey: DRIFT_PROGRAM,
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
            pubkey: spotMarketVault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: oracle,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: spotMarket,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: userTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vaultTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: treasuryTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: mint,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
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

export async function getDriftManagerDepositKeys(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
    managerTokenAccount: PublicKey,
    vaultName: string,
    spotMarket: PublicKey,
    spotMarketVault: PublicKey,
    oracle: PublicKey,
    mint: PublicKey
): Promise<AccountMeta[]> {

    const vault = getVaultPDA(vaultName, programId);
    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);

    const vaultTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        signer,
        mint,
        vault,
        true
    )).address;

    return [
        {
            pubkey: DRIFT_PROGRAM,
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
            pubkey: spotMarketVault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: oracle,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: spotMarket,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: managerTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vaultTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: mint,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
        },
    ]
}

export async function getDriftWithdrawKeys(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
    userTokenAccount: PublicKey,
    treasuryTokenAccount: PublicKey,
    vaultName: string,
    spotMarket: PublicKey,
    spotMarketVault: PublicKey,
    oracle: PublicKey,
    mint: PublicKey
): Promise<AccountMeta[]> {

    const vault = getVaultPDA(vaultName, programId);
    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);

    const vaultTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        signer,
        mint,
        vault,
        true
    )).address;

    return [
        {
            pubkey: DRIFT_PROGRAM,
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
            pubkey: spotMarketVault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: oracle,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: spotMarket,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: new PublicKey('JCNCMFXo5M5qwUPg2Utu1u6YWp3MbygxqBsBeXXJfrw'),
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: userTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vaultTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: treasuryTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: mint,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
        },
    ]
}


export async function getDriftManagerWithdrawKeys(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
    userTokenAccount: PublicKey,
    vaultName: string,
    spotMarket: PublicKey,
    spotMarketVault: PublicKey,
    oracle: PublicKey,
    mint: PublicKey
): Promise<AccountMeta[]> {

    const vault = getVaultPDA(vaultName, programId);
    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);

    const vaultTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        signer,
        mint,
        vault,
        true
    )).address;

    return [
        {
            pubkey: DRIFT_PROGRAM,
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
            pubkey: spotMarketVault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: oracle,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: spotMarket,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: new PublicKey('JCNCMFXo5M5qwUPg2Utu1u6YWp3MbygxqBsBeXXJfrw'),
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: userTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vaultTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: mint,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
        },
    ]
}


export function getDriftUser(vault: PublicKey, subAccountId: number = 0): PublicKey[] {
    return [
        getUserAccountPublicKeySync(DRIFT_PROGRAM, vault),
        getUserStatsAccountPublicKey(DRIFT_PROGRAM, vault),
    ];
}

