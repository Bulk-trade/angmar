import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction } from "@solana/web3.js";

export enum FundStatus {
    Nil = 'Nil',
    Deposited = 'Deposited',
    Pending = 'Pending',
    Failed = 'Failed',
    Withdrawn = 'Withdrawn',
    Locked = 'Locked'
}

export enum BotStatus {
    Init = 'Init',
    Active = 'Active',
    Inactive = 'Inactive',
    Paused = 'Paused',
    Error = 'Error',
    Stopped = 'Stopped'
}

// async function wrapSol(connection: Connection, wallet: Keypair): Promise<PublicKey> {
//     const associatedTokenAccount = await getAssociatedTokenAddress(
//         NATIVE_MINT,
//         wallet.publicKey
//     );

//     const wrapTransaction = new Transaction().add(
//         createAssociatedTokenAccountInstruction(
//             wallet.publicKey,
//             associatedTokenAccount,
//             wallet.publicKey,
//             NATIVE_MINT
//         ),
//         SystemProgram.transfer({
//             fromPubkey: wallet.publicKey,
//             toPubkey: associatedTokenAccount,
//             lamports: LAMPORTS_PER_SOL,
//         }),
//         createSyncNativeInstruction(associatedTokenAccount)
//     );
//     await sendAndConfirmTransaction(connection, wrapTransaction, [wallet]);

//     console.log("✅ - Step 2: SOL wrapped");
//     return associatedTokenAccount;
// }