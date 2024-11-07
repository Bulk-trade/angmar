import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";

export const getSolBalance = async (connection: Connection, address: string) => {
    try {
    
        const lamportBalance = await connection.getBalance(new PublicKey(address));
        const solBalance = lamportBalance / LAMPORTS_PER_SOL;
        return solBalance;
    } catch (e) {
        console.error(e)
        return 0;
    }
}

export const getTokenBalance = async (connection: Connection, user: string, tokenMint: string) => {
;
    let { value: tokenAccount } = await connection.getParsedTokenAccountsByOwner(new PublicKey(user), {
        mint: new PublicKey(tokenMint),
    });

    let tokenBalance: number = 0;
    tokenAccount.forEach((account, i) => {
        //Parse the account data
        const parsedAccountInfo: any = account.account.data;
        tokenBalance = parsedAccountInfo["parsed"]["info"]["tokenAmount"]["uiAmount"];
    });

    return tokenBalance;
}
