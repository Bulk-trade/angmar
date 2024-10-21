import { struct, u8, str, u32, f32 } from "@coral-xyz/borsh";
import { Connection, Keypair, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { BotStatus, FundStatus } from "./util";

const vaultInstructionLayout = struct([
    u8("variant"),
    str("vault_id"),
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
    //const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes

    const [pda] = PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(user_pubkey)],
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
        vault: data.vault_id,
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

export async function initializeVault(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vault_id: string,
) {
    let buffer = Buffer.alloc(1000);
    const pubKey = vault_id.slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 0,
            vault_id: pubKey
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));


    const [vault_pda] = await PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(vault_id)],
        programId
    );

    console.log("PDA is:", vault_pda.toBase58());

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
                pubkey: vault_pda,
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
    const tx = await sendAndConfirmTransaction(connection, transaction, [signer], { skipPreflight: true });
    console.log(`https://solscan.io//tx/${tx}`);
    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
}

export async function deposit(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vault_id: string,
    user_pubkey: string,
    amount: number,
) {

    // Log the input parameters
    console.log('Received deposit parameters:', { vault_id, user_pubkey, amount });

    // Validate input parameters
    if (!vault_id || !user_pubkey) {
        throw new Error('vault_id and user_pubkey must be defined');
    }

    let buffer = Buffer.alloc(1000);
    const vault = vault_id.slice(0, 32); // Truncate to 32 bytes
    const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 1,
            vault_id: vault,
            user_pubkey: pubKey,
            amount: amount,
            fund_status: FundStatus.Deposited,
            bot_status: BotStatus.Init,
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));

    const [user_info_pda] = await PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(pubKey)],
        programId
    );


    console.log("User PDA is:", user_info_pda.toBase58());

    const [vault_pda] = await PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(vault_id)],
        programId
    );

    console.log("Vault PDA is:", vault_pda.toBase58());

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
                pubkey: user_info_pda,
                isSigner: false,
                isWritable: true,
            },
            {
                pubkey: vault_pda,
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
    const tx = await sendAndConfirmTransaction(connection, transaction, [signer], { skipPreflight: true });
    console.log(`https://solscan.io//tx/${tx}`);
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
    console.log(`https://solscan.io//tx/${tx}`);
}



