import { DRIFT_PROGRAM_ID, getDriftStateAccountPublicKey, getUserAccountPublicKeySync, getUserStatsAccountPublicKey } from "@drift-labs/sdk";
import {
    PublicKey,
    SystemProgram,
    AccountMeta,
    SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";

const DRIFT_PROGRAM = new PublicKey(DRIFT_PROGRAM_ID);

const remainingAccountsForOrders: AccountMeta[] = [
    {
        pubkey: new PublicKey("BAtFj4kQttZRVep3UZS2aZRDixkGYgWsbqTBVDbnSsPF"), // sol price oracle
        isWritable: false,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("En8hkHLkRe9d9DraYmBTrus518BvmVH448YcvmrFM6Ce"), // usdc price oracle
        isWritable: false,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("3x85u7SWkmmr7YQGYhtjARgxwegTLJgkSLRprfXod6rh"), // sol spot market
        isWritable: true,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("6gMq3mRCKf8aP3ttTyYhuijVZ2LGi14oDsBbkgubfLB3"), // usdc spot market
        isWritable: true,
        isSigner: false,
    },
    {
        pubkey: new PublicKey("8UJgxaiQx5nTrdDgph5FiahMmzduuLTLf5WmsPegYA6W"), // sol perp market
        isWritable: true,
        isSigner: false,
    },
];

export async function getInitializeDriftKeys(
    authority: PublicKey,
    signer: PublicKey,     // Added signer parameter
    programId: PublicKey,
    vaultId: string
): Promise<AccountMeta[]> {
    const vaultPdaAccount = authority; // Authority is vaultPdaAccount
    const [userAccount, userStatsAccount] = getDriftUserAccounts(vaultPdaAccount);
    const stateAccount = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);
    const [driftSigner] = PublicKey.findProgramAddressSync(
        [Buffer.from("drift_signer")],
        DRIFT_PROGRAM
    );

    return [
        { pubkey: userAccount, isSigner: false, isWritable: true },          // user
        { pubkey: userStatsAccount, isSigner: false, isWritable: true },     // user_stats
        { pubkey: stateAccount, isSigner: false, isWritable: false },        // state
        { pubkey: vaultPdaAccount, isSigner: false, isWritable: false },     // authority (vault PDA)
        { pubkey: signer, isSigner: true, isWritable: true },                // payer
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },  // rent sysvar
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // system program
        { pubkey: driftSigner, isSigner: false, isWritable: false },         // drift_signer
        // Remaining accounts
        ...remainingAccountsForOrders,
    ];
}

function getDriftUserAccounts(vault: PublicKey, subAccountId: number = 0): [PublicKey, PublicKey] {
    const userAccount = getUserAccountPublicKeySync(DRIFT_PROGRAM, vault, subAccountId);
    const userStatsAccount = getUserStatsAccountPublicKey(DRIFT_PROGRAM, vault);
    return [userAccount, userStatsAccount];
}

function getVaultPda(programId: PublicKey, vaultId: string): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from(vaultId)],
        programId
    );
    return pda;
}

function getTreasuryPda(programId: PublicKey, vaultId: string): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vaultId)],
        programId
    );
    return pda;
}
