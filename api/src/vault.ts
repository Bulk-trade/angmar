import { AccountMeta, ComputeBudgetProgram, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { BotStatus, FundStatus } from "./util";
import { DRIFT_PROGRAM, getDriftDepositKeys, getDriftUser, getDriftWithdrawKeys, getInitializeDriftKeys } from "./drift";
import { createInitializeAccountInstruction, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID, TokenInstruction } from "@solana/spl-token"
import { versionedTransactionSenderAndConfirmationWaiter } from "./utils/txns-sender";
import { VersionedTransaction } from "@solana/web3.js";
import { TransactionMessage } from "@solana/web3.js";
import { getSignature } from "./utils/get-signature";
import { handleTransactionResponse } from "./utils/handle-txn";
import BN from "bn.js";
import { initVaultInstuctionLayout, vaultInstructionLayout } from "./utils/layouts";

const computeBudgetInstruction =
    ComputeBudgetProgram.setComputeUnitLimit({
        units: 400_000,
    });

const computePriceInstruction =
    ComputeBudgetProgram.setComputeUnitPrice({
        microLamports: 500000,
    });

export async function readPdaInfo(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    name: string,
) {
    //const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes

    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), Buffer.from(name)],
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
    const data = initVaultInstuctionLayout.decode(accountInfo.data);
    // Convert the user_pubkey from bytes to a PublicKey string
    console.log(JSON.stringify(data, null, 2));
}

export async function initializeVault(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
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

    const Ix = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys,
    });

    // Get the latest blockhash for the transaction
    const blockhashResult = await connection.getLatestBlockhash({ commitment: "confirmed" });

    const transaction = new VersionedTransaction(
        new TransactionMessage({
            payerKey: signer.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [Ix],
        }).compileToV0Message()
    );

    transaction.sign([signer]);

    // Get the transaction signature
    const signature = getSignature(transaction);

    // Serialize the transaction and get the recent blockhash
    const serializedTransaction = transaction.serialize();

    const transactionResponse = await versionedTransactionSenderAndConfirmationWaiter({
        connection,
        serializedTransaction,
        blockhashWithExpiryBlockHeight: blockhashResult,
    });

    // Handle the transaction response
    handleTransactionResponse(transactionResponse, signature);
}

export async function initializeDrift(
    signer: Keypair,
    programId: PublicKey,
    connection: Connection,
    vault_id: string,
) {

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


    const [vault, _] = PublicKey.findProgramAddressSync(
        [Buffer.from(vault_id)],
        programId
    );

    console.log("Vault PDA is:", vault.toBase58());

    const [treasury] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

    const driftKeys = await getInitializeDriftKeys(signer.publicKey, programId, vault);

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

    const driftIx = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys,
    });

    // Get the latest blockhash for the transaction
    const blockhashResult = await connection.getLatestBlockhash({ commitment: "confirmed" });

    const transaction = new VersionedTransaction(
        new TransactionMessage({
            payerKey: signer.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, driftIx],
        }).compileToV0Message()
    );

    transaction.sign([signer]);

    // Get the transaction signature
    const signature = getSignature(transaction);

    // Serialize the transaction and get the recent blockhash
    const serializedTransaction = transaction.serialize();

    const transactionResponse = await versionedTransactionSenderAndConfirmationWaiter({
        connection,
        serializedTransaction,
        blockhashWithExpiryBlockHeight: blockhashResult,
    });

    // Handle the transaction response
    handleTransactionResponse(transactionResponse, signature);
}

export async function initializeDriftWithBulk(
    connection: Connection,
    manager: Keypair,
    programId: PublicKey,
    vault_name: string,
    mint: PublicKey,
) {

    // Log the input parameters
    console.log('Received init drift parameters:', { vault_name });

    let buffer = Buffer.alloc(2000);
    const name = vault_name.slice(0, 32); // Truncate to 32 bytes

    const management_fee = new BN(100000);

    const min_deposit_amount = new BN(1000);

    const profit_share = new BN(100000);

    initVaultInstuctionLayout.encode(
        {
            variant: 5,
            name,
            management_fee,
            min_deposit_amount,
            profit_share,
            spot_market_index: 0,
            permissioned: false,
        },
        buffer
    );

    buffer = buffer.subarray(0, initVaultInstuctionLayout.getSpan(buffer));


    const [vault] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), Buffer.from(vault_name)],
        programId
    );

    console.log("Vault PDA is:", vault.toBase58());

    const [treasury] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_name)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

    const vaultTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        manager,
        mint,
        vault,
        true
    )).address;

    const driftKeys = await getInitializeDriftKeys(manager.publicKey, programId, vault);

    const keys: AccountMeta[] = [
        {
            pubkey: manager.publicKey,
            isSigner: true,
            isWritable: false,
        },
        {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vaultTokenAccount,
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

    const driftIx = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys,
    });

    // Get the latest blockhash for the transaction
    const blockhashResult = await connection.getLatestBlockhash({ commitment: "confirmed" });

    const transaction = new VersionedTransaction(
        new TransactionMessage({
            payerKey: manager.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, driftIx],
        }).compileToV0Message()
    );

    transaction.sign([manager]);

    // Get the transaction signature
    const signature = getSignature(transaction);

    // Serialize the transaction and get the recent blockhash
    const serializedTransaction = transaction.serialize();

    const transactionResponse = await versionedTransactionSenderAndConfirmationWaiter({
        connection,
        serializedTransaction,
        blockhashWithExpiryBlockHeight: blockhashResult,
    });

    // Handle the transaction response
    handleTransactionResponse(transactionResponse, signature);
}

