import { PublicKey } from "@solana/web3.js";

export function getVaultPda(programId: PublicKey, vaultId: string): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from(vaultId)],
        programId
    );
    return pda;
}

export function getTreasuryPda(programId: PublicKey, vaultId: string): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vaultId)],
        programId
    );
    return pda;
}
