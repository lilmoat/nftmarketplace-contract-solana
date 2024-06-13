# Mugs-Marketplace-Contract
Solana NFT Marketplace program with NFT Trading & Auction

## Program Deployment

- Prepare anchor development environments
- Prepare aroun 12 SOL in the deploy wallet keypair
- Confirm Network cluster in `Anchor.toml` file : f.e. `[programs.devnet]`, `cluster = "devnet"`
- Confirm deploy authority wallet keypair location : f.e. `wallet = "/home/ubuntu/deploy-keypair.json"
- Configure solana cli with deploy authority keypair and deploying cluster : f.e. `solana config set -h`
- Build program with `anchor build`
- Copy and paste the result deploy scripts from Build terminal message : f.e. `solana program deploy /home/ubuntu/project/target/deploy/mugs_marketplace.so`

### To Change Program Address

- Delete the program keypair in `/target/deploy/mugs_marketplace-keypair.json`
- Build project with `anchor build`. This will generate new keypair
- Get the address of new keypair with `solana address --keypair ./target/deploy/mugs_marketplace-keypair.json`
- Change program addresses in project code. `Anchor.toml`, `/program/Mugs_Marketplace/src/lib.rs`
- Build program object again with `anchor build`
- Deploy newly built so file with `solana program deploy`

## Cli Command usage

Able to run all commands in `/cli/command.ts` file by running `yarn ts-node xxx`.
When you get this error <br/>
`Error: Provider local is not available on browser.`
You can run this command `export BROWSER=` once.

### Install Dependencies

- Install `node` and `yarn`
- Install `ts-node` as global command
- Confirm the solana wallet preparation in `package.json`: `/home/fury/.config/solana/id.json` in test case

### Init Program

- Initialize program with `init` command
- Should configure the marketplace fee with `update_fee` command
- Should add at least one `treasury` wallet with `add_treasury` command for the fee distribution
- Should Initialize user PDA with `init_user` command for the first time usage

## Commands Help

### init
Initialize Program with creating Global PDA account as Contract Deployer.

### status
Get global PDA info of program. This will show marketplace fee the treasury wallet distributions.

### update_fee
Admin able to update the Marketplace Fee with this command as Admin.
- `sol_fee` is the fee in permyraid

### add_treasury
Admin able to add the team treasury wallet distribution rate for the marketplace fee charge.
- `address` is the treasury wallet
- `rate` is the wallet's distribution rate by permyraid

### remove_treasury
Admin able to remove the team treasury wallet.
- `address` is the treasury wallet

### init_user
Initialize User Data PDA for Escrow Balance & Traded Volume.
This command should be executed for the first time usage of each traders.

### user_status
Get user PDA info for traders. This will show user escrow balance and traded volume info.
- `address` is the trader wallet address

### transfer
Transfer NFT from Sender wallet or it's listed Escrow Account to the Recipient.
- `address` is the NFT mint address
- `recipient` is the recipient wallet address

### list
List NFT for sale as Seller.
- `address` is the NFT mint address
- `price_sol` is the listing price of NFT

### delist
Cancel Listing of NFT as Seller.
- `address` is the NFT mint address

### purchase
Purchase the Listed NFT with `Buy Now` price as Buyer.
- `address` is the NFT mint address

### make_offer
Make offer for a particular Listed NFT as Buyer.
- `address` is the NFT mint address
- `price` is the offering price. Should be in range of `x1 ~ x0.5` of listed price

### cancel_offer
Cancel maden offer for a particular Listed NFT as Buyer.
- `address` is the NFT mint address

### accept_offer
Accpet proper offer from a certain Buyer as Seller.
- `address` is the NFT mint addres
- `buyer` is the Offer provider address

### create_auction
Create Auction for a particular NFT for funny trading as Seller.
- `address` is the NFT mint address
- `start_price` is the bidding start price
- `min_increase` is the minimum increasing amount for the higer bidding
- `duration` is the auction period since started time by second
- `reserve` if this is 1, then the auction is reserve to start from the first bid placed date. Default 0

### palce_bid
Participate in auction with higher bidding as Buyer.
- `address` is the NFT mint address
- `price` is the higher bidding price. Should be more than the latest bid + min_increase_amount

### claim_auction
Claim NFT for winner as Buyer when auction is ended.
- `address` is the NFT mint address

### cancel_auction
Cancel auction as Seller if there is no bid until auction ended.
- `address` is the NFT mint address

### listed_nft_data
Get nft Sell Data PDA info for a particular listed NFT status.
- `address` NFT mint address

### get_offer_data
Get Offer Data PDA info for a particular Offer status.
- `address` NFT mint address
- `buyer` is the offer provider address

### get_auction_data
Get Auction Data PDA info for a particular auction status.
- `address` NFT mint address

### get_all_listed_nfts
Get all listed NFTs info which is active for sale now.

### get_all_offers_for_nft
Get all offers info for a particular NFT which is active for accept now.

### get_all_auctions
Get all auctions info which is live now or not claimed ended auction.

## Notes for FE Integration

For the FE side web3 integration, the scripts in `lib` directory can be use without no change.
The only thing the FE dev should change is providing `web3 connection` & the `anchor program` object from idl.
There is the code part for the `keypair` wallet based `cli` environement case in `cli/scripts`.
Should configure properly in `BROWSER` environment.

## BE Tracking Service Activity Parsing Script
This script will fetch past Txs reacted with Our Marketplace Smartcontract. Then will parse an activity from each Txs so that use the info for DB sync up. \
`yarn be`
