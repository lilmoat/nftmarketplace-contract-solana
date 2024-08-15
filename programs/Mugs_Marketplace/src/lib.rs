use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::{accounts::Metadata, types::Creator};
use solana_program::program::{invoke, invoke_signed};
use solana_program::system_instruction;

pub mod account;
pub mod constants;
pub mod error;

use account::*;
use constants::*;
use error::*;

declare_id!("5J3fJvN67uWLo2uNaygTJjdRoJs5mxn9XgtXroiQkcwm");

#[program]
pub mod mugs_marketplace {

    use mpl_token_metadata::instructions::{
        DelegateLockedTransferV1CpiBuilder, LockV1CpiBuilder, RevokeLockedTransferV1CpiBuilder,
        TransferV1CpiBuilder, UnlockV1CpiBuilder,
    };

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _global_bump: u8, _escrow_bump: u8) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        global_authority.super_admin = ctx.accounts.admin.key();
        Ok(())
    }
    pub fn update_fee(ctx: Context<SetTreshold>, _global_bump: u8, sol_fee: u64) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        // Assert payer is the superadmin
        require!(
            global_authority.super_admin == ctx.accounts.admin.key(),
            MarketplaceError::InvalidSuperOwner
        );
        require!(sol_fee < PERMYRIAD, MarketplaceError::InvalidFeePercent);

        global_authority.market_fee_sol = sol_fee;
        Ok(())
    }
    pub fn add_team_treasury(
        ctx: Context<AddTreasury>,
        _global_bump: u8,
        address: Pubkey,
        rate: u64,
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        // Assert payer is the superadmin
        require!(
            global_authority.super_admin == ctx.accounts.admin.key(),
            MarketplaceError::InvalidSuperOwner
        );
        // Max Team Treasury Count is 8
        require!(
            global_authority.team_count < 8,
            MarketplaceError::MaxTeamCountExceed
        );
        // Distribution rate by Permyriad
        require!(
            rate <= PERMYRIAD && rate > 0,
            MarketplaceError::InvalidFeePercent
        );

        let mut exist: u8 = 0;
        let mut sum: u64 = rate;
        for i in 0..global_authority.team_count {
            let index = i as usize;
            if global_authority.team_treasury[index].eq(&address) {
                exist = 1;
            }
            sum += global_authority.treasury_rate[index];
        }
        require!(exist == 0, MarketplaceError::TreasuryAddressAlreadyAdded);
        // Total sum of treasury rates less than full permyriad
        require!(sum <= PERMYRIAD, MarketplaceError::MaxTreasuryRateSumExceed);

        let index: usize = global_authority.team_count as usize;
        global_authority.team_treasury[index] = address;
        global_authority.treasury_rate[index] = rate;
        global_authority.team_count += 1;
        Ok(())
    }
    pub fn remove_team_treasury(
        ctx: Context<RemoveTreasury>,
        _global_bump: u8,
        address: Pubkey,
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        // Assert payer is the superadmin
        require!(
            global_authority.super_admin == ctx.accounts.admin.key(),
            MarketplaceError::InvalidSuperOwner
        );
        // Assert no treasury exist
        require!(
            global_authority.team_count > 0,
            MarketplaceError::NoTeamTreasuryYet
        );

        let mut exist: u8 = 0;
        for i in 0..global_authority.team_count {
            let index = i as usize;
            if global_authority.team_treasury[index].eq(&address) {
                if i < global_authority.team_count - 1 {
                    let last_idx = (global_authority.team_count - 1) as usize;
                    global_authority.team_treasury[index] =
                        global_authority.team_treasury[last_idx];
                    global_authority.treasury_rate[index] =
                        global_authority.treasury_rate[last_idx];
                }
                global_authority.team_count -= 1;
                exist = 1;
            }
        }
        require!(exist == 1, MarketplaceError::TreasuryAddressNotFound);
        Ok(())
    }

    // Initialize User PDA for Escrow & Traded Volume
    pub fn init_user_pool(ctx: Context<InitUserPool>, _bump: u8) -> Result<()> {
        let user_pool = &mut ctx.accounts.user_pool;
        user_pool.address = ctx.accounts.owner.key();
        Ok(())
    }

    // Init NFT listed info - Sell Data PDA
    pub fn init_sell_data(ctx: Context<InitSellData>, nft: Pubkey, _bump: u8) -> Result<()> {
        let sell_data_info = &mut ctx.accounts.sell_data_info;
        sell_data_info.mint = nft;
        Ok(())
    }

    pub fn set_price(ctx: Context<SetPrice>, _sell_bump: u8, price: u64) -> Result<()> {
        let sell_data_info = &mut ctx.accounts.sell_data_info;
        msg!("Mint: {:?}", sell_data_info.mint);

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        // Assert NFT seller is payer
        require!(
            ctx.accounts.owner.key().eq(&sell_data_info.seller),
            MarketplaceError::SellerMismatch
        );
        // Assert Already Delisted NFT
        require!(sell_data_info.active == 1, MarketplaceError::NotListedNFT);

        sell_data_info.price_sol = price;

        Ok(())
    }

    pub fn deposit_to_escrow(
        ctx: Context<Deposit>,
        _user_bump: u8,
        _escrow_bump: u8,
        sol: u64,
    ) -> Result<()> {
        require!(sol > 0, MarketplaceError::InvalidParamInput);

        let user_pool = &mut ctx.accounts.user_pool;
        msg!("User: {:?}, Sol Deposit: {}", user_pool.address, sol);

        // Assert User Pubkey with User Data PDA Address
        require!(
            ctx.accounts.owner.key().eq(&user_pool.address),
            MarketplaceError::InvalidOwner
        );

        invoke(
            &system_instruction::transfer(
                ctx.accounts.owner.key,
                ctx.accounts.escrow_vault.key,
                sol,
            ),
            &[
                ctx.accounts.owner.to_account_info().clone(),
                ctx.accounts.escrow_vault.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;
        user_pool.escrow_sol_balance += sol;

        Ok(())
    }

    pub fn withdraw_from_escrow(
        ctx: Context<Withdraw>,
        _user_bump: u8,
        escrow_bump: u8,
        sol: u64,
    ) -> Result<()> {
        require!(sol > 0, MarketplaceError::InvalidParamInput);

        let user_pool = &mut ctx.accounts.user_pool;
        msg!("User: {:?}, Sol Withdraw: {}", user_pool.address, sol);

        // Assert User Pubkey with User Data PDA Address
        require!(
            ctx.accounts.owner.key().eq(&user_pool.address),
            MarketplaceError::InvalidOwner
        );

        let seeds = &[ESCROW_VAULT_SEED.as_bytes(), &[escrow_bump]];
        let signer = &[&seeds[..]];

        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.escrow_vault.key,
                ctx.accounts.owner.key,
                sol,
            ),
            &[
                ctx.accounts.owner.to_account_info().clone(),
                ctx.accounts.escrow_vault.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            signer,
        )?;
        user_pool.escrow_sol_balance -= sol;

        Ok(())
    }

    pub fn init_offer_data(ctx: Context<InitOfferData>, nft: Pubkey, _bump: u8) -> Result<()> {
        let offer_data_info = &mut ctx.accounts.offer_data_info;
        offer_data_info.mint = nft;
        offer_data_info.buyer = ctx.accounts.payer.key();
        Ok(())
    }

    pub fn make_offer(
        ctx: Context<MakeOffer>,
        _sell_bump: u8,
        _offer_bump: u8,
        _user_bump: u8,
        _escrow_bump: u8,
        price: u64,
    ) -> Result<()> {
        let sell_data_info = &mut ctx.accounts.sell_data_info;
        msg!(
            "Mint: {:?}, buyer: {:?}",
            sell_data_info.mint,
            ctx.accounts.owner.key()
        );

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );

        let offer_data_info = &mut ctx.accounts.offer_data_info;
        require!(
            ctx.accounts.nft_mint.key().eq(&offer_data_info.mint),
            MarketplaceError::InvalidOfferDataMint
        );
        // Assert Payer is same with Offer Data Buyer
        require!(
            ctx.accounts.owner.key().eq(&offer_data_info.buyer),
            MarketplaceError::InvalidOfferDataBuyer
        );
        // Assert Already delisted NFT
        require!(
            sell_data_info.active == 1,
            MarketplaceError::OfferForNotListedNFT
        );
        // Offer price range is from x1 to x0.5
        require!(
            sell_data_info.price_sol > price && sell_data_info.price_sol / 2 <= price,
            MarketplaceError::InvalidOfferPrice
        );

        offer_data_info.offer_listing_date = sell_data_info.listed_date;
        offer_data_info.offer_price = price;
        offer_data_info.active = 1;

        let user_pool = &mut ctx.accounts.user_pool;
        msg!("User: {:?}, Deposit: {},", user_pool.address, price);

        // Assert User Pubkey with User Data PDA Address
        require!(
            ctx.accounts.owner.key().eq(&user_pool.address),
            MarketplaceError::InvalidOwner
        );

        invoke(
            &system_instruction::transfer(
                ctx.accounts.owner.key,
                ctx.accounts.escrow_vault.key,
                price,
            ),
            &[
                ctx.accounts.owner.to_account_info().clone(),
                ctx.accounts.escrow_vault.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;
        user_pool.escrow_sol_balance += price;

        Ok(())
    }

    pub fn cancel_offer(
        ctx: Context<CancelOffer>,
        _offer_bump: u8,
        _escrow_bump: u8,
        _user_bump: u8,
    ) -> Result<()> {
        let offer_data_info = &mut ctx.accounts.offer_data_info;
        msg!(
            "Mint: {:?}, buyer: {:?}",
            offer_data_info.mint,
            ctx.accounts.owner.key()
        );

        // Assert NFT Pubkey with Offer Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&offer_data_info.mint),
            MarketplaceError::InvalidOfferDataMint
        );
        // Asser Payer is the Offer Data Address
        require!(
            ctx.accounts.owner.key().eq(&offer_data_info.buyer),
            MarketplaceError::InvalidOfferDataBuyer
        );
        require!(offer_data_info.active == 1, MarketplaceError::DisabledOffer);

        let seeds = &[ESCROW_VAULT_SEED.as_bytes(), &[_escrow_bump]];
        let signer = &[&seeds[..]];

        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.escrow_vault.key,
                ctx.accounts.owner.key,
                offer_data_info.offer_price,
            ),
            &[
                ctx.accounts.owner.to_account_info().clone(),
                ctx.accounts.escrow_vault.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            signer,
        )?;

        offer_data_info.active = 0;
        Ok(())
    }

    pub fn init_auction_data(ctx: Context<InitAuctionData>, nft: Pubkey, _bump: u8) -> Result<()> {
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        auction_data_info.mint = nft;
        Ok(())
    }

    pub fn place_bid(
        ctx: Context<PlaceBid>,
        _auction_bump: u8,
        escrow_bump: u8,
        price: u64,
    ) -> Result<()> {
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        let sell_data_info = &mut ctx.accounts.sell_data_info;

        let timestamp = Clock::get()?.unix_timestamp;
        msg!("Place Date: {}", timestamp);
        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        // Assert Already Disabled Auction
        require!(
            auction_data_info.status == 1 || auction_data_info.status == 3,
            MarketplaceError::NotListedNFT
        );
        // Assert Auction Already Ended
        require!(
            auction_data_info.status == 3 || auction_data_info.get_end_date() > timestamp,
            MarketplaceError::EndedAuction
        );
        // New Bid should be increased more than min_increase_amount
        require!(
            auction_data_info.highest_bid + auction_data_info.min_increase_amount <= price,
            MarketplaceError::InvalidBidPrice
        );
        // Assert OutBidder Address with the Last Bidder
        require!(
            Pubkey::default().eq(&auction_data_info.last_bidder)
                || ctx
                    .accounts
                    .out_bidder
                    .key()
                    .eq(&auction_data_info.last_bidder),
            MarketplaceError::OutBidderMismatch
        );
        // Assert New Bidder is same with the Last Bidder
        require!(
            !ctx.accounts.bidder.key().eq(&auction_data_info.last_bidder),
            MarketplaceError::DoubleBidFromOneBidder
        );
        // Assert Bid from Auction Creator
        require!(
            !ctx.accounts.bidder.key().eq(&auction_data_info.creator),
            MarketplaceError::BidFromAuctionCreator
        );

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        if sell_data_info.active == 1 {
            // Assert NFT seller is payer
            require!(
                auction_data_info.creator.eq(&sell_data_info.seller),
                MarketplaceError::SellerMismatch
            );
        }

        msg!(
            "Mint: {:?}, Bidder: {:?}",
            auction_data_info.mint,
            ctx.accounts.bidder.key()
        );

        let seeds = &[ESCROW_VAULT_SEED.as_bytes(), &[escrow_bump]];
        let signer = &[&seeds[..]];

        // Refund Last Bidder Escrow
        if !Pubkey::default().eq(&auction_data_info.last_bidder) {
            invoke_signed(
                &system_instruction::transfer(
                    ctx.accounts.escrow_vault.key,
                    ctx.accounts.out_bidder.key,
                    auction_data_info.highest_bid,
                ),
                &[
                    ctx.accounts.out_bidder.to_account_info().clone(),
                    ctx.accounts.escrow_vault.to_account_info().clone(),
                    ctx.accounts.system_program.to_account_info().clone(),
                ],
                signer,
            )?;
        }
        // Escrow New Bidder funds
        invoke(
            &system_instruction::transfer(
                ctx.accounts.bidder.key,
                ctx.accounts.escrow_vault.key,
                price,
            ),
            &[
                ctx.accounts.bidder.to_account_info().clone(),
                ctx.accounts.escrow_vault.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;

        auction_data_info.last_bid_date = timestamp;
        auction_data_info.last_bidder = ctx.accounts.bidder.key();
        auction_data_info.highest_bid = price;
        if auction_data_info.status == 3 {
            auction_data_info.status = 1;
            auction_data_info.start_date = timestamp;
            sell_data_info.active = 0;
        }

        Ok(())
    }

    pub fn update_reserve(
        ctx: Context<UpdateReserve>,
        _auction_bump: u8,
        price: u64,
    ) -> Result<()> {
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        msg!("Mint: {:?}", auction_data_info.mint);

        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        // Assert Valid Reserved Auction
        require!(
            auction_data_info.status == 3,
            MarketplaceError::NotListedNFT
        );
        // Assert Auction Has No Bidder
        require!(
            Pubkey::default().eq(&auction_data_info.last_bidder),
            MarketplaceError::AuctionHasBid
        );
        // Assert Creator Pubkey is same with the Auction Data Creator
        require!(
            ctx.accounts.creator.key().eq(&auction_data_info.creator),
            MarketplaceError::CreatorAccountMismatch
        );

        auction_data_info.start_price = price;

        Ok(())
    }

    pub fn list_pnft_for_sale(
        ctx: Context<ListPNftForSale>,
        _global_bump: u8,
        _sell_bump: u8,
        _auction_bump: u8,
        price_sol: u64,
    ) -> Result<()> {
        let sell_data_info = &mut ctx.accounts.sell_data_info;

        msg!("Mint: {:?}", sell_data_info.mint);

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[_global_bump]];
        let signer = &[&seeds[..]];

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );

        let auction_data_info = &mut ctx.accounts.auction_data_info;
        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );

        // Assert Reserved Or Not Started Auction
        require!(
            auction_data_info.status != 1,
            MarketplaceError::ListingNotAvailable
        );

        // Assert Owner Pubkey is same with the Auction Data Creator if NFT is in Reserved Auction
        if auction_data_info.status == 3 {
            require!(
                ctx.accounts.owner.key().eq(&auction_data_info.creator),
                MarketplaceError::CreatorAccountMismatch
            );
        }

        // Get Collection address from Metadata
        let mint_metadata: &AccountInfo = &ctx.accounts.mint_metadata;
        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Metadata::find_pda(&ctx.accounts.nft_mint.key());

        require!(
            metadata == mint_metadata.key(),
            MarketplaceError::InvaliedMetadata
        );

        // verify metadata is legit
        let nft_metadata =
            Metadata::safe_deserialize(&mut mint_metadata.data.borrow_mut()).unwrap();

        if let Some(creators) = nft_metadata.creators {
            let mut collection: Pubkey = Pubkey::default();
            for creator in creators {
                if creator.verified {
                    collection = creator.address;
                    break;
                }
            }
            sell_data_info.collection = collection;
            msg!("Collection= {:?}", collection);
        } else {
            return Err(error!(MarketplaceError::MetadataCreatorParseError));
        }

        // Save Sell Data info
        let timestamp = Clock::get()?.unix_timestamp;
        msg!("Listed Date: {}", timestamp);

        sell_data_info.seller = ctx.accounts.owner.key();
        sell_data_info.price_sol = price_sol;
        sell_data_info.listed_date = timestamp;
        sell_data_info.active = 1;

        let token_account_info = &ctx.accounts.user_token_account;
        let dest_token_account_info = &ctx.accounts.dest_nft_token_account;
        let owner: &Signer = &ctx.accounts.owner;
        let nft_mint = &ctx.accounts.nft_mint;
        let token_mint_record = &ctx.accounts.token_mint_record;
        let token_mint_edition = &ctx.accounts.token_mint_edition;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let sysvar_instructions = &ctx.accounts.sysvar_instructions;
        let auth_rules_program = &ctx.accounts.auth_rules_program;
        let auth_rules = &ctx.accounts.auth_rules;
        let global_authority = &ctx.accounts.global_authority;

        // Transfer NFT only if Not in Reserved Auction
        if auction_data_info.status != 3 {
            // Assert NFT is in user Account
            require!(
                token_account_info.amount == 1,
                MarketplaceError::NFTIsNotInUserATA
            );

            let expected_token_account = anchor_spl::associated_token::get_associated_token_address(
                &global_authority.key(),
                &nft_mint.key(),
            );

            if expected_token_account == dest_token_account_info.key() {
                msg!("list pnft start");

                DelegateLockedTransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                    .master_edition(Some(&token_mint_edition.to_account_info()))
                    .spl_token_program(Some(&token_program.to_account_info()))
                    .system_program(&system_program.to_account_info())
                    .authority(&owner.to_account_info())
                    .delegate(&global_authority.to_account_info())
                    .payer(&owner.to_account_info())
                    .mint(&nft_mint.to_account_info())
                    .metadata(&mint_metadata.to_account_info())
                    .token(&token_account_info.to_account_info())
                    .token_record(Some(&token_mint_record.to_account_info()))
                    .amount(1)
                    .authorization_rules(Some(&auth_rules.to_account_info()))
                    .sysvar_instructions(&sysvar_instructions.to_account_info())
                    .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                    .system_program(&system_program.to_account_info())
                    .locked_address(token_account_info.key())
                    .invoke_signed(signer)?;
                LockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                    .edition(Some(&token_mint_edition.to_account_info()))
                    .spl_token_program(Some(&token_program.to_account_info()))
                    .system_program(&system_program.to_account_info())
                    .authority(&global_authority.to_account_info())
                    .payer(&owner.to_account_info())
                    .mint(&nft_mint.to_account_info())
                    .metadata(&mint_metadata.to_account_info())
                    .token(&token_account_info.to_account_info())
                    .token_owner(Some(&owner.to_account_info()))
                    .token_record(Some(&token_mint_record.to_account_info()))
                    .authorization_rules(Some(&auth_rules.to_account_info()))
                    .sysvar_instructions(&sysvar_instructions.to_account_info())
                    .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                    .system_program(&system_program.to_account_info())
                    .invoke_signed(signer)?;
                msg!("list pnft ix end");
            } else {

                // no longer needed with delegates

                // Assert NFT is in escrow Account
                // require!(
                //     dest_token_account_info.amount == 1,
                //     MarketplaceError::NFTIsNotInEscrowATA
                // );
            }
        }

        Ok(())
    }

    pub fn delist_pnft(ctx: Context<DelistPNft>, global_bump: u8, _sell_bump: u8) -> Result<()> {
        let sell_data_info = &mut ctx.accounts.sell_data_info;
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        msg!("Mint: {:?}", sell_data_info.mint);

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        // Assert NFT seller is payer
        require!(
            ctx.accounts.owner.key().eq(&sell_data_info.seller),
            MarketplaceError::SellerMismatch
        );
        // Assert Already Delisted NFT
        require!(sell_data_info.active == 1, MarketplaceError::NotListedNFT);
        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        if auction_data_info.status == 3 {
            // Assert Creator Pubkey is same with the Auction Data Creator
            require!(
                ctx.accounts.owner.key().eq(&auction_data_info.creator),
                MarketplaceError::CreatorAccountMismatch
            );
        }

        sell_data_info.active = 0;

        let token_program = &mut &ctx.accounts.token_program;
        let seeds: &[&[u8]; 2] = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];

        // Get Collection address from Metadata
        let mint_metadata: &AccountInfo = &ctx.accounts.mint_metadata;
        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Metadata::find_pda(&ctx.accounts.nft_mint.key());

        require!(
            metadata == mint_metadata.key(),
            MarketplaceError::InvaliedMetadata
        );

        let global_authority = &ctx.accounts.global_authority;
        let owner = &ctx.accounts.owner;
        let token_account_info = &ctx.accounts.user_token_account;
        let nft_mint = &ctx.accounts.nft_mint;

        let token_mint_edition = &ctx.accounts.token_mint_edition;
        let token_mint_record = &ctx.accounts.token_mint_record;
        let system_program = &ctx.accounts.system_program;
        let sysvar_instructions = &ctx.accounts.sysvar_instructions;
        let auth_rules_program = &ctx.accounts.auth_rules_program;
        let auth_rules = &ctx.accounts.auth_rules;

        if auction_data_info.status != 3 {
            msg!("delist ixn start");

            if mint_metadata.owner != ctx.accounts.token_metadata_program.key {
                msg!("Metadata Owner Err: {:?}", mint_metadata.owner);
            }
            if nft_mint.owner != ctx.accounts.token_program.key {
                msg!("NFT Mint Owner Err: {:?}", nft_mint.owner);
            }
            if token_account_info.owner != *ctx.accounts.token_program.key {
                msg!("Token Owner Err: {:?}", token_account_info.owner);
            }

            if token_mint_record.owner != ctx.accounts.token_metadata_program.key {
                msg!("Token Mint Record Owner Err: {:?}", token_mint_record.owner);
            }
            if token_mint_edition.owner != ctx.accounts.token_metadata_program.key {
                msg!(
                    "Token Mint Edition Owner Err: {:?}",
                    token_mint_edition.owner
                );
            }
            if auth_rules.owner != ctx.accounts.auth_rules_program.key {
                msg!("Auth Rules O");
            }
            UnlockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                .edition(Some(&token_mint_edition.to_account_info()))
                .spl_token_program(Some(&token_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .authority(&global_authority.to_account_info())
                .payer(&owner.to_account_info())
                .mint(&nft_mint.to_account_info())
                .metadata(&mint_metadata.to_account_info())
                .token(&token_account_info.to_account_info())
                .token_owner(Some(&owner.to_account_info()))
                .token_record(Some(&token_mint_record.to_account_info()))
                .authorization_rules(Some(&auth_rules.to_account_info()))
                .sysvar_instructions(&sysvar_instructions.to_account_info())
                .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .invoke_signed(signer)?;
            RevokeLockedTransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                .master_edition(Some(&token_mint_edition.to_account_info()))
                .spl_token_program(Some(&token_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .authority(&owner.to_account_info())
                .delegate(&global_authority.to_account_info())
                .payer(&owner.to_account_info())
                .mint(&nft_mint.to_account_info())
                .metadata(&mint_metadata.to_account_info())
                .token_record(Some(&token_mint_record.to_account_info()))
                .token(&token_account_info.to_account_info())
                .authorization_rules(Some(&auth_rules.to_account_info()))
                .sysvar_instructions(&sysvar_instructions.to_account_info())
                .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                .invoke_signed(signer)?;
            msg!("Create token account end");
        }

        Ok(())
    }

    pub fn purchase_pnft<'info>(
        ctx: Context<'_, '_, '_, 'info, PurchasePNft<'info>>,
        global_bump: u8,
        _nft_bump: u8,
        _seller_bump: u8,
        _buyer_bump: u8,
    ) -> Result<()> {
        // By Token should be zero or one
        let sell_data_info: &mut Box<Account<'info, SellData>> = &mut ctx.accounts.sell_data_info;
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        let buyer_user_pool = &mut ctx.accounts.buyer_user_pool;
        let seller_user_pool: &mut Account<'info, UserData> = &mut ctx.accounts.seller_user_pool;

        msg!("Purchase Mint: {:?}", sell_data_info.mint);

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        if auction_data_info.status == 3 {
            // Assert Creator Pubkey is same with the Auction Data Creator
            require!(
                ctx.accounts.seller.key().eq(&auction_data_info.creator),
                MarketplaceError::CreatorAccountMismatch
            );
        }

        // Get Collection address from Metadata
        let mint_metadata = &mut &ctx.accounts.mint_metadata;
        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Metadata::find_pda(&ctx.accounts.nft_mint.key());

        require!(
            metadata == mint_metadata.key(),
            MarketplaceError::InvaliedMetadata
        );

        // verify metadata is legit
        let nft_metadata =
            Metadata::safe_deserialize(&mut mint_metadata.data.borrow_mut()).unwrap();

        require!(sell_data_info.active == 1, MarketplaceError::NotListedNFT);
        // Assert Seller Sell Data Address
        require!(
            ctx.accounts.seller.key().eq(&sell_data_info.seller),
            MarketplaceError::SellerAccountMismatch
        );
        // Assert Seller User PDA Address
        require!(
            ctx.accounts.seller.key().eq(&seller_user_pool.address),
            MarketplaceError::InvalidOwner
        );
        // Assert Buyer User PDA Address
        require!(
            ctx.accounts.buyer.key().eq(&buyer_user_pool.address),
            MarketplaceError::InvalidOwner
        );

        sell_data_info.active = 0;
        if auction_data_info.status == 3 {
            auction_data_info.status = 0;
        }

        let dest_nft_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];

        let remaining_accounts: Vec<AccountInfo> = ctx.remaining_accounts.to_vec();
        let global_authority = &ctx.accounts.global_authority;
        let owner = &ctx.accounts.buyer;
        let token_account_info = &ctx.accounts.user_nft_token_account;
        let nft_mint = &ctx.accounts.nft_mint;

        let token_mint_edition = &ctx.accounts.token_mint_edition;
        let token_mint_record = &ctx.accounts.token_mint_record;
        let dest_token_mint_record = &ctx.accounts.dest_token_mint_record;
        let system_program = &ctx.accounts.system_program;
        let sysvar_instructions = &ctx.accounts.sysvar_instructions;
        let associated_token_program = &ctx.accounts.associated_token_program;
        let auth_rules_program = &ctx.accounts.auth_rules_program;
        let auth_rules = &ctx.accounts.auth_rules;
        let seller = &ctx.accounts.seller;
        // let creator = &ctx.accounts.creator;
        // At least one treasury should exist to trade NFT
        require!(
            global_authority.team_count > 0,
            MarketplaceError::NoTeamTreasuryYet
        );

        let creators: &Vec<Creator>;
        if let Some(cts) = &nft_metadata.creators {
            creators = cts;
        } else {
            return Err(error!(MarketplaceError::MetadataCreatorParseError));
        };
        require!(
            global_authority.team_count + creators.len() as u64 == remaining_accounts.len() as u64,
            MarketplaceError::TeamTreasuryCountMismatch
        );

        let total_share_fee =
            sell_data_info.price_sol * (nft_metadata.seller_fee_basis_points as u64) / PERMYRIAD;
        let fee_amount: u64 =
            sell_data_info.price_sol * global_authority.market_fee_sol / PERMYRIAD;
        // let total_fee_amount: u64 = total_share_fee + fee_amount;

        invoke(
            &system_instruction::transfer(
                ctx.accounts.buyer.key,
                ctx.accounts.seller.key,
                sell_data_info.price_sol,
            ),
            &[
                ctx.accounts.buyer.to_account_info().clone(),
                ctx.accounts.seller.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;

        let mut i = 0;
        // This is not expensive cuz the max count is 8
        for team_account in remaining_accounts {
            if i < global_authority.team_count {
                require!(
                    team_account
                        .key()
                        .eq(&global_authority.team_treasury[i as usize]),
                    MarketplaceError::TeamTreasuryAddressMismatch
                );
                invoke(
                    &system_instruction::transfer(
                        ctx.accounts.buyer.key,
                        &global_authority.team_treasury[i as usize],
                        fee_amount * global_authority.treasury_rate[i as usize] / PERMYRIAD,
                    ),
                    &[
                        ctx.accounts.buyer.to_account_info().clone(),
                        team_account.clone(),
                        ctx.accounts.system_program.to_account_info().clone(),
                    ],
                )?;
            } else {
                for creator in creators {
                    if creator.address == team_account.key() && creator.share != 0 {
                        let share_amount: u64 = total_share_fee * (creator.share as u64) / 100;
                        invoke(
                            &system_instruction::transfer(
                                ctx.accounts.buyer.key,
                                &team_account.key(),
                                share_amount,
                            ),
                            &[
                                ctx.accounts.buyer.to_account_info().clone(),
                                team_account.clone(),
                                ctx.accounts.system_program.to_account_info().clone(),
                            ],
                        )?;
                    }
                }
            }

            i += 1;
        }
        buyer_user_pool.traded_volume += sell_data_info.price_sol;
        seller_user_pool.traded_volume += sell_data_info.price_sol;
        UnlockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .edition(Some(&token_mint_edition.to_account_info()))
            .authority(&global_authority.to_account_info())
            .payer(&owner.to_account_info())
            .mint(&nft_mint.to_account_info())
            .metadata(&mint_metadata.to_account_info())
            .token_record(Some(&dest_token_mint_record.to_account_info()))
            .token_owner(Some(&seller.to_account_info()))
            .token(&dest_nft_token_account_info.to_account_info())
            .authorization_rules(Some(&auth_rules.to_account_info()))
            .spl_token_program(Some(&token_program.to_account_info()))
            .system_program(&system_program.to_account_info())
            .sysvar_instructions(&sysvar_instructions.to_account_info())
            .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
            .system_program(&system_program.to_account_info())
            .invoke_signed(signer)?;
        TransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .authority(&global_authority.to_account_info())
            .payer(&owner.to_account_info())
            .mint(&nft_mint.to_account_info())
            .metadata(&mint_metadata.to_account_info())
            .edition(Some(&token_mint_edition.to_account_info()))
            .destination_token(&token_account_info.to_account_info())
            .destination_owner(&owner.to_account_info())
            .destination_token_record(Some(&token_mint_record.to_account_info()))
            .token_record(Some(&dest_token_mint_record.to_account_info()))
            .token_owner(&seller.to_account_info())
            .token(&dest_nft_token_account_info.to_account_info())
            .amount(1)
            .authorization_rules(Some(&auth_rules.to_account_info()))
            .sysvar_instructions(&sysvar_instructions.to_account_info())
            .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
            .spl_ata_program(&associated_token_program.to_account_info())
            .spl_token_program(&token_program.to_account_info())
            .system_program(&system_program.to_account_info())
            .invoke_signed(signer)?;

        Ok(())
    }

    pub fn accept_offer_pnft<'info>(
        ctx: Context<'_, '_, '_, 'info, AcceptOfferPNft<'info>>,
        global_bump: u8,
        _nft_bump: u8,
        _offer_bump: u8,
        _buyer_bump: u8,
        _seller_bump: u8,
        escrow_bump: u8,
    ) -> Result<()> {
        let sell_data_info = &mut ctx.accounts.sell_data_info;
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        let buyer_user_pool = &mut ctx.accounts.buyer_user_pool;
        let seller_user_pool = &mut ctx.accounts.seller_user_pool;

        let buyer = &ctx.accounts.buyer;
        let token_account_info = &ctx.accounts.user_nft_token_account;
        let nft_mint = &ctx.accounts.nft_mint;

        let token_mint_edition = &ctx.accounts.token_mint_edition;
        let token_mint_record = &ctx.accounts.token_mint_record;
        let dest_token_mint_record = &ctx.accounts.dest_token_mint_record;
        let system_program = &ctx.accounts.system_program;
        let sysvar_instructions = &ctx.accounts.sysvar_instructions;
        let associated_token_program = &ctx.accounts.associated_token_program;
        let auth_rules_program = &ctx.accounts.auth_rules_program;
        let auth_rules = &ctx.accounts.auth_rules;

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );

        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        if auction_data_info.status == 3 {
            // Assert Creator Pubkey is same with the Auction Data Creator
            require!(
                ctx.accounts.seller.key().eq(&auction_data_info.creator),
                MarketplaceError::CreatorAccountMismatch
            );
        }

        // Get Collection address from Metadata
        let mint_metadata = &mut &ctx.accounts.mint_metadata;
        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Metadata::find_pda(&ctx.accounts.nft_mint.key());
        require!(
            metadata == mint_metadata.key(),
            MarketplaceError::InvaliedMetadata
        );

        // verify metadata is legit
        let nft_metadata =
            Metadata::safe_deserialize(&mut mint_metadata.data.borrow_mut()).unwrap();

        // Assert Buyer User PDA Address
        require!(
            ctx.accounts.buyer.key().eq(&buyer_user_pool.address),
            MarketplaceError::InvalidOwner
        );
        // Assert Seller User PDA Address
        require!(
            ctx.accounts.seller.key().eq(&seller_user_pool.address),
            MarketplaceError::InvalidOwner
        );

        // Assert Already Delisted NFT
        require!(sell_data_info.active == 1, MarketplaceError::NotListedNFT);
        // Assert Seller Pubkey with Sell Data PDA Seller Address
        require!(
            ctx.accounts.seller.key().eq(&sell_data_info.seller),
            MarketplaceError::SellerAccountMismatch
        );

        let offer_data_info = &mut ctx.accounts.offer_data_info;
        // Assert NFT Pubkey with Offer Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&offer_data_info.mint),
            MarketplaceError::InvalidOfferDataMint
        );
        // Assert Buyer Pubkey with Offer Data PDA Buyer Address
        require!(
            ctx.accounts.buyer.key().eq(&offer_data_info.buyer),
            MarketplaceError::InvalidOfferDataBuyer
        );
        // Assert Already Disabled Offer
        require!(offer_data_info.active == 1, MarketplaceError::DisabledOffer);
        // Assert Offer provided date with the NFT Listed Date
        require!(
            offer_data_info.offer_listing_date == sell_data_info.listed_date,
            MarketplaceError::OfferForExpiredListingNFT
        );

        msg!(
            "Offer Mint: {:?}, Seller: {:?}, Buyer: {:?}, Price: {}",
            offer_data_info.mint,
            sell_data_info.seller,
            offer_data_info.buyer,
            offer_data_info.offer_price,
        );

        offer_data_info.active = 0;
        sell_data_info.active = 0;

        if auction_data_info.status == 3 {
            auction_data_info.status = 0;
        }

        require!(
            offer_data_info.offer_price <= buyer_user_pool.escrow_sol_balance,
            MarketplaceError::InsufficientBuyerSolBalance
        );
        buyer_user_pool.escrow_sol_balance -= offer_data_info.offer_price;
        buyer_user_pool.traded_volume += offer_data_info.offer_price;
        seller_user_pool.traded_volume += offer_data_info.offer_price;

        let token_program = &mut &ctx.accounts.token_program;
        let seeds = &[ESCROW_VAULT_SEED.as_bytes(), &[escrow_bump]];
        let signer = &[&seeds[..]];

        let global_authority = &mut ctx.accounts.global_authority;
        let remaining_accounts: Vec<AccountInfo> = ctx.remaining_accounts.to_vec();
        require!(
            global_authority.team_count > 0,
            MarketplaceError::NoTeamTreasuryYet
        );

        let creators: &Vec<Creator>;
        if let Some(cts) = &nft_metadata.creators {
            creators = cts;
        } else {
            return Err(error!(MarketplaceError::MetadataCreatorParseError));
        };
        require!(
            global_authority.team_count + creators.len() as u64 == remaining_accounts.len() as u64,
            MarketplaceError::TeamTreasuryCountMismatch
        );

        let total_share_fee =
            offer_data_info.offer_price * (nft_metadata.seller_fee_basis_points as u64) / PERMYRIAD;
        let fee_amount: u64 =
            offer_data_info.offer_price * global_authority.market_fee_sol / PERMYRIAD;
        let total_fee_amount: u64 = fee_amount + total_share_fee;

        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.escrow_vault.key,
                ctx.accounts.seller.key,
                offer_data_info.offer_price - total_fee_amount,
            ),
            &[
                ctx.accounts.seller.to_account_info().clone(),
                ctx.accounts.escrow_vault.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            signer,
        )?;

        let mut i = 0;
        // This is not expensive cuz the max count is 8
        for team_account in remaining_accounts {
            if i < global_authority.team_count {
                // Assert Provided Remaining Account is Treasury
                require!(
                    team_account
                        .key()
                        .eq(&global_authority.team_treasury[i as usize]),
                    MarketplaceError::TeamTreasuryAddressMismatch
                );
                invoke_signed(
                    &system_instruction::transfer(
                        ctx.accounts.escrow_vault.key,
                        &global_authority.team_treasury[i as usize],
                        fee_amount * global_authority.treasury_rate[i as usize] / PERMYRIAD,
                    ),
                    &[
                        ctx.accounts.escrow_vault.to_account_info().clone(),
                        team_account.clone(),
                        ctx.accounts.system_program.to_account_info().clone(),
                    ],
                    signer,
                )?;
            } else {
                for creator in creators {
                    if creator.address == team_account.key() && creator.share != 0 {
                        let share_amount: u64 = total_share_fee * (creator.share as u64) / 100;
                        invoke_signed(
                            &system_instruction::transfer(
                                ctx.accounts.escrow_vault.key,
                                &team_account.key(),
                                share_amount,
                            ),
                            &[
                                ctx.accounts.escrow_vault.to_account_info().clone(),
                                team_account.clone(),
                                ctx.accounts.system_program.to_account_info().clone(),
                            ],
                            signer,
                        )?;
                    }
                }
            }
            i += 1;
        }
        let dest_nft_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let seller = &ctx.accounts.seller;

        UnlockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .edition(Some(&token_mint_edition.to_account_info()))
            .authority(&global_authority.to_account_info())
            .payer(&seller.to_account_info())
            .mint(&nft_mint.to_account_info())
            .metadata(&mint_metadata.to_account_info())
            .token_record(Some(&dest_token_mint_record.to_account_info()))
            .token_owner(Some(&seller.to_account_info()))
            .token(&dest_nft_token_account_info.to_account_info())
            .authorization_rules(Some(&auth_rules.to_account_info()))
            .spl_token_program(Some(&token_program.to_account_info()))
            .system_program(&system_program.to_account_info())
            .sysvar_instructions(&sysvar_instructions.to_account_info())
            .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
            .system_program(&system_program.to_account_info())
            .invoke_signed(signer)?;
        TransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .authority(&global_authority.to_account_info())
            .payer(&seller.to_account_info())
            .mint(&nft_mint.to_account_info())
            .metadata(&mint_metadata.to_account_info())
            .edition(Some(&token_mint_edition.to_account_info()))
            .destination_token(&token_account_info.to_account_info())
            .destination_owner(&buyer.to_account_info())
            .destination_token_record(Some(&token_mint_record.to_account_info()))
            .token_record(Some(&dest_token_mint_record.to_account_info()))
            .token_owner(&seller.to_account_info())
            .token(&dest_nft_token_account_info.to_account_info())
            .amount(1)
            .authorization_rules(Some(&auth_rules.to_account_info()))
            .sysvar_instructions(&sysvar_instructions.to_account_info())
            .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
            .spl_ata_program(&associated_token_program.to_account_info())
            .spl_token_program(&token_program.to_account_info())
            .system_program(&system_program.to_account_info())
            .invoke_signed(signer)?;

        Ok(())
    }

    pub fn claim_auction_pnft<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimAuctionPNft<'info>>,
        global_bump: u8,
        _auction_bump: u8,
        escrow_bump: u8,
    ) -> Result<()> {
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        msg!("Mint: {:?}", auction_data_info.mint);

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );

        // Get Collection address from Metadata
        let mint_metadata = &mut &ctx.accounts.mint_metadata;
        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Metadata::find_pda(&ctx.accounts.nft_mint.key());
        require!(
            metadata == mint_metadata.key(),
            MarketplaceError::InvaliedMetadata
        );

        // verify metadata is legit
        let nft_metadata =
            Metadata::safe_deserialize(&mut mint_metadata.data.borrow_mut()).unwrap();

        let timestamp = Clock::get()?.unix_timestamp;
        msg!("Claim Date: {}", timestamp);
        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        // Assert Auction End Date is Passed
        // require!(
        //     auction_data_info.get_end_date() <= timestamp,
        //     MarketplaceError::NotEndedAuction
        // );
        // Assert Already Ended or Not Started Auction
        require!(
            auction_data_info.status == 1,
            MarketplaceError::NotListedNFT
        );
        // Assert Creator Pubkey with Auction Data Creator Address
        require!(
            ctx.accounts.creator.key().eq(&auction_data_info.creator),
            MarketplaceError::CreatorAccountMismatch
        );
        // Assert Bidder Pubkey with Auction Data Last Bidder Address
        require!(
            ctx.accounts.bidder.key().eq(&auction_data_info.last_bidder),
            MarketplaceError::BidderAccountMismatch
        );

        let bidder_user_pool = &mut ctx.accounts.bidder_user_pool;
        let creator_user_pool = &mut ctx.accounts.creator_user_pool;
        // Assert Bidder User PDA Address
        require!(
            ctx.accounts.bidder.key().eq(&bidder_user_pool.address),
            MarketplaceError::BidderAccountMismatch
        );
        // Assert Creator User PDA Address
        require!(
            ctx.accounts.creator.key().eq(&creator_user_pool.address),
            MarketplaceError::CreatorAccountMismatch
        );

        // Set Flag as Claimed Auction
        auction_data_info.status = 2;
        bidder_user_pool.traded_volume += auction_data_info.highest_bid;
        creator_user_pool.traded_volume += auction_data_info.highest_bid;

        let token_program = &mut &ctx.accounts.token_program;
        let dest_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let seeds = &[ESCROW_VAULT_SEED.as_bytes(), &[escrow_bump]];
        let signer = &[&seeds[..]];

        let global_authority = &mut ctx.accounts.global_authority;
        let remaining_accounts: Vec<AccountInfo> = ctx.remaining_accounts.to_vec();
        require!(
            global_authority.team_count > 0,
            MarketplaceError::NoTeamTreasuryYet
        );

        let creators: &Vec<Creator>;
        if let Some(cts) = &nft_metadata.creators {
            creators = cts;
        } else {
            return Err(error!(MarketplaceError::MetadataCreatorParseError));
        };
        require!(
            global_authority.team_count + creators.len() as u64 == remaining_accounts.len() as u64,
            MarketplaceError::TeamTreasuryCountMismatch
        );
        let total_share_fee = auction_data_info.highest_bid
            * (nft_metadata.seller_fee_basis_points as u64)
            / PERMYRIAD;
        let fee_amount: u64 =
            auction_data_info.highest_bid * global_authority.market_fee_sol / PERMYRIAD;
        let total_fee_amount: u64 = total_share_fee + fee_amount;
        msg!("ix1");
        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.escrow_vault.key,
                ctx.accounts.creator.key,
                auction_data_info.highest_bid - total_fee_amount,
            ),
            &[
                ctx.accounts.escrow_vault.to_account_info().clone(),
                ctx.accounts.creator.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            signer,
        )?;

        let mut i = 0;
        // This is not expensive cuz the max count is 8
        for team_account in remaining_accounts {
            msg!("ix:{}", i);
            if i < global_authority.team_count {
                // Assert Provided Remaining Account is Treasury
                require!(
                    team_account
                        .key()
                        .eq(&global_authority.team_treasury[i as usize]),
                    MarketplaceError::TeamTreasuryAddressMismatch
                );
                msg!("ix2.1exec");
                invoke_signed(
                    &system_instruction::transfer(
                        ctx.accounts.escrow_vault.key,
                        &global_authority.team_treasury[i as usize],
                        fee_amount * global_authority.treasury_rate[i as usize] / PERMYRIAD,
                    ),
                    &[
                        ctx.accounts.escrow_vault.to_account_info().clone(),
                        team_account.clone(),
                        ctx.accounts.system_program.to_account_info().clone(),
                    ],
                    signer,
                )?;
            } else {
                for creator in creators {
                    if creator.address == team_account.key() && creator.share != 0 {
                        let share_amount: u64 = total_share_fee * (creator.share as u64) / 100;
                        msg!("ix2.2exec");
                        invoke_signed(
                            &system_instruction::transfer(
                                ctx.accounts.escrow_vault.key,
                                &team_account.key(),
                                share_amount,
                            ),
                            &[
                                ctx.accounts.escrow_vault.to_account_info().clone(),
                                team_account.clone(),
                                ctx.accounts.system_program.to_account_info().clone(),
                            ],
                            signer,
                        )?;
                    }
                }
            }
            i += 1;
        }
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let global_authority = &ctx.accounts.global_authority;
        let owner = &ctx.accounts.bidder;
        let token_account_info = &ctx.accounts.user_token_account;
        let nft_mint = &ctx.accounts.nft_mint;

        let token_mint_edition = &ctx.accounts.token_mint_edition;
        let token_mint_record = &ctx.accounts.token_mint_record;
        let dest_token_mint_record = &ctx.accounts.dest_token_mint_record;
        let system_program = &ctx.accounts.system_program;
        let sysvar_instructions = &ctx.accounts.sysvar_instructions;
        let associated_token_program = &ctx.accounts.associated_token_program;
        let auth_rules_program = &ctx.accounts.auth_rules_program;
        let auth_rules = &ctx.accounts.auth_rules;
        let creator = &ctx.accounts.creator;
        msg!("ix3");
        UnlockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .edition(Some(&token_mint_edition.to_account_info()))
            .authority(&global_authority.to_account_info())
            .payer(&owner.to_account_info())
            .mint(&nft_mint.to_account_info())
            .metadata(&mint_metadata.to_account_info())
            .token_record(Some(&dest_token_mint_record.to_account_info()))
            .token_owner(Some(&creator.to_account_info()))
            .token(&dest_token_account_info.to_account_info())
            .authorization_rules(Some(&auth_rules.to_account_info()))
            .spl_token_program(Some(&token_program.to_account_info()))
            .system_program(&system_program.to_account_info())
            .sysvar_instructions(&sysvar_instructions.to_account_info())
            .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
            .system_program(&system_program.to_account_info())
            .invoke_signed(signer)?;
        msg!("ix4");
        TransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            .authority(&global_authority.to_account_info())
            .payer(&owner.to_account_info())
            .mint(&nft_mint.to_account_info())
            .metadata(&mint_metadata.to_account_info())
            .edition(Some(&token_mint_edition.to_account_info()))
            .destination_token(&token_account_info.to_account_info())
            .destination_owner(&owner.to_account_info())
            .destination_token_record(Some(&token_mint_record.to_account_info()))
            .token_record(Some(&dest_token_mint_record.to_account_info()))
            .token_owner(&creator.to_account_info())
            .token(&dest_token_account_info.to_account_info())
            .amount(1)
            .authorization_rules(Some(&auth_rules.to_account_info()))
            .sysvar_instructions(&sysvar_instructions.to_account_info())
            .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
            .spl_ata_program(&associated_token_program.to_account_info())
            .spl_token_program(&token_program.to_account_info())
            .system_program(&system_program.to_account_info())
            .invoke_signed(signer)?;

        Ok(())
    }

    pub fn cancel_auction_pnft(
        ctx: Context<CancelAuctionPNft>,
        global_bump: u8,
        _auction_bump: u8,
    ) -> Result<()> {
        let auction_data_info = &mut ctx.accounts.auction_data_info;
        let sell_data_info = &mut ctx.accounts.sell_data_info;

        msg!("Mint: {:?}", auction_data_info.mint);

        let timestamp = Clock::get()?.unix_timestamp;
        msg!("Cancel Date: {}", timestamp);
        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        // Assert Auction End Date is passed
        require!(
            auction_data_info.get_end_date() <= timestamp,
            MarketplaceError::NotEndedAuction
        );

        // Assert Already Ended Or Not Started Auction
        require!(
            auction_data_info.status == 1 || auction_data_info.status == 3,
            MarketplaceError::NotListedNFT
        );
        // Assert Auction Has No Bidder
        require!(
            Pubkey::default().eq(&auction_data_info.last_bidder),
            MarketplaceError::AuctionHasBid
        );
        // Assert Creator Pubkey is same with the Auction Data Creator
        require!(
            ctx.accounts.creator.key().eq(&auction_data_info.creator),
            MarketplaceError::CreatorAccountMismatch
        );

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );
        if sell_data_info.active == 1 {
            // Assert NFT seller is payer
            require!(
                ctx.accounts.creator.key().eq(&sell_data_info.seller),
                MarketplaceError::SellerMismatch
            );
        }

        auction_data_info.status = 0;

        // let dest_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;
        let seeds: &[&[u8]; 2] = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];

        // Get Collection address from Metadata
        let mint_metadata: &AccountInfo = &ctx.accounts.mint_metadata;
        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Metadata::find_pda(&ctx.accounts.nft_mint.key());

        require!(
            metadata == mint_metadata.key(),
            MarketplaceError::InvaliedMetadata
        );

        let global_authority = &ctx.accounts.global_authority;
        let owner = &ctx.accounts.creator;
        let token_account_info = &ctx.accounts.user_token_account;
        let nft_mint = &ctx.accounts.nft_mint;

        let token_mint_edition = &ctx.accounts.token_mint_edition;
        let token_mint_record = &ctx.accounts.token_mint_record;
        let system_program = &ctx.accounts.system_program;
        let sysvar_instructions = &ctx.accounts.sysvar_instructions;
        let auth_rules_program = &ctx.accounts.auth_rules_program;
        let auth_rules = &ctx.accounts.auth_rules;

        if sell_data_info.active == 0 {
            if mint_metadata.owner != ctx.accounts.token_metadata_program.key {
                msg!("Metadata Owner Err: {:?}", mint_metadata.owner);
            }
            if nft_mint.owner != ctx.accounts.token_program.key {
                msg!("NFT Mint Owner Err: {:?}", nft_mint.owner);
            }
            if token_account_info.owner != *ctx.accounts.token_program.key {
                msg!("Token Owner Err: {:?}", token_account_info.owner);
            }

            if token_mint_record.owner != ctx.accounts.token_metadata_program.key {
                msg!("Token Mint Record Owner Err: {:?}", token_mint_record.owner);
            }
            if token_mint_edition.owner != ctx.accounts.token_metadata_program.key {
                msg!(
                    "Token Mint Edition Owner Err: {:?}",
                    token_mint_edition.owner
                );
            }
            if auth_rules.owner != ctx.accounts.auth_rules_program.key {
                msg!("Auth Rules O");
            }
            UnlockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                .edition(Some(&token_mint_edition.to_account_info()))
                .spl_token_program(Some(&token_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .authority(&global_authority.to_account_info())
                .payer(&owner.to_account_info())
                .mint(&nft_mint.to_account_info())
                .metadata(&mint_metadata.to_account_info())
                .token(&token_account_info.to_account_info())
                .token_owner(Some(&owner.to_account_info()))
                .token_record(Some(&token_mint_record.to_account_info()))
                .authorization_rules(Some(&auth_rules.to_account_info()))
                .sysvar_instructions(&sysvar_instructions.to_account_info())
                .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .invoke_signed(signer)?;
            RevokeLockedTransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                .master_edition(Some(&token_mint_edition.to_account_info()))
                .spl_token_program(Some(&token_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .authority(&owner.to_account_info())
                .delegate(&global_authority.to_account_info())
                .payer(&owner.to_account_info())
                .mint(&nft_mint.to_account_info())
                .metadata(&mint_metadata.to_account_info())
                .token_record(Some(&token_mint_record.to_account_info()))
                .token(&token_account_info.to_account_info())
                .authorization_rules(Some(&auth_rules.to_account_info()))
                .sysvar_instructions(&sysvar_instructions.to_account_info())
                .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                .invoke_signed(signer)?;
        }

        Ok(())
    }

    // pub fn cancel_bid(
    //     ctx: Context<CancelBid>,
    //     _auction_bump: u8,
    //     _escrow_bump: u8,
    //     _user_bump: u8,
    // ) -> Result<()> {
    //     let auction_data_info = &mut ctx.accounts.auction_data_info;
    //     msg!(
    //         "Mint: {:?}, buyer: {:?}",
    //         auction_data_info.mint,
    //         ctx.accounts.owner.key()
    //     );

    //     // Assert NFT Pubkey with Offer Data PDA Mint
    //     require!(
    //         ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
    //         MarketplaceError::InvalidOfferDataMint
    //     );
    //     // Asser Payer is the Offer Data Address
    //     require!(
    //         ctx.accounts.owner.key().eq(&auction_data_info.creator),
    //         MarketplaceError::InvalidOfferDataBuyer
    //     );
    //     require!(
    //         auction_data_info.status == 1,
    //         MarketplaceError::DisabledOffer
    //     );

    //     let seeds = &[ESCROW_VAULT_SEED.as_bytes(), &[_escrow_bump]];
    //     let signer = &[&seeds[..]];

    //     invoke_signed(
    //         &system_instruction::transfer(
    //             ctx.accounts.escrow_vault.key,
    //             ctx.accounts.owner.key,
    //             auction_data_info.price,
    //         ),
    //         &[
    //             ctx.accounts.owner.to_account_info().clone(),
    //             ctx.accounts.escrow_vault.to_account_info().clone(),
    //             ctx.accounts.system_program.to_account_info().clone(),
    //         ],
    //         signer,
    //     )?;

    //     auction_data_info.status = 0;
    //     Ok(())
    // }

    pub fn create_auction_pnft(
        ctx: Context<CreateAuctionPNft>,
        _global_bump: u8,
        _auction_bump: u8,
        _sell_bump: u8,
        start_price: u64,
        min_increase: u64,
        duration: i64,
        reserved: u8,
    ) -> Result<()> {
        require!(reserved < 2, MarketplaceError::InvalidParamInput);

        let auction_data_info = &mut ctx.accounts.auction_data_info;
        msg!("Mint: {:?}, Reserved: {}", auction_data_info.mint, reserved);

        // Assert NFT Pubkey with Auction Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&auction_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );

        let sell_data_info = &mut ctx.accounts.sell_data_info;

        // Assert NFT Pubkey with Sell Data PDA Mint
        require!(
            ctx.accounts.nft_mint.key().eq(&sell_data_info.mint),
            MarketplaceError::InvalidNFTDataAcount
        );

        // Shouldn't listed to create normal auction creation
        if reserved == 0 {
            require!(sell_data_info.active == 0, MarketplaceError::NotListedNFT);
        }
        // Assert NFT seller is payer
        if sell_data_info.active == 1 {
            require!(
                ctx.accounts.owner.key().eq(&sell_data_info.seller),
                MarketplaceError::SellerMismatch
            );
        }
        // Get Collection address from Metadata
        let mint_metadata: &AccountInfo = &ctx.accounts.mint_metadata;
        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Metadata::find_pda(&ctx.accounts.nft_mint.key());

        require!(
            metadata == mint_metadata.key(),
            MarketplaceError::InvaliedMetadata
        );

        // verify metadata is legit
        let nft_metadata =
            Metadata::safe_deserialize(&mut mint_metadata.data.borrow_mut()).unwrap();

        if let Some(creators) = nft_metadata.creators {
            let mut collection: Pubkey = Pubkey::default();
            for creator in creators {
                if creator.verified {
                    collection = creator.address;
                    break;
                }
            }
            sell_data_info.collection = collection;
            msg!("Collection= {:?}", collection);
        } else {
            return Err(error!(MarketplaceError::MetadataCreatorParseError));
        }

        let timestamp = Clock::get()?.unix_timestamp;
        msg!("Created Date: {}", timestamp);

        auction_data_info.creator = ctx.accounts.owner.key();
        auction_data_info.start_price = start_price;
        auction_data_info.min_increase_amount = min_increase;
        auction_data_info.duration = duration;
        auction_data_info.last_bidder = Pubkey::default();
        auction_data_info.highest_bid = start_price - auction_data_info.min_increase_amount;
        auction_data_info.status = 3;

        if reserved == 0 {
            auction_data_info.status = 1;
            auction_data_info.start_date = timestamp;
        }

        let token_account_info = &ctx.accounts.user_token_account;
        // let dest_token_account_info = &ctx.accounts.dest_nft_token_account;
        let owner: &Signer = &ctx.accounts.owner;
        let nft_mint = &ctx.accounts.nft_mint;
        let global_authority = &ctx.accounts.global_authority;
        let token_mint_record = &ctx.accounts.token_mint_record;
        let token_mint_edition = &ctx.accounts.token_mint_edition;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let sysvar_instructions = &ctx.accounts.sysvar_instructions;
        let auth_rules_program = &ctx.accounts.auth_rules_program;
        let auth_rules = &ctx.accounts.auth_rules;
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[_global_bump]];
        let signer = &[&seeds[..]];

        // Transfer NFT only Not listed
        if sell_data_info.active == 0 {
            // Assert NFT is in user Account
            require!(
                token_account_info.amount == 1,
                MarketplaceError::NFTIsNotInUserATA
            );
            if mint_metadata.owner != ctx.accounts.token_metadata_program.key {
                msg!("Metadata Owner Err: {:?}", mint_metadata.owner);
            }
            if nft_mint.owner != ctx.accounts.token_program.key {
                msg!("NFT Mint Owner Err: {:?}", nft_mint.owner);
            }
            if &token_account_info.owner != ctx.accounts.token_program.key {
                msg!("Token Owner Err: {:?}", token_account_info.owner);
            }

            if token_mint_record.owner != ctx.accounts.token_metadata_program.key {
                msg!("Token Mint Record Owner Err: {:?}", token_mint_record.owner);
            }
            if auth_rules.owner != ctx.accounts.auth_rules_program.key {
                msg!("Auth Rules Owner Err: {:?}", auth_rules.owner);
            }
            DelegateLockedTransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                .amount(1)
                .master_edition(Some(&token_mint_edition.to_account_info()))
                .spl_token_program(Some(&token_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .authority(&owner.to_account_info())
                .delegate(&global_authority.to_account_info())
                .payer(&owner.to_account_info())
                .mint(&nft_mint.to_account_info())
                .metadata(&mint_metadata.to_account_info())
                .token_record(Some(&token_mint_record.to_account_info()))
                .token(&token_account_info.to_account_info())
                .amount(1)
                .authorization_rules(Some(&auth_rules.to_account_info()))
                .sysvar_instructions(&sysvar_instructions.to_account_info())
                .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .locked_address(token_account_info.key())
                .invoke_signed(signer)?;
            LockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
                .edition(Some(&token_mint_edition.to_account_info()))
                .spl_token_program(Some(&token_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .authority(&global_authority.to_account_info())
                .payer(&owner.to_account_info())
                .mint(&nft_mint.to_account_info())
                .metadata(&mint_metadata.to_account_info())
                .token(&token_account_info.to_account_info())
                .token_owner(Some(&owner.to_account_info()))
                .token_record(Some(&token_mint_record.to_account_info()))
                .authorization_rules(Some(&auth_rules.to_account_info()))
                .sysvar_instructions(&sysvar_instructions.to_account_info())
                .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
                .system_program(&system_program.to_account_info())
                .invoke_signed(signer)?;
            // TransferV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
            //     .authority(&owner.to_account_info())
            //     .payer(&owner.to_account_info())
            //     .mint(&nft_mint.to_account_info())
            //     .metadata(&mint_metadata.to_account_info())
            //     .edition(Some(&token_mint_edition.to_account_info()))
            //     .token(&token_account_info.to_account_info())
            //     .token_owner(&owner.to_account_info())
            //     .token_record(Some(&token_mint_record.to_account_info()))
            //     .destination_token_record(Some(&dest_token_mint_record.to_account_info()))
            //     .destination_owner(&global_authority.to_account_info())
            //     .destination_token(&dest_token_account_info.to_account_info())
            //     .amount(1)
            //     .authorization_rules(Some(&auth_rules.to_account_info()))
            //     .sysvar_instructions(&sysvar_instructions.to_account_info())
            //     .authorization_rules_program(Some(&auth_rules_program.to_account_info()))
            //     .spl_ata_program(&associated_token_program.to_account_info())
            //     .spl_token_program(&token_program.to_account_info())
            //     .system_program(&system_program.to_account_info())
            //     .invoke_signed(signer)?;
            // let cpi_accounts = Transfer {
            //     from: token_account_info.to_account_info().clone(),
            //     to: dest_token_account_info.to_account_info().clone(),
            //     authority: ctx.accounts.owner.to_account_info().clone(),
            // };
            // token::transfer(
            //     CpiContext::new(token_program.clone().to_account_info(), cpi_accounts),
            //     1,
            // )?;
        } else {
            // Not needed since we moved to delegates

            // Assert NFT is in escrow Account
            // require!(
            //     dest_token_account_info.amount == 1,
            //     MarketplaceError::NFTIsNotInEscrowATA
            // );
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
        space = 8 + 368,
        payer = admin
    )]
    pub global_authority: Account<'info, GlobalPool>,
    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SetTreshold<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,
}
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct AddTreasury<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,
}
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct RemoveTreasury<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitUserPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        seeds = [USER_DATA_SEED.as_ref(), owner.key().as_ref()],
        bump,
        space = 8 + 48,
        payer = owner,
    )]
    pub user_pool: Account<'info, UserData>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub user_pool: Account<'info, UserData>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub user_pool: Account<'info, UserData>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nft: Pubkey, bump: u8)]
pub struct InitSellData<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        seeds = [SELL_DATA_SEED.as_ref(), nft.to_bytes().as_ref()],
        bump,
        space = 8 + 120,
        payer = payer,
    )]
    pub sell_data_info: Account<'info, SellData>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ListPNftForSale<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Account<'info, SellData>,

    #[account(
        mut,
        constraint = user_token_account.mint == nft_mint.key(),
        constraint = user_token_account.owner == *owner.key,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // #[account(
    //     init_if_needed,
    //     associated_token::mint = nft_mint,
    //     associated_token::authority = global_authority,
    //     payer = owner,
    // )]
    // pub dest_nft_token_account: Account<'info, TokenAccount>,
    /// CHECK: legacy pre delegates will be removed
    pub dest_nft_token_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
    /// the mint metadata
    #[account(
        mut,
        constraint = mint_metadata.owner == &mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub dest_token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: this account is safe
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Account<'info, AuctionData>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct DelistPNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Account<'info, SellData>,

    #[account(
        mut,
        constraint = user_token_account.mint == nft_mint.key(),
        constraint = user_token_account.owner == *owner.key,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // #[account(
    //     mut,
    //     constraint = dest_nft_token_account.mint == nft_mint.key(),
    //     constraint = dest_nft_token_account.owner == global_authority.key(),
    //     constraint = dest_nft_token_account.amount == 1,
    // )]
    // pub dest_nft_token_account: Account<'info, TokenAccount>,
    /// CHECK: legacy pre delegates will be removed
    pub dest_nft_token_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
    /// the mint metadata
    #[account(
            mut,
            constraint = mint_metadata.owner == &mpl_token_metadata::ID
        )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,

    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub dest_token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: this account is safe
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Box<Account<'info, AuctionData>>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SetPrice<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Account<'info, SellData>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct PurchasePNft<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Box<Account<'info, SellData>>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub buyer_user_pool: Account<'info, UserData>,

    // #[account(
    //     mut,
    //     constraint = auction_data_info.creator==creator.key(),
    // )]
    // /// CHECK: This is not dangerous because we don't read or write from this account
    // pub creator: AccountInfo<'info>,
    #[account(
        mut,
        constraint = user_nft_token_account.mint == nft_mint.key(),
        constraint = user_nft_token_account.owner == *buyer.key,
    )]
    pub user_nft_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = dest_nft_token_account.mint == nft_mint.key(),
        constraint = dest_nft_token_account.owner == sell_data_info.seller.key(),
        constraint = dest_nft_token_account.amount == 1,
    )]
    pub dest_nft_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub seller: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), seller.key().as_ref()],
        bump,
    )]
    pub seller_user_pool: Account<'info, UserData>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
    /// the mint metadata
    #[account(
        mut,
        constraint = mint_metadata.owner == &mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub dest_token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: this account is safe
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Box<Account<'info, AuctionData>>,
}

