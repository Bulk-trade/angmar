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
import { ConfirmOptions, Keypair, Signer } from '@solana/web3.js';
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
