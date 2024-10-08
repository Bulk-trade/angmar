import { struct, u8, str, u32, f32 } from "@coral-xyz/borsh";
import { Connection, Keypair, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { BotStatus, FundStatus } from "./util";

const vaultInstructionLayout = struct([
    u8("variant"),
    str("user_pubkey"),
    f32("amount"),
    str("fund_status"),
    str("bot_status"),
]);

export async function readUserInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    user_pubkey: string,
) {
    const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes

    const [pda] = PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(pubKey)],
        programId
    );

    const accountInfo = await connection.getAccountInfo(pda);
    if (accountInfo === null) {
        console.log("Account not found");
        return null;
    }
    //Deserialize the data
    const data = vaultInstructionLayout.decode(accountInfo.data);
    // Convert the user_pubkey from bytes to a PublicKey string
    console.log({
        variant: data.variant,
        user_pubkey: data.user_pubkey,
        amount: data.amount,
        fund_status: data.fund_status,
        bot_status: data.bot_status,
    });

    return {
        variant: data.variant,
        user_pubkey: data.user_pubkey,
        amount: data.amount,
        fund_status: data.fund_status,
        bot_status: data.bot_status,
    }
}

export async function addUserInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    user_pubkey: string,
    amount: number,
) {
    let buffer = Buffer.alloc(1000);
    const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 0,
            user_pubkey: pubKey,
            amount: amount,
            fund_status: FundStatus.Deposited,
            bot_status: BotStatus.Init,
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

export async function updateUserInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    user_pubkey: string,
    amount: number,
) {
    let buffer = Buffer.alloc(1000);
    const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 1,
            user_pubkey: pubKey,
            amount: amount,
            fund_status: FundStatus.Deposited,
            bot_status: BotStatus.Init,
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



