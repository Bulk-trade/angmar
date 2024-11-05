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
import { ConfirmOptions, Keypair, LAMPORTS_PER_SOL, Signer } from '@solana/web3.js';
import { assert, expect } from 'chai';
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
import { initializeKeypair } from '@solana-developers/helpers';

// ammInvariant == k == x * y
const mantissaSqrtScale = new BN(100_000);
const ammInitialQuoteAssetReserve = new BN(5 * 10 ** 13).mul(mantissaSqrtScale);
const ammInitialBaseAssetReserve = new BN(5 * 10 ** 13).mul(mantissaSqrtScale);

const opts: ConfirmOptions = {
    preflightCommitment: 'confirmed',
    skipPreflight: false,
    commitment: 'confirmed',
};

// Configure the client to use the local cluster.
const provider = anchor.AnchorProvider.local();
anchor.setProvider(provider);
const connection = provider.connection;

const program = anchor.workspace.DriftVaults as Program<DriftVaults>;
const usdcMint = Keypair.generate();
let solPerpOracle: PublicKey;
const metaplex = Metaplex.make(connection);

let adminClient: AdminClient;
let adminInitialized = false;
const initialSolPerpPrice = 100;

let perpMarketIndexes: number[] = [];
let spotMarketIndexes: number[] = [];
let oracleInfos: OracleInfo[] = [];

const bulkAccountLoader = new BulkAccountLoader(connection, 'confirmed', 1);

let _manager: Keypair;
let managerClient: VaultClient;
let managerUser: User;

let vd2: Keypair;
let vd2Client: VaultClient;
let vd2UserUSDCAccount: PublicKey;
let _vd2User: User;

let _delegate: Keypair;
let delegateClient: VaultClient;
let _delegateUser: User;

const usdcAmount = new BN(1_000).mul(QUOTE_PRECISION);

// initialize adminClient first to make sure program is bootstrapped

export async function localnetSetup() {
    console.log('Setting up localnet...');
    await mockUSDCMint(provider, usdcMint)

    try {
        if (adminClient && (await isDriftInitialized(adminClient))) {
            console.log('Drift already initialized');
            return;
        }

        solPerpOracle = await mockOracle(initialSolPerpPrice, undefined, undefined);
        perpMarketIndexes = [0];
        spotMarketIndexes = [0, 1];
        oracleInfos = [{ publicKey: solPerpOracle, source: OracleSource.PYTH }];
        adminClient = new AdminClient({
            connection,
            wallet: provider.wallet,
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

        bootstrapVaults();
    }
    catch (e) {
        console.error('Error initializing AdminClient:', e);
        throw e;
    }
}

export async function bootstrapVaults() {
    await adminClient.subscribe();

    const signer = await initializeKeypair(connection, {
        airdropAmount: LAMPORTS_PER_SOL,
        envVariableName: "PRIVATE_KEY",
    });

    // init vault manager
    const bootstrapManager = await bootstrapSignerClientAndUser({
        signer,
        payer: provider,
        usdcMint,
        usdcAmount,
        driftClientConfig: {
            accountSubscription: {
                type: 'websocket',
                resubTimeoutMs: 30_000,
            },
            opts,
            activeSubAccountId: 0,
            perpMarketIndexes,
            spotMarketIndexes,
            oracleInfos,
        },
    });
    _manager = bootstrapManager.signer;
   // managerClient = bootstrapManager.vaultClient;
    managerUser = bootstrapManager.user;

    console.log('_manager:', _manager.publicKey.toString());
    //console.log('managerUser:', managerUser);

    const delegateSigner = await initializeKeypair(connection, {
        airdropAmount: LAMPORTS_PER_SOL,
        envVariableName: "PRIVATE_KEY_DELEGATE",
    });

    const bootstrapDelegate = await bootstrapSignerClientAndUser({
        signer: delegateSigner,
        payer: provider,
        usdcMint,
        usdcAmount,
        skipUser: true,
        driftClientConfig: {
            accountSubscription: {
                type: 'websocket',
                resubTimeoutMs: 30_000,
            },
            opts,
            activeSubAccountId: 0,
            perpMarketIndexes,
            spotMarketIndexes,
            oracleInfos,
        },
    });
    _delegate = bootstrapDelegate.signer;
    _delegateUser = bootstrapDelegate.user;

    console.log('_delegate:', _delegate.publicKey.toString());
    //console.log('_delegateUser:', _delegateUser);

    const userSigner = await initializeKeypair(connection, {
        airdropAmount: LAMPORTS_PER_SOL,
        envVariableName: "PRIVATE_KEY_USER",
    });


    // the VaultDepositor for the vault
    const bootstrapVD2 = await bootstrapSignerClientAndUser({
        signer: userSigner,
        payer: provider,
        usdcMint,
        usdcAmount,
        skipUser: true,
        depositCollateral: false,
        driftClientConfig: {
            accountSubscription: {
                type: 'websocket',
                resubTimeoutMs: 30_000,
            },
            opts,
            activeSubAccountId: 0,
            perpMarketIndexes,
            spotMarketIndexes,
            oracleInfos,
        },
    });
    vd2 = bootstrapVD2.signer;
   // vd2Client = bootstrapVD2.vaultClient;
    vd2UserUSDCAccount = bootstrapVD2.userUSDCAccount.publicKey;
    _vd2User = bootstrapVD2.user;

    console.log('vd2:', vd2.publicKey.toString());
    console.log('vd2UserUSDCAccount:', vd2UserUSDCAccount.toString());
   // console.log('_vd2User:', _vd2User);

    // start account loader
    bulkAccountLoader.startPolling();
    await bulkAccountLoader.load();
 }

localnetSetup()