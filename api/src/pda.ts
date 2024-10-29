import { struct, u8, str, f32 } from "@coral-xyz/borsh";
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
import { getInitializeDriftKeys } from "./drift";
import { getVaultPda, getTreasuryPda } from "./pda_utils"; // Assume these functions are imported or defined

export async function initializeDrift(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vaultId: string
) {
    console.log('Received init drift parameters:', { vaultId });

    // Get PDAs
    const vaultPdaAccount = getVaultPda(programId, vaultId);
    const treasuryPdaAccount = getTreasuryPda(programId, vaultId);

    // Get Drift initialization keys
    const driftKeys = await getInitializeDriftKeys(
        vaultPdaAccount,      // authority
        signer.publicKey,     // signer
        programId,
        vaultId
    );

    // Prepare the accounts in the correct order
    const accounts: AccountMeta[] = [
        { pubkey: signer.publicKey, isSigner: true, isWritable: false }, // initializer (Account 0)
        { pubkey: vaultPdaAccount, isSigner: false, isWritable: true },  // vault_pda_account (Account 1)
        { pubkey: treasuryPdaAccount, isSigner: false, isWritable: true }, // treasury_pda_account (Account 2)
        ...driftKeys, // Rest of the accounts
    ];

    // Construct instruction data
    const instructionData = Buffer.from(Uint8Array.of(3, ...Buffer.from(vaultId)));

    const instruction = new TransactionInstruction({
        keys: accounts,
        programId,
        data: instructionData,
    });

    const transaction = new Transaction().add(instruction);

    try {
        const txid = await sendAndConfirmTransaction(connection, transaction, [signer]);
        console.log('Transaction ID:', txid);
    } catch (error) {
        console.error('Error sending transaction:', error);
    }
}

// Export the functions from their respective files
export { deposit } from './deposit';
export { initializeVault } from './initializeVault';
export { readPdaInfo } from './readPdaInfo';
export { updateUserInfo } from './updateUserInfo';
export { withdraw } from './withdraw';
