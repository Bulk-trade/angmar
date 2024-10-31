import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
    AdminClient,
    BASE_PRECISION,
    BN,
    BulkAccountLoader,
    ZERO,
    PRICE_PRECISION,
    User,
    OracleSource,
    PublicKey,
    getLimitOrderParams,
    PostOnlyParams,
    PositionDirection,
    getUserAccountPublicKey,
    UserAccount,
    QUOTE_PRECISION,
    getOrderParams,
    MarketType,
    PEG_PRECISION,
    calculatePositionPNL,
    getInsuranceFundStakeAccountPublicKey,
    InsuranceFundStake,
    DriftClient,
    OracleInfo,
    TEN,
    PERCENTAGE_PRECISION,
    TWO,
    getTokenAmount,
    getUserStatsAccountPublicKey,
    DRIFT_PROGRAM_ID,
    OrderType,
    isVariant,
    Wallet,
} from '@drift-labs/sdk';
import {
    bootstrapSignerClientAndUser,
    calculateAllTokenizedVaultPdas,
    createUserWithUSDCAccount,
    doWashTrading,
    getVaultDepositorValue,
    initializeQuoteSpotMarket,
    initializeSolSpotMarket,
    initializeSolSpotMarketMaker,
    isDriftInitialized,
    mockOracle,
    mockUSDCMint,
    printTxLogs,
    setFeedPrice,
    sleep,
    validateTotalUserShares,
} from './testHelpers';
import { getMint } from '@solana/spl-token';
import { ConfirmOptions, Connection, Keypair, LAMPORTS_PER_SOL, Signer } from '@solana/web3.js';
import { assert } from 'chai';
import {
    VaultClient,
    getTokenizedVaultMintAddressSync,
    getVaultAddressSync,
    getVaultDepositorAddressSync,
    encodeName,
    DriftVaults,
    VaultProtocolParams,
    getVaultProtocolAddressSync,
    WithdrawUnit,
} from '@drift-labs/vaults-sdk';

import { Metaplex } from '@metaplex-foundation/js';
import { sign } from 'crypto';
import { initializeKeypair } from '@solana-developers/helpers';

const BULK_PROGRAM_ID = process.env.PROGRAM_ID || '';
const connection = new Connection("http://localhost:8899", "confirmed");
const vaultProgramId = new PublicKey(
    BULK_PROGRAM_ID
);

const mantissaSqrtScale = new BN(100_000);
const ammInitialQuoteAssetReserve = new BN(5 * 10 ** 13).mul(mantissaSqrtScale);
const ammInitialBaseAssetReserve = new BN(5 * 10 ** 13).mul(mantissaSqrtScale);

const usdcMint = Keypair.generate();
let solPerpOracle: PublicKey;
const metaplex = Metaplex.make(connection);

let adminClient: AdminClient;
let adminInitialized = false;
const initialSolPerpPrice = 100;

let perpMarketIndexes: number[] = [];
let spotMarketIndexes: number[] = [];
let oracleInfos: OracleInfo[] = [];

export async function setup() {
    const signer = await initializeKeypair(connection, {
        airdropAmount: LAMPORTS_PER_SOL,
        envVariableName: "PRIVATE_KEY",
    });

    // initialize adminClient first to make sure program is bootstrapped
    await mockUSDCMint(connection, signer, usdcMint);

    if (adminClient && (await isDriftInitialized(adminClient))) {
        console.log('Drift already initialized');
        return;
    }

    try {
        solPerpOracle = await mockOracle(initialSolPerpPrice, undefined, undefined);
        perpMarketIndexes = [0];
        spotMarketIndexes = [0, 1];
        oracleInfos = [{ publicKey: solPerpOracle, source: OracleSource.PYTH }];
        adminClient = new AdminClient({
            connection,
            wallet: new Wallet(signer),
            opts: {
                commitment: 'confirmed',
            },
            activeSubAccountId: 0,
            perpMarketIndexes,
            spotMarketIndexes,
            oracleInfos,
            accountSubscription: {
                type: 'websocket',
                resubTimeoutMs: 30_000,
            },
        });

        const startInitTime = Date.now();
        console.log('Initializing AdminClient...');

        await adminClient.initialize(usdcMint.publicKey, true);
        await adminClient.subscribe();
        await initializeQuoteSpotMarket(adminClient, usdcMint.publicKey);
        await initializeSolSpotMarket(adminClient, solPerpOracle);
        await Promise.all([
            adminClient.updateSpotMarketOrdersEnabled(0, true),
            adminClient.updateSpotMarketOrdersEnabled(1, true),
            adminClient.initializePerpMarket(
                0,
                solPerpOracle,
                ammInitialBaseAssetReserve,
                ammInitialQuoteAssetReserve,
                new BN(0), // 1 HOUR
                new BN(initialSolPerpPrice).mul(PEG_PRECISION)
            ),
        ]);
        await Promise.all([
            adminClient.updatePerpAuctionDuration(new BN(0)),
            adminClient.updatePerpMarketCurveUpdateIntensity(0, 100),
        ]);

        await adminClient.fetchAccounts();

        console.log(`AdminClient initialized in ${Date.now() - startInitTime}ms`);
        adminInitialized = true;
    } catch (e) {
        console.error('Error initializing AdminClient:', e);
        throw e;
    }
}