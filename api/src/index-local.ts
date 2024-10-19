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
import { deposit as deposit, initializeVault, readUserInfo, updateUserInfo } from './pda';

dotenv.config();

const app = express();
app.use(express.json());
// Enable CORS
app.use(cors());

const BULK_PROGRAM_ID = 'ASyAtnUUtXao8sfbGHQu63cqMgbWWFnzdQ15fjhsxqAU'
const connection = new Connection("http://localhost:8899", "confirmed");


// app.post('/deposit', async (req, res) => {
//     try {
//         const { user_pubkey, amount, signature } = req.body;

//         console.log(user_pubkey, amount, signature)

//         //Confirm txn before going forward
//         const blockhash = await connection.getLatestBlockhash();
//         await connection.confirmTransaction({
//             blockhash: blockhash.blockhash,
//             lastValidBlockHeight: blockhash.lastValidBlockHeight,
//             signature
//         });

//         const roundedAmount = Number(Number(amount).toFixed(6));

//         console.log(user_pubkey, roundedAmount, signature)

//         const signer = await initializeKeypair(connection, {
//             airdropAmount: LAMPORTS_PER_SOL,
//             envVariableName: "PRIVATE_KEY",
//         });

//         const userInfoProgramId = new PublicKey(
//             BULK_PROGRAM_ID
//         );

//         const response = await readUserInfo(signer, userInfoProgramId, connection, user_pubkey);

//         if (!response) {
//             await addUserInfo(signer, userInfoProgramId, connection, user_pubkey, roundedAmount);
//         } else {
//             await updateUserInfo(signer, userInfoProgramId, connection, user_pubkey, response.amount + roundedAmount);
//             console.log("After Update");
//             await readUserInfo(signer, userInfoProgramId, connection, user_pubkey);
//         }

//         //Trigger the deposit keeper bot
//         const result = await fetch('http://72.46.84.23:4000/collateral', {
//             method: 'POST',
//             headers: {
//                 'Content-Type': 'application/json',
//             },
//             body: JSON.stringify({ amount: amount }),
//         });

//         if (!result.ok) {
//             throw new Error(`HTTP error! depositing to keeper bot status: ${result.status}`);
//         }

//         res.status(200).send('User info added successfully');
//     } catch (error) {
//         console.error(error);
//         res.status(500).send('Error depositing');
//     }
// });

app.post('/initVault', async (req, res) => {
    try {
        const { vault_id } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });
        const userInfoProgramId = new PublicKey(
            BULK_PROGRAM_ID
        );

        await initializeVault(signer, userInfoProgramId, connection, vault_id);
        res.status(200).send('Initialized Vault successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error adding user info');
    }
});

app.post('/deposit', async (req, res) => {
    try {
        const { vault_id, user_pubkey, amount } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        const userInfoProgramId = new PublicKey(
            BULK_PROGRAM_ID
        );

        await deposit(signer, userInfoProgramId, connection, vault_id, user_pubkey, amount);
        
        console.log("after deposit")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

app.post('/updateUserInfo', async (req, res) => {
    try {
        const { user_pubkey, amount, fund_status, bot_status } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });
        const userInfoProgramId = new PublicKey(
            BULK_PROGRAM_ID
        );

        await updateUserInfo(signer, userInfoProgramId, connection, user_pubkey, amount);
        
        console.log("after withdraw")
        console.log(await connection.getBalance(signer.publicKey))
        res.status(200).send('Deposited successfully');
    } catch (error) {
        console.error('Error during deposit:', error);
        res.status(500).send('Error during deposit');
    }
});

const PORT = process.env.PORT || 4001;
app.listen(PORT, () => {
    console.log(`Server is running on port ${PORT}`);
});