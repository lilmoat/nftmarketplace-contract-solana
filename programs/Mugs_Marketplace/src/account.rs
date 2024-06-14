use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalPool {
    // 8 + 368
    pub super_admin: Pubkey,        // 32
    pub market_fee_sol: u64,        // 8    Permyriad
    pub team_count: u64,            // 8
    pub team_treasury: [Pubkey; 8], // 8 * 32
    pub treasury_rate: [u64; 8],    // 8 * 8
}

#[account]
#[derive(Default)]
pub struct SellData {
    // 8 + 120
    pub mint: Pubkey,           // 32
    pub seller: Pubkey,         // 32
    pub collection: Pubkey,     // 32
    pub price_sol: u64,         // 8
    pub listed_date: i64,       // 8
    pub active: u64,            // 8
}

#[account]
#[derive(Default)]
pub struct OfferData {
    // 8 + 88
    pub mint: Pubkey,               // 32
    pub buyer: Pubkey,              // 32
    pub offer_price: u64,           // 8
    pub offer_listing_date: i64,    // 8
    pub active: u64,                // 8
}

#[account]
#[derive(Default)]
pub struct AuctionData {
    // 8 + 152
    pub mint: Pubkey,               // 32
    pub creator: Pubkey,            // 32
    pub start_price: u64,           // 8
    pub min_increase_amount: u64,   // 8
    pub start_date: i64,            // 8
    pub last_bid_date: i64,         // 8
    pub last_bidder: Pubkey,        // 32
    pub highest_bid: u64,           // 8
    pub duration: i64,              // 8
    // 0-canceled, 1-started, 2-claimed, 3-reserved
    pub status: u64,                // 8
}

#[account]
#[derive(Default)]
pub struct UserData {
    // 8 + 48
    pub address: Pubkey,            // 32
    pub traded_volume: u64,         // 8
    pub escrow_sol_balance: u64,    // 8
}

impl AuctionData {
    pub fn get_end_date(&self) -> i64 {
        self.start_date + self.duration
    }
}