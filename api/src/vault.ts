import { AccountMeta, ComputeBudgetProgram, Connection, Keypair, PublicKey, SystemProgram, TransactionInstruction } from "@solana/web3.js";
import { BotStatus, getTreasuryPDA, getVaultDepositorPDA, getVaultPDA as getVaultPDA, FundStatus } from "./util";
import { DRIFT_PROGRAM, getDriftDepositKeys, getDriftManagerDepositKeys, getDriftManagerWithdrawKeys, getDriftUser, getDriftWithdrawKeys, getInitializeDriftKeys } from "./drift";
import { createInitializeAccountInstruction, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID, TokenInstruction } from "@solana/spl-token"
import { versionedTransactionSenderAndConfirmationWaiter } from "./utils/txns-sender";
import { VersionedTransaction } from "@solana/web3.js";
import { TransactionMessage } from "@solana/web3.js";
import { getSignature } from "./utils/get-signature";
import { handleTransactionResponse } from "./utils/handle-txn";
import BN from "bn.js";
import { depositInstuctionLayout, baseVaultInstuctionLayout, requestWithdrawInstuctionLayout, updateDelegateInstuctionLayout } from "./utils/layouts";
import { getDriftStateAccountPublicKey } from "@drift-labs/sdk";

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
    const data = baseVaultInstuctionLayout.decode(accountInfo.data);
    // Convert the user_pubkey from bytes to a PublicKey string
    console.log(JSON.stringify(data, null, 2));
}

