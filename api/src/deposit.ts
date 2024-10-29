import {
    AccountMeta,
    Connection,
    Keypair,
    PublicKey,
    sendAndConfirmTransaction,
    SystemProgram,
    Transaction,
    TransactionInstruction,
    SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { getVaultPda, getTreasuryPda } from "./pda_utils";

export async function deposit(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vaultId: string,
    userPubkey: string,
    amount: number
) {
    // Implementation of the deposit function
    // Prepare PDAs and accounts
    // Construct instruction data
    // Send transaction
}
