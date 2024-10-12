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
import { addUserInfo, readUserInfo, updateUserInfo } from './pda';

dotenv.config();

const app = express();
app.use(express.json());
// Enable CORS
app.use(cors());



const connection = new Connection("http://localhost:8899", "confirmed");


app.post('/deposit', async (req, res) => {
    try {
        const { user_pubkey, amount, signature } = req.body;

        console.log(user_pubkey, amount, signature)

        //Confirm txn before going forward
        // const blockhash = await connection.getLatestBlockhash();
        // await connection.confirmTransaction({
        //     blockhash: blockhash.blockhash,
        //     lastValidBlockHeight: blockhash.lastValidBlockHeight,
        //     signature
        // });

        const roundedAmount = Number(Number(amount).toFixed(6));

        console.log(user_pubkey, roundedAmount, signature)

        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        const userInfoProgramId = new PublicKey(
            "98PdopvDo8HrWnahS1EhGK3BFSY3BXKFWDCt7EmzSs7P"
        );

        const response = await readUserInfo(signer, userInfoProgramId, connection, user_pubkey);

        if (!response) {
            await addUserInfo(signer, userInfoProgramId, connection, user_pubkey, roundedAmount);
        } else {
            await updateUserInfo(signer, userInfoProgramId, connection, user_pubkey, response.amount + roundedAmount);
            console.log("After Update");
            await readUserInfo(signer, userInfoProgramId, connection, user_pubkey);
        }

        //Trigger the deposit keeper bot
        const result = await fetch('http://localhost:4000/collateral', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ amount: amount }),
        });

        if (!result.ok) {
            throw new Error(`HTTP error! depositing to keeper bot status: ${result.status}`);
        }

        res.status(200).send('User info added successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error depositing');
    }
});

app.post('/addUserInfo', async (req, res) => {
    try {
        const { user_pubkey, amount, fund_status, bot_status } = req.body;
        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });
        const userInfoProgramId = new PublicKey(
            "98PdopvDo8HrWnahS1EhGK3BFSY3BXKFWDCt7EmzSs7P"
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
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });
        const userInfoProgramId = new PublicKey(
            "98PdopvDo8HrWnahS1EhGK3BFSY3BXKFWDCt7EmzSs7P"
        );

        await updateUserInfo(signer, userInfoProgramId, connection, user_pubkey, amount);
        res.status(200).send('User info updated successfully');
    } catch (error) {
        console.error(error);
        res.status(500).send('Error updating user info');
    }
});

const PORT = process.env.PORT || 8000;
app.listen(PORT, () => {
    console.log(`Server is running on port ${PORT}`);
});

(async () => {
    try {
        // const connection = new Connection(clusterApiUrl("testnet"));
        const connection = new Connection("http://localhost:8899", "confirmed");

        const signer = await initializeKeypair(connection, {
            airdropAmount: LAMPORTS_PER_SOL,
            envVariableName: "PRIVATE_KEY",
        });

        const userInfoProgramId = new PublicKey(
            "C2raSeo5Y7cDrZDiWYW7vRiWA8Cn2rX77F6DFmpyrPdp"
        );



        // const response = await readUserInfo(signer, userInfoProgramId, connection, 'sunit');

        // if (!response) {
        //     await addUserInfo(signer, userInfoProgramId, connection, 'sunit', 1.5);
        // } else {
        //     await updateUserInfo(signer, userInfoProgramId, connection, 'sunit', response.amount + 1);
        // }

        //await addUserInfo(signer, userInfoProgramId, connection, '82QCjg8kM5D17n28reCWYE85XbWLrtZJ', 1.5);
        //await updateUserInfo(signer, userInfoProgramId, connection, '82QCjg8kM5D17n28reCWYE85XbWLrtZJ', 1);

        console.log("Finished successfully");
    } catch (error) {
        console.error(error);
        process.exit(1);
    }
})();



//fund status, amount, user pubkey, bot status