// Client
import {
    Keypair,
    Connection,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SystemProgram,
    LAMPORTS_PER_SOL,
    clusterApiUrl,
    sendAndConfirmTransaction,
} from "@solana/web3.js";
import { struct, u8, str, u32 } from "@coral-xyz/borsh";
import { writeFileSync } from "fs";
import dotenv from "dotenv";
import {
    initializeKeypair,
    addKeypairToEnvFile,
} from "@solana-developers/helpers";

dotenv.config();


const vaultInstructionLayout = struct([
    u8("variant"),
    str("user_pubkey"),
    u32("amount"),
    str("fund_status"),
    str("bot_status"),
]);

async function updateUserInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection
) {
    let buffer = Buffer.alloc(1000);
    const pubKey = signer.publicKey.toString().slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 1,
            user_pubkey: pubKey,
            amount: Math.random() * 1000,
            fund_status: 'Deposited',
            bot_status: 'Active',
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));

    const [pda] = await PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(pubKey)],
        programId
    );

    console.log("PDA is:", pda.toBase58());

    const transaction = new Transaction();

    const instruction = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys: [
            {
                pubkey: signer.publicKey,
                isSigner: true,
                isWritable: false,
            },
            {
                pubkey: pda,
                isSigner: false,
                isWritable: true,
            },
            {
                pubkey: SystemProgram.programId,
                isSigner: false,
                isWritable: false,
            },
        ],
    });

    transaction.add(instruction);
    const tx = await sendAndConfirmTransaction(connection, transaction, [signer]);
    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
}

async function addUserInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection
) {
    let buffer = Buffer.alloc(1000);
    const pubKey = signer.publicKey.toString().slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 0,
            user_pubkey: pubKey,
            amount: Math.random() * 1000,
            fund_status: 'Nil',
            bot_status: 'Init',
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));

    const [pda] = await PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(pubKey)],
        programId
    );

    console.log("PDA is:", pda.toBase58());

    const transaction = new Transaction();

    const instruction = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys: [
            {
                pubkey: signer.publicKey,
                isSigner: true,
                isWritable: false,
            },
            {
                pubkey: pda,
                isSigner: false,
                isWritable: true,
            },
            {
                pubkey: SystemProgram.programId,
                isSigner: false,
                isWritable: false,
            },
        ],
    });

    transaction.add(instruction);
    const tx = await sendAndConfirmTransaction(connection, transaction, [signer]);
    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
}

(async () => {
    try {
        // const connection = new Connection(clusterApiUrl("testnet"));
        const connection = new Connection("http://localhost:8899", "confirmed");

        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        const userInfoProgramId = new PublicKey(
            "98PdopvDo8HrWnahS1EhGK3BFSY3BXKFWDCt7EmzSs7P"
        );

        //await addUserInfo(signer, userInfoProgramId, connection);
       await updateUserInfo(signer, userInfoProgramId, connection);
        
        console.log("Finished successfully");
    } catch (error) {
        console.error(error);
        process.exit(1);
    }
})();


//fund status, amount, user pubkey, bot status