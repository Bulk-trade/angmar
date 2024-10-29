import {
    AccountMeta,
    Connection,
    Keypair,
    PublicKey,
    sendAndConfirmTransaction,
    SystemProgram,
    Transaction,
    TransactionInstruction,
} from "@solana/web3.js";
import { getVaultPda, getTreasuryPda } from "./pda_utils";

export async function withdraw(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vaultId: string,
    userPubkey: string,
    amount: number
) {
    // Implementation of the withdraw function
    // Prepare PDAs and accounts
    // Construct instruction data
    // Send transaction
}
