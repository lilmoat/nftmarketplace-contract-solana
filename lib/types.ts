import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

export const GLOBAL_AUTHORITY_SEED = "global-authority-v1";
export const SELL_DATA_SEED = "sell-info-v1";
export const SELL_DATA_SIZE = 128;
export const OFFER_DATA_SEED = "offer-info-v1";
export const OFFER_DATA_SIZE = 96;
export const USER_DATA_SEED = "user-info-v1";
export const AUCTION_DATA_SEED = "auction-info-v1";
export const AUCTION_DATA_SIZE = 160;
export const ESCROW_VAULT_SEED = "escrow-vault";

export const MARKETPLACE_PROGRAM_ID = new PublicKey("C29hER4SXQr3atHsuCrRmLAkXBpxvfLMCNeXg2TRTd9o");

export interface GlobalPool {
    // 8 + 368
    superAdmin: PublicKey,      // 32
    marketFeeSol: anchor.BN,    // 8
    teamCount: anchor.BN,       // 8
    teamTreasury: PublicKey[],  // 8 * 32
    treasuryRate: anchor.BN[],  // 8 * 8
}

export interface SellData {
    // 8 + 120
    mint: PublicKey,            // 32
    seller: PublicKey,          // 32
    collection: PublicKey,      // 32
    priceSol: anchor.BN,        // 8
    listedDate: anchor.BN,      // 8
    active: anchor.BN,          // 8
}

export interface OfferData {
    // 8 + 88
    mint: PublicKey,                // 32
    buyer: PublicKey,               // 32
    offerPrice: anchor.BN,          // 8
    offerListingDate: anchor.BN,    // 8
    active: anchor.BN,              // 8
}

export interface AuctionData {
    // 8 + 152
    mint: PublicKey,                // 32
    creator: PublicKey,             // 32
    startPrice: anchor.BN,          // 8
    minIncreaseAmount: anchor.BN,   // 8
    startDate: anchor.BN,           // 8
    lastBidDate: anchor.BN,         // 8
    lastBidder: PublicKey,          // 32
    highestBid: anchor.BN,          // 8
    duration: anchor.BN,            // 8
    status: anchor.BN,              // 8
}

export interface UserData {
    // 8 + 48
    address: PublicKey,             // 32
    tradedVolume: anchor.BN,        // 8
    escrowSolBalance: anchor.BN,    // 8
}