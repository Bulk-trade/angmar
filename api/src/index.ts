// Client
import express from 'express';
import {
    Connection,
    PublicKey,
} from "@solana/web3.js";
import dotenv from "dotenv";
import cors from 'cors';
import { deposit as deposit, getVaultPda, initializeDrift, initializeVault, updateDelegate, updateUserInfo, withdraw } from './vault';
import { Keypair } from '@solana/web3.js';
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes/index';
import { getTokenBalance } from './utils/get-balance';
import { getOrCreateAssociatedTokenAccount } from '@solana/spl-token';
import { getOracleClient, getPythPullOraclePublicKey } from '@drift-labs/sdk';
import { OracleClientCache } from '@drift-labs/sdk/lib/node/oracles/oracleClientCache';

dotenv.config({ path: '.env.production' });

const app = express();
app.use(express.json());
// Enable CORS
app.use(cors());

const connection = new Connection(process.env.RPC_URL || '', "confirmed");

const BULK_PROGRAM_ID = new PublicKey('8dge6cap3vEmrG8QmUve9xiCvaMVCDXLaRJ6ZPZgs7v5');
const USDC_MINT = new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v');
const WSOL = new PublicKey('So11111111111111111111111111111111111111112');
const SPOT_MARKET_VAULT_USDC = new PublicKey('GXWqPpjQpdz7KZw9p7f5PX2eGxHAhvpNXiviFkAB8zXg');
const SPOT_MARKET_VAULT_WSOL = new PublicKey('DfYCNezifxAEsQbAJ1b3j6PX3JVBe8fu11KBhxsbw5d2');

const SPOT_MARKET_USDC = new PublicKey('6gMq3mRCKf8aP3ttTyYhuijVZ2LGi14oDsBbkgubfLB3');
const SPOT_MARKET_WSOL = new PublicKey('3x85u7SWkmmr7YQGYhtjARgxwegTLJgkSLRprfXod6rh');

const ORACLE_USDC = new PublicKey('En8hkHLkRe9d9DraYmBTrus518BvmVH448YcvmrFM6Ce');
const ORACLE_WSOL = new PublicKey('BAtFj4kQttZRVep3UZS2aZRDixkGYgWsbqTBVDbnSsPF');

app.post('/initVault', async (req, res) => {
    try {
        const { vault_id } = req.body;

        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY || ''));

        console.log(`Signer: ${signer.publicKey}`);

        await initializeVault(connection, signer, BULK_PROGRAM_ID, vault_id);
        res.status(200).send('Initialized Vault successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error initializing Vault');
    }
});

app.post('/initDrift', async (req, res) => {
    try {
        const { vault_id } = req.body;

        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY || ''));

        console.log(`Signer: ${signer.publicKey}`);

        await initializeDrift(signer, BULK_PROGRAM_ID, connection, vault_id);
        res.status(200).send('Initialized Vault successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error initializing Vault');
    }
});

app.post('/deposit-usdc', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || ''));

        console.log("before deposit");
        const usdcBalance = await getTokenBalance(connection, signer.publicKey.toString(), USDC_MINT.toString());
        console.log(usdcBalance);

        await deposit(connection, signer, BULK_PROGRAM_ID, vault_id, user_pubkey, amount, 0, SPOT_MARKET_USDC, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, USDC_MINT);

        console.log("after deposit")
        const newUsdcBalance = await getTokenBalance(connection, signer.publicKey.toString(), USDC_MINT.toString());
        console.log(newUsdcBalance);
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

app.post('/deposit-wsol', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;

        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || ''));

        console.log("before deposit")
        console.log(await connection.getBalance(signer.publicKey))

        await deposit(connection, signer, BULK_PROGRAM_ID, vault_id, user_pubkey, amount, 1, SPOT_MARKET_WSOL, SPOT_MARKET_VAULT_WSOL, ORACLE_WSOL, WSOL);

        console.log("after deposit")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

app.post('/withdraw-usdc', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || ''));

        console.log("before withdraw")
        console.log(await connection.getBalance(signer.publicKey))

        await withdraw(connection, signer, BULK_PROGRAM_ID, vault_id, user_pubkey, amount, 0, SPOT_MARKET_USDC, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, USDC_MINT);

        console.log("after withdraw")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Withdraw successfully');
    } catch (error) {
        console.error('Error during withdraw:', error);
        res.status(500).send('Error during withdraw');
    }
});

app.post('/update-delegate', async (req, res) => {
    try {
        const { vault_id, delegate, sub_account } = req.body;
        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || ''));

        await updateDelegate(connection, signer, BULK_PROGRAM_ID, vault_id, delegate, sub_account);

        res.status(200).send('Update successfully');
    } catch (error) {
        console.error('Error during update:', error);
        res.status(500).send('Error during update');
    }
});

app.post('/updateUserInfo', async (req, res) => {
    try {
        const { user_pubkey, amount, fund_status, bot_status } = req.body;
        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY || ''));

        await updateUserInfo(signer, BULK_PROGRAM_ID, connection, user_pubkey, amount);

        console.log("after withdraw")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});



const PORT = process.env.PORT || 4001;
app.listen(PORT, async () => {
    console.log(`Server is running on port ${PORT}`);
    console.log(`BULK Vault Program Id: ${BULK_PROGRAM_ID.toString()}`);

    const admin = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY || ''));
    const user = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || ''));

    const vault_id = "bulk_vault";

    console.log('Admin SIGNER', admin.publicKey.toString());
    console.log('User SIGNER', user.publicKey.toString());
    console.log('Delegate SIGNER', Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_DELEGATE || '')).publicKey.toString());

    const usdcBalance = await getTokenBalance(connection, user.publicKey.toString(), USDC_MINT.toString());
    console.log(usdcBalance);

    const vault = getVaultPda(BULK_PROGRAM_ID, vault_id);

    console.log("Vault PDA is:", vault.toBase58());

    const vaultTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        admin,
        USDC_MINT,
        vault,
        true
    );

    console.log("Vault Token account:", vaultTokenAccount.address.toString());


    const [treasury] = await PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vault_id)],
        BULK_PROGRAM_ID
    );

    console.log("Treasury PDA is:", treasury.toBase58());

    const treasuryTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        admin,
        USDC_MINT,
        treasury,
        true
    );

    console.log("Treasury Token account:", treasuryTokenAccount.address.toString());

});