export async function deposit(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
    vault_id: string,
    user_pubkey: string,
    amount: number,
    marketIndex: number,
    spotMarket: PublicKey,
    spotMarketVault: PublicKey,
    oracle: PublicKey,
    mint: PublicKey,
) {
    // Log the input parameters
    console.log('Received deposit parameters:', { vault_id, user_pubkey, amount, marketIndex, spotMarket: spotMarket.toString(), spotMarketVault: spotMarketVault.toString(), oracle: oracle.toString(), mint: mint.toString() });

    // Validate input parameters
    if (!vault_id || !user_pubkey) {
        throw new Error('vault_id and user_pubkey must be defined');
    }

    let buffer = Buffer.alloc(1000);
    const vault = vault_id.slice(0, 32); // Truncate to 32 bytes
    const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes

    // Assuming `amount` is a number
    const amountBN = new BN(amount);

    vaultInstructionLayout.encode(
        {
            variant: 1,
            vault_id: vault,
            user_pubkey: pubKey,
            amount: amountBN,
            fund_status: FundStatus.Deposited,
            bot_status: BotStatus.Init,
            market_index: marketIndex,
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));

    const [user_info_pda] = PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(pubKey)],
        programId
    );

    console.log("User PDA is:", user_info_pda.toBase58());

    const vault_pda = getVaultPda(programId, vault_id);

    console.log("Vault PDA is:", vault_pda.toBase58());

    const [treasury] = await PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

    const userTokenAccount = (await connection.getTokenAccountsByOwner(signer.publicKey, {
        mint: mint
    })).value[0].pubkey;

    console.log("User Token account:", userTokenAccount.toString());

    const treasuryTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        signer,
        mint,
        treasury,
        true
    )).address;

    console.log("Treasury Token account:", treasuryTokenAccount.toString());

    const driftKeys = await getDriftDepositKeys(connection, signer, programId, userTokenAccount, treasuryTokenAccount, vault_id, spotMarket, spotMarketVault, oracle, mint);

    const keys: AccountMeta[] = [
        {
            pubkey: signer.publicKey,
            isSigner: true,
            isWritable: true,
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
    ];

    keys.push(...driftKeys);

    console.log(`Keys Length: ${keys.length}`);

    const instruction = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys
    });

    // Get the latest blockhash for the transaction
    const blockhashResult = await connection.getLatestBlockhash({ commitment: "confirmed" });

    const transaction = new VersionedTransaction(
        new TransactionMessage({
            payerKey: signer.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, instruction],
        }).compileToV0Message()
    );

    transaction.sign([signer]);

    // Get the transaction signature
    const signature = getSignature(transaction);

    // Serialize the transaction and get the recent blockhash
    const serializedTransaction = transaction.serialize();

    const transactionResponse = await versionedTransactionSenderAndConfirmationWaiter({
        connection,
        serializedTransaction,
        blockhashWithExpiryBlockHeight: blockhashResult,
    });

    // Handle the transaction response
    handleTransactionResponse(transactionResponse, signature);

    //const tx = await sendAndConfirmTransaction(connection, transaction, [signer], { skipPreflight: true });
}

