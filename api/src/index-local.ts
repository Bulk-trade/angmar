// Client
import express from 'express';
import {
    Keypair,
    Connection,
    PublicKey,
    LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import dotenv from "dotenv";
import {
    initializeKeypair,
} from "@solana-developers/helpers";
import cors from 'cors';
import { deposit as deposit, initializeVault, readPdaInfo, updateUserInfo, withdraw } from './pda';
import bs58 from "bs58";

dotenv.config();

const app = express();
app.use(express.json());
// Enable CORS
app.use(cors());

const BULK_PROGRAM_ID = process.env.PROGRAM_ID || '';
const connection = new Connection("http://localhost:8899", "confirmed");
const vaultProgramId = new PublicKey(
    BULK_PROGRAM_ID
);

app.post('/initVault', async (req, res) => {
    try {
        const { vault_id } = req.body;

        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        await initializeVault(signer, vaultProgramId, connection, vault_id);
        res.status(200).send('Initialized Vault successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error initializing Vault');
    }
});

app.post('/deposit', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY_USER",
        });

        console.log("before deposit")
        console.log(await connection.getBalance(signer.publicKey))

        await deposit(signer, vaultProgramId, connection, vault_id, user_pubkey, amount);

        console.log("after deposit")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

app.post('/withdraw', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY_USER",
        });

        console.log("before withdraw")
        console.log(await connection.getBalance(signer.publicKey))

        await withdraw(signer, vaultProgramId, connection, vault_id, user_pubkey, amount);

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
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        await updateUserInfo(signer, vaultProgramId, connection, user_pubkey, amount);

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
    console.log(`BULK Vault Program Id: ${vaultProgramId.toString()}`);

    // const signer = await initializeKeypair(connection, {
    //     airdropAmount: LAMPORTS_PER_SOL,
    //     envVariableName: "PRIVATE_KEY",
    // });

    //await readPdaInfo(signer, vaultProgramId, connection, 'sunit01')
});