#[derive(Accounts)]
#[instruction(nft: Pubkey, bump: u8)]
pub struct InitOfferData<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        seeds = [OFFER_DATA_SEED.as_ref(), nft.to_bytes().as_ref(), payer.key().to_bytes().as_ref()],
        bump,
        space = 8 + 88,
        payer = payer,
    )]
    pub offer_data_info: Account<'info, OfferData>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Account<'info, SellData>,

    #[account(
        mut,
        seeds = [OFFER_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref(), owner.key().to_bytes().as_ref()],
        bump,
    )]
    pub offer_data_info: Account<'info, OfferData>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub user_pool: Account<'info, UserData>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [OFFER_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref(), owner.key().to_bytes().as_ref()],
        bump,
    )]
    pub offer_data_info: Account<'info, OfferData>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub user_pool: Box<Account<'info, UserData>>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CancelBid<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref(), owner.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Account<'info, AuctionData>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub user_pool: Box<Account<'info, UserData>>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct AcceptOfferPNft<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Box<Account<'info, SellData>>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub buyer: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [OFFER_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref(), buyer.key().to_bytes().as_ref()],
        bump,
    )]
    pub offer_data_info: Box<Account<'info, OfferData>>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), seller.key().as_ref()],
        bump,
    )]
    pub seller_user_pool: Box<Account<'info, UserData>>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub buyer_user_pool: Box<Account<'info, UserData>>,

    #[account(
        mut,
        constraint = user_nft_token_account.mint == nft_mint.key(),
        constraint = user_nft_token_account.owner == *buyer.key,
    )]
    pub user_nft_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = dest_nft_token_account.mint == nft_mint.key(),
        constraint = dest_nft_token_account.owner == seller.key(),
        constraint = dest_nft_token_account.amount == 1,
    )]
    pub dest_nft_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,

    /// the mint metadata
    #[account(
        mut,
        constraint = mint_metadata.owner == &mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub dest_token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: this account is safe
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Box<Account<'info, AuctionData>>,
}