export async function initializeDriftWithBulk(
    connection: Connection,
    manager: Keypair,
    programId: PublicKey,
    mint: PublicKey,
    vault_name: string,
    lock_in_period: number,
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
    const lock_in_period_bn = new BN(lock_in_period);
    const redeem_period_bn = new BN(redeem_period);
    const max_tokens_bn = new BN(max_tokens);
    const hurdle_rate_bn = new BN(hurdle_rate);

    baseVaultInstuctionLayout.encode(
        {
            variant: 0,
            name,
            lock_in_period: lock_in_period_bn,
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

    buffer = buffer.subarray(0, baseVaultInstuctionLayout.getSpan(buffer));


    const vault = getVaultPDA(vault_name, programId);

    console.log("Vault PDA is:", vault.toBase58());

    const treasury = getTreasuryPDA(vault_name, programId);

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
    baseVaultInstuctionLayout.encode(
        {
            variant: 1
        },
        buffer
    );

    buffer = buffer.subarray(0, baseVaultInstuctionLayout.getSpan(buffer));


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
            variant: 2,
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

export async function requestWithdraw(
    connection: Connection,
    authority: Keypair,
    programId: PublicKey,
    vault_name: string,
    amount: number,
    spotMarket: PublicKey,
    oracle: PublicKey,
) {
    // Log the input parameters
    console.log('Received deposit parameters:', { vault_name, amount });

    let buffer = Buffer.alloc(1000);

    // Assuming `amount` is a number
    const amountBN = new BN(amount);

    requestWithdrawInstuctionLayout.encode(
        {
            variant: 3,
            amount: amountBN,
        },
        buffer
    );

    buffer = buffer.subarray(0, requestWithdrawInstuctionLayout.getSpan(buffer));

    const vault = getVaultPDA(vault_name, programId)

    console.log("Vault PDA is:", vault.toBase58());

    const vaultDepositor = getVaultDepositorPDA(vault, authority.publicKey, programId);

    console.log("Vault Depositor is:", vaultDepositor.toBase58());

    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);

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
            pubkey: user,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: userStats,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: state,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: oracle,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: spotMarket,
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

export async function cancelWithdrawRequest(
    connection: Connection,
    authority: Keypair,
    programId: PublicKey,
    vault_name: string,
    spotMarket: PublicKey,
    oracle: PublicKey,
) {
    // Log the input parameters
    console.log('Received deposit parameters:', { vault_name });

    let buffer = Buffer.alloc(1000);

    baseVaultInstuctionLayout.encode(
        {
            variant: 4
        },
        buffer
    );

    buffer = buffer.subarray(0, baseVaultInstuctionLayout.getSpan(buffer));

    const vault = getVaultPDA(vault_name, programId)

    console.log("Vault PDA is:", vault.toBase58());

    const vaultDepositor = getVaultDepositorPDA(vault, authority.publicKey, programId);

    console.log("Vault Depositor is:", vaultDepositor.toBase58());

    const [user, userStats] = getDriftUser(vault);
    const state = await getDriftStateAccountPublicKey(DRIFT_PROGRAM);

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
            pubkey: user,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: userStats,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: state,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: oracle,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: spotMarket,
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

export async function withdraw(
    connection: Connection,
    authority: Keypair,
    programId: PublicKey,
    vault_name: string,
    spotMarketVault: PublicKey,
    spotMarket: PublicKey,
    oracle: PublicKey,
    mint: PublicKey,
) {

    // Log the input parameters
    console.log('Received withdraw parameters:', { spotMarket, spotMarketVault, oracle, mint });

    let buffer = Buffer.alloc(1000);
    baseVaultInstuctionLayout.encode(
        {
            variant: 5
        },
        buffer
    );

    buffer = buffer.subarray(0, baseVaultInstuctionLayout.getSpan(buffer));

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

    const driftKeys = await getDriftWithdrawKeys(connection, authority, programId, userTokenAccount, treasuryTokenAccount, vault_name, spotMarket, spotMarketVault, oracle, mint);

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
            variant: 6,
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

export async function managerDeposit(
    connection: Connection,
    manager: Keypair,
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
            variant: 7,
            name: vault_name.slice(0, 32),
            amount: amountBN,
        },
        buffer
    );

    buffer = buffer.subarray(0, depositInstuctionLayout.getSpan(buffer));

    const vault = getVaultPDA(vault_name, programId)

    console.log("Vault PDA is:", vault.toBase58());


    const managerTokenAccount = (await connection.getTokenAccountsByOwner(manager.publicKey, {
        mint: mint
    })).value[0].pubkey;

    console.log("Manager Token account:", managerTokenAccount.toString());

    const driftKeys = await getDriftManagerDepositKeys(connection, manager, programId, managerTokenAccount, vault_name, spotMarket, spotMarketVault, oracle, mint);

    const keys: AccountMeta[] = [
        {
            pubkey: manager.publicKey,
            isSigner: true,
            isWritable: true,
        },
        {
            pubkey: vault,
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
            payerKey: manager.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, instruction],
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

export async function managerWithdraw(
    connection: Connection,
    manager: Keypair,
    programId: PublicKey,
    vault_name: string,
    amount: number,
    spotMarketVault: PublicKey,
    spotMarket: PublicKey,
    oracle: PublicKey,
    mint: PublicKey,
) {

    // Log the input parameters
    console.log('Received manager withdraw parameters:', { spotMarket, spotMarketVault, oracle, mint });

    // Assuming `amount` is a number
    const amountBN = new BN(amount);

    let buffer = Buffer.alloc(1000);
    requestWithdrawInstuctionLayout.encode(
        {
            variant: 8,
            amount: amountBN,
        },
        buffer
    );

    buffer = buffer.subarray(0, requestWithdrawInstuctionLayout.getSpan(buffer));

    const vault = getVaultPDA(vault_name, programId)

    console.log("Vault PDA is:", vault.toBase58());

    const managerTokenAccount = (await connection.getTokenAccountsByOwner(manager.publicKey, {
        mint: mint
    })).value[0].pubkey;

    console.log("User Token account:", managerTokenAccount.toString());

    const driftKeys = await getDriftManagerWithdrawKeys(connection, manager, programId, managerTokenAccount, vault_name, spotMarket, spotMarketVault, oracle, mint);

    const keys: AccountMeta[] = [
        {
            pubkey: manager.publicKey,
            isSigner: true,
            isWritable: true,
        },
        {
            pubkey: vault,
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
            payerKey: manager.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, instruction],
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

export async function managerCollectFees(
    connection: Connection,
    manager: Keypair,
    programId: PublicKey,
    vault_name: string,
    amount: number,
    mint: PublicKey,
) {

    // Log the input parameters
    console.log('Received Collect Fees parameters:', { vault_name, amount, mint });

    // Assuming `amount` is a number
    const amountBN = new BN(amount);

    let buffer = Buffer.alloc(1000);
    requestWithdrawInstuctionLayout.encode(
        {
            variant: 9,
            amount: amountBN,
        },
        buffer
    );

    buffer = buffer.subarray(0, requestWithdrawInstuctionLayout.getSpan(buffer));

    const vault = getVaultPDA(vault_name, programId)

    console.log("Vault PDA is:", vault.toBase58());

    const managerTokenAccount = (await connection.getTokenAccountsByOwner(manager.publicKey, {
        mint: mint
    })).value[0].pubkey;

    console.log("Manager Token account:", managerTokenAccount.toString());

    const treasury = getTreasuryPDA(vault_name, programId);

    console.log("Treasury PDA is:", treasury.toBase58());

    const treasuryTokenAccount = (await getOrCreateAssociatedTokenAccount(
        connection,
        manager,
        mint,
        treasury,
        true
    )).address;

    const keys: AccountMeta[] = [
        {
            pubkey: manager.publicKey,
            isSigner: true,
            isWritable: true,
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
            pubkey: managerTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: treasuryTokenAccount,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: mint,
            isSigner: false,
            isWritable: true,
        },
        {
            pubkey: TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
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
            payerKey: manager.publicKey,
            recentBlockhash: blockhashResult.blockhash,
            instructions: [computeBudgetInstruction, computePriceInstruction, instruction],
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

export async function updateVault(
    connection: Connection,
    manager: Keypair,
    programId: PublicKey,
    mint: PublicKey,
    vault_name: string,
    lock_in_period: number,
    redeem_period: number,
    max_tokens: number,
    management_fee: number,
    min_deposit_amount: number,
    profit_share: number,
    hurdle_rate: number,
    permissioned: boolean,
) {

    // Log the input parameters
    console.log('Received init drift parameters:', { vault_name });

    let buffer = Buffer.alloc(1000);
    const name = vault_name.slice(0, 32); // Truncate to 32 bytes
    const management_fee_bn = new BN(management_fee);
    const min_deposit_amount_bn = new BN(min_deposit_amount);
    const profit_share_bn = new BN(profit_share);
    const lock_in_period_bn = new BN(lock_in_period);
    const redeem_period_bn = new BN(redeem_period);
    const max_tokens_bn = new BN(max_tokens);
    const hurdle_rate_bn = new BN(hurdle_rate);

    baseVaultInstuctionLayout.encode(
        {
            variant: 10,
            name,
            lock_in_period: lock_in_period_bn,
            redeem_period: redeem_period_bn,
            max_tokens: max_tokens_bn,
            management_fee: management_fee_bn,
            min_deposit_amount: min_deposit_amount_bn,
            profit_share: profit_share_bn,
            hurdle_rate: hurdle_rate_bn,
            permissioned,
        },
        buffer
    );

    buffer = buffer.subarray(0, baseVaultInstuctionLayout.getSpan(buffer));


    const vault = getVaultPDA(vault_name, programId);

    console.log("Vault PDA is:", vault.toBase58());

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
        }
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

export async function resetDelegate(
    connection: Connection,
    manager: Keypair,
    programId: PublicKey,
    vault_name: string,
) {

    // Log the input parameters
    console.log('Received resetDelegate parameters:', { vault_name });

    let buffer = Buffer.alloc(1000);
    baseVaultInstuctionLayout.encode(
        {
            variant: 11
        },
        buffer
    );

    buffer = buffer.subarray(0, baseVaultInstuctionLayout.getSpan(buffer));


    const [vault] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), Buffer.from(vault_name)],
        programId
    );

    console.log("Vault PDA is:", vault.toBase58());

    const [user] = getDriftUser(vault);

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