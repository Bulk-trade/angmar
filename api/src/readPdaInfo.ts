import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { getVaultPda, getTreasuryPda } from "./pda_utils";

export async function readPdaInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    userPubkey: string
) {
    // Implementation of the readPdaInfo function
    // Fetch account data
    // Deserialize data
    // Return the parsed information
}
