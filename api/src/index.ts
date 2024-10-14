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
import bs58 from "bs58";
import { addUserInfo, readUserInfo, updateUserInfo } from './pda';

dotenv.config();

const app = express();
app.use(express.json());
// Enable CORS
app.use(cors());

const BULK_PROGRAM_ID = '8SyXDExbLvDU6Ny7HpRNZB4YKNRg65q9gHbPvhq8JHwP'
const connection = new Connection(process.env.RPC_URL || '', "confirmed");
const tritonConnection = new Connection(process.env.TRITON_PRO_RPC || '', "confirmed");


app.post('/deposit', async (req, res) => {
    try {
        const { user_pubkey, amount, signature } = req.body;

        console.log("Values Received")
        console.log(user_pubkey, amount, signature)

        //Confirm txn before going forward
        // const blockhash = await tritonConnection.getLatestBlockhash();
        // await tritonConnection.confirmTransaction( signature, 'confirmed');

        const roundedAmount = Number(Number(amount).toFixed(6));

        console.log("Values after modification")
        console.log(user_pubkey, roundedAmount, signature)

        const signer = Keypair.fromSecretKey(bs58.decode(process.env.PRIVATE_KEY || ''));

        const userInfoProgramId = new PublicKey(
            BULK_PROGRAM_ID
        );

        console.log('Reading User Info')
        const response = await readUserInfo(signer, userInfoProgramId, connection, user_pubkey);

        if (!response) {
            console.log('Adding new user Info')
            await addUserInfo(signer, userInfoProgramId, connection, user_pubkey, roundedAmount);
            console.log("After Adding user Info");
            await readUserInfo(signer, userInfoProgramId, connection, user_pubkey);
        } else {
            console.log('Updating user Info')
            await updateUserInfo(signer, userInfoProgramId, connection, user_pubkey, response.amount + roundedAmount);
            console.log("After Updating user Info");
            await readUserInfo(signer, userInfoProgramId, connection, user_pubkey);
        }

        //Trigger the deposit keeper bot
        const result = await fetch('http://72.46.84.23:4000/collateral', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ amount: amount }),
        });

        if (!result.ok) {
            throw new Error(`HTTP error! depositing to keeper bot status: ${result.status}`);
        }

        res.status(200).send('User info added/updated successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error depositing');
    }
});

app.post('/addUserInfo', async (req, res) => {
    try {
        const { user_pubkey, amount, fund_status, bot_status } = req.body;
        const signer = await initializeKeypair(connection, {
            envVariableName: "PRIVATE_KEY",
        });
        const userInfoProgramId = new PublicKey(
            BULK_PROGRAM_ID
        );

        await addUserInfo(signer, userInfoProgramId, connection, user_pubkey, amount);
        res.status(200).send('User info added successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error adding user info');
    }
});

app.post('/updateUserInfo', async (req, res) => {
    try {
        const { user_pubkey, amount, fund_status, bot_status } = req.body;
        const signer = await initializeKeypair(connection, {
            envVariableName: "PRIVATE_KEY",
        });
        const userInfoProgramId = new PublicKey(
           BULK_PROGRAM_ID
        );

        await updateUserInfo(signer, userInfoProgramId, connection, user_pubkey, amount);
        res.status(200).send('User info updated successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error updating user info');
    }
});

const PORT = process.env.PORT || 4001;
app.listen(PORT, () => {
    console.log(`Server is running on port ${PORT}`);
});