#[derive(Accounts)]
#[instruction(nft: Pubkey, bump: u8)]
pub struct InitAuctionData<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft.to_bytes().as_ref()],
        bump,
        space = 8 + 152,
        payer = payer,
    )]
    pub auction_data_info: Account<'info, AuctionData>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateAuctionPNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Account<'info, AuctionData>,

    #[account(
        mut,
        constraint = user_token_account.mint == nft_mint.key(),
        constraint = user_token_account.owner == *owner.key,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // #[account(
    //     mut,
    //     constraint = dest_nft_token_account.mint == nft_mint.key(),
    //     constraint = dest_nft_token_account.owner == global_authority.key(),
    // )]
    // pub dest_nft_token_account: Account<'info, TokenAccount>,
    /// CHECK: legacy pre delegates will be removed
    pub dest_nft_token_account: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
    /// the mint metadata
    #[account(
    mut,
    constraint = mint_metadata.owner == &mpl_token_metadata::ID
)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub dest_token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: this account is safe
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Account<'info, SellData>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Account<'info, AuctionData>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub out_bidder: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Box<Account<'info, SellData>>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct ClaimAuctionPNft<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Account<'info, AuctionData>,

    #[account(
        mut,
        constraint = user_token_account.mint == nft_mint.key(),
        constraint = user_token_account.owner == *bidder.key,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = dest_nft_token_account.mint == nft_mint.key(),
        constraint = dest_nft_token_account.owner == auction_data_info.creator.key(),
        constraint = dest_nft_token_account.amount == 1,
    )]
    pub dest_nft_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = auction_data_info.creator==creator.key(),
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub creator: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED.as_ref()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub escrow_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), bidder.key().as_ref()],
        constraint = bidder_user_pool.address == bidder.key(),
        bump,
    )]
    pub bidder_user_pool: Box<Account<'info, UserData>>,

    #[account(
        mut,
        seeds = [USER_DATA_SEED.as_ref(), creator.key().as_ref()],
        bump,
    )]
    pub creator_user_pool: Box<Account<'info, UserData>>,

    /// the mint metadata
    #[account(
        mut,
        constraint = mint_metadata.owner == &mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub dest_token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: this account is safe
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct UpdateReserve<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Box<Account<'info, AuctionData>>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CancelAuctionPNft<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [AUCTION_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub auction_data_info: Box<Account<'info, AuctionData>>,

    #[account(
        mut,
        constraint = user_token_account.mint == nft_mint.key(),
        constraint = user_token_account.owner == *creator.key,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    // #[account(
    //     mut,
    //     constraint = dest_nft_token_account.mint == nft_mint.key(),
    //     constraint = dest_nft_token_account.owner == global_authority.key(),
    //     constraint = dest_nft_token_account.amount == 1,
    // )]
    // pub dest_nft_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: legacy pre delegates will be removed
    pub dest_nft_token_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint: AccountInfo<'info>,
    /// the mint metadata
    #[account(
            mut,
            constraint = mint_metadata.owner == &mpl_token_metadata::ID
        )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub mint_metadata: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,

    /// CHECK instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong record is supplied
    #[account(mut)]
    pub dest_token_mint_record: AccountInfo<'info>,

    /// CHECK instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    /// CHECK: this account is safe
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint = token_metadata_program.key == &mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [SELL_DATA_SEED.as_ref(), nft_mint.key().to_bytes().as_ref()],
        bump,
    )]
    pub sell_data_info: Account<'info, SellData>,
}
