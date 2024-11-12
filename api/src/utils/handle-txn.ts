import { VersionedTransactionResponse } from "@solana/web3.js";

export function handleTransactionResponse(transactionResponse: VersionedTransactionResponse | null, signature: string) {
    // If no response is received, log an error and return
    if (!transactionResponse) {
        console.error("Transaction not confirmed");
        throw new Error("Transaction not confirmed");
        
    }

    // If the transaction fails, log the error
    if (transactionResponse.meta?.err) {
        console.error(`Transaction Failed: ${JSON.stringify(transactionResponse.meta?.err)}`);
        console.log(`https://solscan.io/tx/${signature}`);
        console.log(`https://explorer.solana.com/tx/${signature}?cluster=custom`);
        console.log(`https://solscan.io/tx/${signature}?cluster=custom`);
        throw new Error(`Transaction Failed: ${JSON.stringify(transactionResponse.meta?.err)}`);
    }

    console.log(`https://solscan.io/tx/${signature}`);
    console.log(`https://explorer.solana.com/tx/${signature}?cluster=custom`);
    console.log(`https://solscan.io/tx/${signature}?cluster=custom`);
    return 1;
}
