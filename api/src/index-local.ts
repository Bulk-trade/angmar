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
import { deposit as deposit, getVaultPda, initializeDrift, initializeVault, updateDelegate, updateUserInfo, withdraw } from './vault';

dotenv.config();

const app = express();
app.use(express.json());
// Enable CORS
app.use(cors());

const connection = new Connection("http://localhost:8899", "confirmed");
const BULK_PROGRAM_ID = new PublicKey(process.env.PROGRAM_ID || '');

const USDC_MINT_LOCAL = new PublicKey('Fgfq9JbxkvAXcuqW2BSHgRZHY9DeEc8vQzieL3QaBy8G');
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

        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        console.log(`Signer: ${signer.publicKey}`)

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

        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        console.log(`Signer: ${signer.publicKey}`)

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
        const signer = await initializeKeypair(connection, {
            airdropAmount: 2 * LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY_USER",
        });

        console.log("before deposit")
        console.log(await connection.getBalance(signer.publicKey))

        await deposit(connection, signer, BULK_PROGRAM_ID, vault_id, user_pubkey, amount, 0, SPOT_MARKET_USDC, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, USDC_MINT_LOCAL);
       
        console.log("after deposit")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

app.post('/deposit-wsol', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: 2 * LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY_USER",
        });

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
        const signer = await initializeKeypair(connection, {
            airdropAmount: 2 * LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY_USER",
        });

        console.log("before withdraw")
        console.log(await connection.getBalance(signer.publicKey))

        await withdraw(connection, signer, BULK_PROGRAM_ID, vault_id, user_pubkey, amount, 0, SPOT_MARKET_USDC, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, USDC_MINT_LOCAL);

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
        const signer = await initializeKeypair(connection, {
            airdropAmount: 2 * LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

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
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

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

    const signer = await initializeKeypair(connection, {
        airdropAmount: 2 * LAMPORTS_PER_SOL,
        envVariableName: "PRIVATE_KEY_USER",
    });

    console.log('SIGNER', signer.publicKey.toString());

    const usdcAccount = await connection.getTokenAccountsByOwner(signer.publicKey, {
        mint: USDC_MINT_LOCAL
    });

    console.log('USER USDC account', usdcAccount.value[0].pubkey.toString());

    // if (BULK_PROGRAM_ID) {
    //     const vault = getVaultPda(BULK_PROGRAM_ID, 'bulk_vault');
    //     const vaultTokenAccount = (await connection.getTokenAccountsByOwner(vault, {
    //         mint: USDC_MINT_LOCAL
    //     })).value[0].pubkey;

    //     console.log('Vault USDC account', vaultTokenAccount.toString());
    // } else {
    //     console.error('BULK_PROGRAM_ID is not set');
    // }
});

