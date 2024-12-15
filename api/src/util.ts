import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction } from "@solana/web3.js";

export enum FundStatus {
    Nil = 'Nil',
    Deposited = 'Deposited',
    Pending = 'Pending',
    Failed = 'Failed',
    Withdrawn = 'Withdrawn',
    Locked = 'Locked'
}

export enum BotStatus {
    Init = 'Init',
    Active = 'Active',
    Inactive = 'Inactive',
    Paused = 'Paused',
    Error = 'Error',
    Stopped = 'Stopped'
}

/**
 * Find the vault PDA for a given vault name
 * @param vaultName name of the vault
 * @param programId program ID
 * @returns vault PDA public key
 */
export function getVaultPDA(vaultName: string, programId: PublicKey): PublicKey {
    const [vault] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), Buffer.from(vaultName)],
        programId
    );
    return vault;
}

/**
 * Find the vault depositor PDA for a given vault and authority
 * @param vault vault public key
 * @param authority authority public key
 * @param programId program ID
 * @returns vault depositor PDA public key
 */
export function getVaultDepositorPDA(
    vault: PublicKey,
    authority: PublicKey,
    programId: PublicKey
): PublicKey {
    const [vaultDepositor] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("vault_depositor"),
            vault.toBuffer(),
            authority.toBuffer()
        ],
        programId
    );
    return vaultDepositor;
}

/**
 * Find the treasury PDA for a given vault name
 * @param vaultName name of the vault
 * @param programId program ID
 * @returns treasury PDA public key
 */
export function getTreasuryPDA(vaultName: string, programId: PublicKey): PublicKey {
    const [treasury] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury"), Buffer.from(vaultName)],
        programId
    );
    return treasury;
}