export async function withdraw(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
    vault_id: string,
    user_pubkey: string,
    amount: number,
    marketIndex: number,
    spotMarket: PublicKey,
    spotMarketVault: PublicKey,
    oracle: PublicKey,
    mint: PublicKey,
) {

    // Log the input parameters
    console.log('Received withdraw parameters:', { vault_id, user_pubkey, amount, marketIndex, spotMarket, spotMarketVault, oracle, mint });

    // Validate input parameters
    if (!vault_id || !user_pubkey) {
        throw new Error('vault_id and user_pubkey must be defined');
    }

    let buffer = Buffer.alloc(1000);
    const vault = vault_id.slice(0, 32); // Truncate to 32 bytes
    const pubKey = user_pubkey.slice(0, 32); // Truncate to 32 bytes

    // Assuming `amount` is a number
    const amountBN = new BN(amount);
    vaultInstructionLayout.encode(
        {
            variant: 2,
            vault_id: vault,
            user_pubkey: pubKey,
            amount: amountBN,
            fund_status: FundStatus.Withdrawn,
            bot_status: BotStatus.Init,
            market_index: marketIndex,
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));

    const [user_info_pda] = PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Buffer.from(pubKey)],
        programId
    );

    console.log("User PDA is:", user_info_pda.toBase58());

    const vault_pda = getVaultPda(programId, vault_id);

    console.log("Vault PDA is:", vault_pda.toBase58());

    const [treasury] = await PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        programId
    );

    console.log("Treasury PDA is:", treasury.toBase58());

    const userTokenAccount = (await connection.getTokenAccountsByOwner(signer.publicKey, {
        mint: mint
    })).value[0].pubkey;

    console.log("User Token account:", userTokenAccount.toString());

    const treasuryTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        signer,
        mint,
        treasury,
        true
    )).address;

    console.log("Treasury Token account:", userTokenAccount.toString());

    const driftKeys = await getDriftWithdrawKeys(connection, signer, programId, userTokenAccount, treasuryTokenAccount, vault_id, spotMarket, spotMarketVault, oracle, mint);

    const keys: AccountMeta[] = [
        {
            pubkey: signer.publicKey,
            isSigner: true,
            isWritable: true,
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
    ];

    keys.push(...driftKeys);

    console.log(`Keys Length: ${keys.length}`);

    const instruction = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys
    });

    // Get the latest blockhash for the transaction
    const blockhashResult = await connection.getLatestBlockhash({ commitment: "confirmed" });

    const transaction = new VersionedTransaction(
        new TransactionMessage({
            payerKey: signer.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, instruction],
        }).compileToV0Message()
    );

    transaction.sign([signer]);

    // Get the transaction signature
    const signature = getSignature(transaction);

    // Serialize the transaction and get the recent blockhash
    const serializedTransaction = transaction.serialize();

    const transactionResponse = await versionedTransactionSenderAndConfirmationWaiter({
        connection,
        serializedTransaction,
        blockhashWithExpiryBlockHeight: blockhashResult,
    });

    // Handle the transaction response
    handleTransactionResponse(transactionResponse, signature);
}

export async function updateDelegate(
    connection: Connection,
    signer: Keypair,
    programId: PublicKey,
    vault_id: string,
    delegate: string,
    sub_account: number,
) {

    // Log the input parameters
    console.log('Received update delegate parameters:', { vault_id, delegate, sub_account });

    console.log(new PublicKey(delegate).toString())
    let buffer = Buffer.alloc(1000);
    const vault = vault_id.slice(0, 32); // Truncate to 32 bytes
    const delegate_key = delegate.slice(0, 32); // Truncate to 32 bytes

    const sub_account_bn = new BN(sub_account);
    vaultInstructionLayout.encode(
        {
            variant: 4,
            vault_id: vault,
            user_pubkey: '',
            amount: sub_account_bn,
            fund_status: FundStatus.Deposited,
            bot_status: BotStatus.Init,
            market_index: 0,
            delegate: delegate,
            sub_account: sub_account_bn
        },
        buffer
    );

    buffer = buffer.subarray(0, vaultInstructionLayout.getSpan(buffer));

    const vault_pda = getVaultPda(programId, vault_id);

    console.log("Vault PDA is:", vault_pda.toBase58());

    const [user] = getDriftUser(vault_pda);

    const keys: AccountMeta[] = [
        {
            pubkey: signer.publicKey,
            isSigner: true,
            isWritable: true,
        },
        {
            pubkey: vault_pda,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: DRIFT_PROGRAM,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: user,
            isSigner: false,
            isWritable: true,
        },
    ];

    console.log(`Keys Length: ${keys.length}`);

    const instruction = new TransactionInstruction({
        programId: programId,
        data: buffer,
        keys
    });

    // Get the latest blockhash for the transaction
    const blockhashResult = await connection.getLatestBlockhash({ commitment: "confirmed" });

    const transaction = new VersionedTransaction(
        new TransactionMessage({
            payerKey: signer.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, instruction],
        }).compileToV0Message()
    );

    transaction.sign([signer]);

    // Get the transaction signature
    const signature = getSignature(transaction);

    // Serialize the transaction and get the recent blockhash
    const serializedTransaction = transaction.serialize();

    const transactionResponse = await versionedTransactionSenderAndConfirmationWaiter({
        connection,
        serializedTransaction,
        blockhashWithExpiryBlockHeight: blockhashResult,
    });

    // Handle the transaction response
    handleTransactionResponse(transactionResponse, signature);
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

export function getVaultPda(programId: PublicKey, name: String) {
    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), Buffer.from(name)],
        programId
    );

    return pda;
}