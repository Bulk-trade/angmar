import { struct, u8, str, u32, f32, u64 } from "@coral-xyz/borsh";
import { AccountMeta, ComputeBudgetProgram, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { BotStatus, FundStatus } from "./util";
import { getInitializeDriftKeys } from "./drift";
import {createInitializeAccountInstruction, mintTo, TOKEN_PROGRAM_ID, TokenInstruction} from "@solana/spl-token"

const vaultInstructionLayout = struct([
    u8("variant"),
    str("vault_id"),
    str("user_pubkey"),
    u64("amount"),
    str("fund_status"),
    str("bot_status"),
]);

export async function readPdaInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    user_pubkey: string,
) {
    //const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes

    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from(user_pubkey)],
        programId
    );

    console.log("PDA is:", pda.toBase58());

    const accountInfo = await connection.getAccountInfo(pda);
    if (accountInfo === null) {
        console.log("Account not found");
        return null;
    }

    console.log(accountInfo.data)
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
    // Log the input parameters
    console.log('Received init parameters:', { vault_id });

    let buffer = Buffer.alloc(1000);
    const id = vault_id.slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 0,
            vault_id: id
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));


    const [vault, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from(vault_id)],
        programId
    );

    console.log("Vault PDA is:", vault.toBase58());

    const [treasury] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

   
    const keys: AccountMeta[] = [
        {
            pubkey: signer.publicKey,
            isSigner: true,
            isWritable: false,
        },
        {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: treasury,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
        },
    ];

    console.log(`Keys Length: ${keys.length}`);

    const transaction = new Transaction();

    const Ix = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys,
    });

    transaction.add(Ix)

    transaction.feePayer = signer.publicKey!;

    const latestBlockhash = await connection.getLatestBlockhash();
    transaction.recentBlockhash = latestBlockhash.blockhash;

    transaction.sign(signer);

    const tx = await sendAndConfirmTransaction(connection, transaction, [signer], { skipPreflight: true });
    console.log(`https://solscan.io//tx/${tx}`);
    console.log(`https://explorer.solana.com/tx/${tx}?cluster=custom`);
}

export async function initializeDrift(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vault_id: string,
) {
    const seedPubkey = await PublicKey.createWithSeed(signer.publicKey, 'seed', TOKEN_PROGRAM_ID);

    // Log the input parameters
    console.log('Received init drift parameters:', { vault_id });

    let buffer = Buffer.alloc(1000);
    const id = vault_id.slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 3,
            vault_id: id
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));


    const [vault, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from(vault_id)],
        programId
    );

    console.log("Vault PDA is:", vault.toBase58());

    const [treasury] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

    const driftKeys = await getInitializeDriftKeys(signer.publicKey, programId, vault_id);

    const keys: AccountMeta[] = [
        {
            pubkey: signer.publicKey,
            isSigner: true,
            isWritable: false,
        },
        {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: treasury,
            isSigner: false,
            isWritable: true,
        },
    ];

    keys.push(...driftKeys);

    console.log(`Keys Length: ${keys.length}`);

    const transaction = new Transaction();
    
    const computeBudgetInstruction =
        ComputeBudgetProgram.setComputeUnitLimit({
            units: 4_00_000,
        });

    const driftIx = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys,
    });

    transaction.add( computeBudgetInstruction, driftIx);

    transaction.feePayer = signer.publicKey!;

    const latestBlockhash = await connection.getLatestBlockhash();
    transaction.recentBlockhash = latestBlockhash.blockhash;

    transaction.sign(signer);

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
        [Buffer.from(vault_id)],
        programId
    );

    console.log("Vault PDA is:", vault_pda.toBase58());

    const [treasury] = await PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

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
                pubkey: treasury,
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

export async function withdraw(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vault_id: string,
    user_pubkey: string,
    amount: number,
) {

    // Log the input parameters
    console.log('Received withdraw parameters:', { vault_id, user_pubkey, amount });

    // Validate input parameters
    if (!vault_id || !user_pubkey) {
        throw new Error('vault_id and user_pubkey must be defined');
    }

    let buffer = Buffer.alloc(1000);
    const vault = vault_id.slice(0, 32); // Truncate to 32 bytes
    const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes
    vaultInstructionLayout.encode(
        {
            variant: 2,
            vault_id: vault,
            user_pubkey: pubKey,
            amount: amount,
            fund_status: FundStatus.Withdrawn,
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
        [Buffer.from(vault_id)],
        programId
    );

    console.log("Vault PDA is:", vault_pda.toBase58());

    const [treasury] = await PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

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
                pubkey: treasury,
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



