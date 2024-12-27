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
import { cancelWithdrawRequest, deposit, initializeDriftWithBulk, initializeVaultDepositor, managerDeposit, requestWithdraw, updateDelegate, withdraw } from './vault';
import { getTokenBalance } from './utils/get-balance';

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


app.post('/init-drift-bulk', async (req, res) => {
    try {
        const { name } = req.body;

        const manager = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        console.log(`Signer: ${manager.publicKey}`)

        await initializeDriftWithBulk(connection, manager, BULK_PROGRAM_ID, USDC_MINT_LOCAL, name, 5 * 60, 5 * 60, 1000 * 1_000_000, 10000, 1_000_000, 10_000, 0, 0, false); //1% fees 1% profit share
        res.status(200).send('Initialized Vault with bulk successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error initializing Vault with bulk ');
    }
});

app.post('/deposit-usdc', async (req, res) => {
    try {
        const { vault_name, amount } = req.body;
        const user = await initializeKeypair(connection, {
            airdropAmount: 2 * LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY_USER",
        });

        console.log("before deposit");
        const usdcBalance = await getTokenBalance(connection, user.publicKey.toString(), USDC_MINT_LOCAL.toString());
        console.log(usdcBalance);

        await deposit(connection, user, BULK_PROGRAM_ID, vault_name, amount, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, SPOT_MARKET_USDC, USDC_MINT_LOCAL);

        console.log("after deposit")
        const newUsdcBalance = await getTokenBalance(connection, user.publicKey.toString(), USDC_MINT_LOCAL.toString());
        console.log(newUsdcBalance);
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

app.post('/withdraw-usdc', async (req, res) => {
    try {
        const { vault_name } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: 2 * LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY_USER",
        });

        console.log("before withdraw")
        console.log(await connection.getBalance(signer.publicKey))

        await withdraw(connection, signer, BULK_PROGRAM_ID, vault_name, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, SPOT_MARKET_USDC, USDC_MINT_LOCAL);

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



const PORT = process.env.PORT || 4001;
app.listen(PORT, async () => {
    console.log(`Server is running on port ${PORT}`);
    console.log(`Restart cmd: cd api && PROGRAM_ID=${BULK_PROGRAM_ID.toString()} npm run local`);
    console.log(`BULK Vault Program Id: ${BULK_PROGRAM_ID.toString()}`);

    const manager = await initializeKeypair(connection, {
        airdropAmount: 2 * LAMPORTS_PER_SOL,
        envVariableName: "PRIVATE_KEY",
    });


    const user = await initializeKeypair(connection, {
        airdropAmount: 2 * LAMPORTS_PER_SOL,
        envVariableName: "PRIVATE_KEY_USER",
    });

    const vault_name = 'bulk1';

    console.log('Admin SIGNER', manager.publicKey.toString());
    console.log('User SIGNER', user.publicKey.toString());

    // await initializeDriftWithBulk(connection, manager, BULK_PROGRAM_ID, USDC_MINT_LOCAL, vault_name, 1 * 30, 1 * 30, 1000 * 1_000_000, 10000, 1_000_000, 10_000, 0, 0, false); //1% fees 1% profit share

    // await initializeVaultDepositor(connection, user, BULK_PROGRAM_ID, vault_name)

     await deposit(connection, user, BULK_PROGRAM_ID, vault_name, 1000000, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, SPOT_MARKET_USDC, USDC_MINT_LOCAL);

    // await updateDelegate(connection, admin, BULK_PROGRAM_ID, vault_name, user.publicKey.toString(), 0)

    // await requestWithdraw(connection, user, BULK_PROGRAM_ID, vault_name, 900000, ORACLE_USDC, SPOT_MARKET_USDC);

    // await cancelWithdrawRequest(connection, user, BULK_PROGRAM_ID, vault_name, ORACLE_USDC, SPOT_MARKET_USDC);

    // await withdraw(connection, user, BULK_PROGRAM_ID, vault_name, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, SPOT_MARKET_USDC,  USDC_MINT_LOCAL);

    // await managerDeposit(connection, manager, BULK_PROGRAM_ID, vault_name, 1000000, SPOT_MARKET_VAULT_USDC, ORACLE_USDC, SPOT_MARKET_USDC, USDC_MINT_LOCAL);
});

