// Client
import express from 'express';
import {
    Connection,
    PublicKey,
    LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import dotenv from "dotenv";
import {
    initializeKeypair,
} from "@solana-developers/helpers";
import cors from 'cors';
import { deposit as deposit, initializeDrift, initializeVault, readPdaInfo, updateUserInfo, withdraw } from './pda';
import { Keypair } from '@solana/web3.js';
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes/index';
import { getTokenBalance } from './utils/get-balance';


dotenv.config();

const app = express();
app.use(express.json());
// Enable CORS
app.use(cors());

const connection = new Connection(process.env.RPC_URL || '', "confirmed");
const tritonConnection = new Connection(process.env.TRITON_PRO_RPC || '', "confirmed");

const USDC_MINT = new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v');
const SPOT_MARKET_VAULT = new PublicKey('GXWqPpjQpdz7KZw9p7f5PX2eGxHAhvpNXiviFkAB8zXg');
const BULK_PROGRAM_ID = new PublicKey('5NDdLPr4hLbL2S5BQtGFqFYRXAuq3x72CLKncEmFMbDQ');

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

app.post('/deposit', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || ''));

        console.log("before deposit");
        const usdcBalance = await getTokenBalance(connection, signer.publicKey.toString(), USDC_MINT.toString());
        console.log(usdcBalance);

        await deposit(connection, signer, BULK_PROGRAM_ID, vault_id, user_pubkey, amount, SPOT_MARKET_VAULT, USDC_MINT);

        console.log("after deposit")
        const newUsdcBalance = await getTokenBalance(connection, signer.publicKey.toString(), USDC_MINT.toString());
        console.log(newUsdcBalance);
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

app.post('/withdraw', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || ''));

        console.log("before withdraw")
        console.log(await connection.getBalance(signer.publicKey))

        await withdraw(signer, BULK_PROGRAM_ID, connection, vault_id, user_pubkey, amount);

        console.log("after withdraw")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Withdraw successfully');
    } catch (error) {
        console.error('Error during withdraw:', error);
        res.status(500).send('Error during withdraw');
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

    console.log('Admin SIGNER', Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY || '')).publicKey.toString());
    console.log('User SIGNER', Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_USER || '')).publicKey.toString());
    console.log('Delegate SIGNER', Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY_DELEGATE || '')).publicKey.toString());

    // const usdcAccount = await connection.getTokenAccountsByOwner(signer.publicKey, {
    //     mint: USDC_MINT
    // });

    // console.log('USDC account', usdcAccount.value[0].pubkey.toString());

});

