import { VersionedTransactionResponse } from "@solana/web3.js";

export function handleTransactionResponse(transactionResponse: VersionedTransactionResponse | null, signature: string) {
    // If no response is received, log an error and return
    if (!transactionResponse) {
        console.error("Transaction not confirmed");
        return 0;
    }

    // If the transaction fails, log the error
    if (transactionResponse.meta?.err) {
        console.error(`Transaction Failed: ${JSON.stringify(transactionResponse.meta?.err)}`);
        console.error(`https://solscan.io/tx/${signature}`);
        return 0;
    }

    console.log(`https://solscan.io/tx/${signature}`);
    return 1;
}
