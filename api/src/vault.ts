import { AccountMeta, ComputeBudgetProgram, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { BotStatus, getTreasuryPDA, getVaultDepositorPDA, getVaultPDA as getVaultPDA, FundStatus } from "./util";
import { DRIFT_PROGRAM, getDriftDepositKeys, getDriftUser, getDriftWithdrawKeys, getInitializeDriftKeys } from "./drift";
import { createInitializeAccountInstruction, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID, TokenInstruction } from "@solana/spl-token"
import { versionedTransactionSenderAndConfirmationWaiter } from "./utils/txns-sender";
import { VersionedTransaction } from "@solana/web3.js";
import { TransactionMessage } from "@solana/web3.js";
import { getSignature } from "./utils/get-signature";
import { handleTransactionResponse } from "./utils/handle-txn";
import BN from "bn.js";
import { depositInstuctionLayout, initVaultInstuctionLayout, updateDelegateInstuctionLayout, vaultInstructionLayout } from "./utils/layouts";

const computeBudgetInstruction =
    ComputeBudgetProgram.setComputeUnitLimit({
        units: 4_000_000,
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
    mint: PublicKey,
    vault_name: string,
    redeem_period: number,
    max_tokens: number,
    management_fee: number,
    min_deposit_amount: number,
    profit_share: number,
    hurdle_rate: number,
    spot_market_index: number,
    permissioned: boolean,
) {

    // Log the input parameters
    console.log('Received init drift parameters:', { vault_name });

    let buffer = Buffer.alloc(1000);
    const name = vault_name.slice(0, 32); // Truncate to 32 bytes
    const management_fee_bn = new BN(management_fee);
    const min_deposit_amount_bn = new BN(min_deposit_amount);
    const profit_share_bn = new BN(profit_share);
    const redeem_period_bn = new BN(redeem_period);
    const max_tokens_bn = new BN(max_tokens);
    const hurdle_rate_bn = new BN(hurdle_rate);

    initVaultInstuctionLayout.encode(
        {
            variant: 5,
            name,
            redeem_period: redeem_period_bn,
            max_tokens: max_tokens_bn,
            management_fee: management_fee_bn,
            min_deposit_amount: min_deposit_amount_bn,
            profit_share: profit_share_bn,
            hurdle_rate: hurdle_rate_bn,
            spot_market_index,
            permissioned,
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

    const treasuryTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        manager,
        mint,
        treasury,
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
        {
            pubkey: treasuryTokenAccount,
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

export async function initializeVaultDepositor(
    connection: Connection,
    authority: Keypair,
    programId: PublicKey,
    vault_name: string,
) {

    // Log the input parameters
    console.log('Received initializeVaultDepositor parameters:', { vault_name });

    let buffer = Buffer.alloc(1000);
    initVaultInstuctionLayout.encode(
        {
            variant: 6
        },
        buffer
    );

    buffer = buffer.subarray(0, initVaultInstuctionLayout.getSpan(buffer));


    const [vault] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), Buffer.from(vault_name)],
        programId
    );

    console.log("Vault PDA is:", vault.toBase58());

    const [vault_depositor] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault_depositor"), vault.toBuffer(), authority.publicKey.toBuffer()],
        programId
    );

    console.log("Vault Depositor is:", vault_depositor.toBase58());


    const keys: AccountMeta[] = [
        {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vault_depositor,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: authority.publicKey,
            isSigner: true,
            isWritable: false,
        },
        {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
        },
    ];

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
            payerKey: authority.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, driftIx],
        }).compileToV0Message()
    );

    transaction.sign([authority]);

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
    authority: Keypair,
    programId: PublicKey,
    vault_name: string,
    amount: number,
    spotMarketVault: PublicKey,
    spotMarket: PublicKey,
    oracle: PublicKey,
    mint: PublicKey,
) {
    // Log the input parameters
    console.log('Received deposit parameters:', { vault_name, amount, spotMarket: spotMarket.toString(), spotMarketVault: spotMarketVault.toString(), oracle: oracle.toString(), mint: mint.toString() });


    let buffer = Buffer.alloc(1000);

    // Assuming `amount` is a number
    const amountBN = new BN(amount);

    depositInstuctionLayout.encode(
        {
            variant: 1,
            name: vault_name.slice(0, 32),
            amount: amountBN,
        },
        buffer
    );

    buffer = buffer.subarray(0, depositInstuctionLayout.getSpan(buffer));

    const vault = getVaultPDA(vault_name, programId)

    console.log("Vault PDA is:", vault.toBase58());

    const vaultDepositor = getVaultDepositorPDA(vault, authority.publicKey, programId);

    console.log("Vault Depositor is:", vaultDepositor.toBase58());

    const treasury = getTreasuryPDA(vault_name, programId);

    console.log("Treasury PDA is:", treasury.toBase58());

    const userTokenAccount = (await connection.getTokenAccountsByOwner(authority.publicKey, {
        mint: mint
    })).value[0].pubkey;

    console.log("User Token account:", userTokenAccount.toString());

    const treasuryTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        authority,
        mint,
        treasury,
        true
    )).address;

    console.log("Treasury Token account:", treasuryTokenAccount.toString());

    const driftKeys = await getDriftDepositKeys(connection, authority, programId, userTokenAccount, treasuryTokenAccount, vault_name, spotMarket, spotMarketVault, oracle, mint);

    const keys: AccountMeta[] = [
        {
            pubkey: vault,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: vaultDepositor,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: authority.publicKey,
            isSigner: true,
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
            payerKey: authority.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, instruction],
        }).compileToV0Message()
    );

    transaction.sign([authority]);

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

export async function deposit_old(
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

    const vault_pda = getVaultPDA(vault_id, programId);

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

    const vault_pda = getVaultPDA(vault_id, programId);

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
    vault_name: string,
    delegate: string,
    sub_account: number,
) {

    // Log the input parameters
    console.log('Received update delegate parameters:', { vault_name: vault_name, delegate, sub_account });

    console.log(new PublicKey(delegate).toString())
    let buffer = Buffer.alloc(1000);
    const vault_slice = vault_name.slice(0, 32); // Truncate to 32 bytes
    const delegate_slice = delegate.slice(0, 32); // Truncate to 32 bytes
    const sub_account_bn = new BN(0);

    updateDelegateInstuctionLayout.encode(
        {
            variant: 4,
            name: vault_name,
            delegate: delegate,
            sub_account: sub_account_bn,
        },
        buffer
    );

    buffer = buffer.subarray(0, updateDelegateInstuctionLayout.getSpan(buffer));

    const vault_pda = getVaultPDA(vault_name, programId);

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