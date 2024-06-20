use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    // 0x1770 - 0
    #[msg("Invalid Super Owner")]
    InvalidSuperOwner,
    // 0x1771
    #[msg("Invalid Owner")]
    InvalidOwner,
    // 0x1772
    #[msg("Invalid Global Pool Address")]
    InvalidGlobalPool,
    // 0x1773
    #[msg("Marketplace Fee is Permyriad")]
    InvalidFeePercent,

    // 0x1774
    #[msg("Max Team Count is 8")]
    MaxTeamCountExceed,
    // 0x1775 - 5
    #[msg("Treasury Wallet Not Configured")]
    NoTeamTreasuryYet,
    // 0x1776
    #[msg("Treasury Address Not Exist")]
    TreasuryAddressNotFound,
    // 0x1777
    #[msg("Treasury Address Already Exist")]
    TreasuryAddressAlreadyAdded,
    // 0x1778
    #[msg("Total Treasury Rate Sum Should Less Than 100%")]
    MaxTreasuryRateSumExceed,
    // 0x1779
    #[msg("Team Treasury Wallet Count Mismatch")]
    TeamTreasuryCountMismatch,
    // 0x177a - 10
    #[msg("Team Treasury Wallet Address Mismatch")]
    TeamTreasuryAddressMismatch,

    // 0x177b
    #[msg("Uninitialized Account")]
    Uninitialized,
    // 0x177c
    #[msg("Instruction Parameter is Invalid")]
    InvalidParamInput,

    // 0x177d
    #[msg("Payer Mismatch with NFT Seller")]
    SellerMismatch,
    // 0x177e
    #[msg("Invalid NFT Data Account")]
    InvalidNFTDataAcount,
    // 0x177f - 15
    #[msg("The NFT Is Not Listed")]
    NotListedNFT,

    // 0x1780
    #[msg("Seller Account Mismatch with NFT Seller Data")]
    SellerAccountMismatch,
    // 0x1781
    #[msg("Buyer Sol Balance is Less than NFT SOL Price")]
    InsufficientBuyerSolBalance,
    // 0x1782
    #[msg("Buyer Token Balance is Less than NFT Token Price")]
    InsufficientBuyerTokenBalance,

    // 0x1783
    #[msg("Invalid Metadata Address")]
    InvaliedMetadata,
    // 0x1784 - 20
    #[msg("Can't Parse The NFT's Creators")]
    MetadataCreatorParseError,

    // 0x1785
    #[msg("Offer Data Mint mismatch with NFT Pubkey")]
    InvalidOfferDataMint,
    // 0x1786
    #[msg("Offer Data Buyer mismatch with Payer Pubkey")]
    InvalidOfferDataBuyer,
    // 0x1787
    #[msg("Making Offer for Not Listed NFT")]
    OfferForNotListedNFT,
    // 0x1788
    #[msg("Offer Price Over Thank Listed Price")]
    InvalidOfferPrice,
    // 0x1789 - 25
    #[msg("Already Canceled Offer")]
    DisabledOffer,
    // 0x178a
    #[msg("Offer For Sold Or Canceled NFT Listing")]
    OfferForExpiredListingNFT,

    // 0x178b
    #[msg("Placing Bid For Ended Auction")]
    EndedAuction,
    // 0x178c
    #[msg("Placing Bid With Lower Than Highest Bid")]
    InvalidBidPrice,
    // 0x178d
    #[msg("Placing Bid Double From One Bidder")]
    DoubleBidFromOneBidder,
    // 0x178e - 30
    #[msg("Out Bidder Account Mismatch With LastBidder Data")]
    OutBidderMismatch,
    // 0x178f
    #[msg("Claiming Auction For Not Ended Auction")]
    NotEndedAuction,
    // 0x1790
    #[msg("Creator Account Mismatch with Auction Data")]
    CreatorAccountMismatch,
    // 0x1791
    #[msg("Bidder Account Mismatch with Auction Data")]
    BidderAccountMismatch,
    // 0x1792
    #[msg("Canceling Auction which has Bid")]
    AuctionHasBid,
    // 0x1793
    #[msg("Placing Bid From Auction Creator")]
    BidFromAuctionCreator,

    // 0x1794
    #[msg("Only Listing and Reserved Auction are possible to exist together")]
    ListingNotAvailable,
    // 0x1795
    #[msg("NFT Is Not In User ATA")]
    NFTIsNotInUserATA,
    // 0x1796
    #[msg("NFT Is Not In Escrow ATA")]
    NFTIsNotInEscrowATA,
}