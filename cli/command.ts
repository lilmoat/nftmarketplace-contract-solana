#!/usr/bin/env ts-node
import * as dotenv from "dotenv";
import { program } from 'commander';
import {
  PublicKey,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import {
  initProject,
  getGlobalInfo,
  setClusterConfig,
  getNFTPoolInfo,
  listNftForSale,
  delistNft,
  getAllNFTs,
  purchase,
  getUserPoolInfo,
  depositEscrow,
  withdrawEscrow,
  getOfferDataInfo,
  makeOffer,
  cancelOffer,
  acceptOffer,
  getAllOffersForNFT,
  getAllAuctions,
  getAuctionDataInfo,
  claimAuction,
  cancelAuction,
  placeBid,
  createAuction,
  updateFee,
  addTreasury,
  removeTreasury,
  initUserPool,
  transfer,
  setPrice,
  updateReserve,
} from "./scripts";

dotenv.config({ path: __dirname + '/../.env' });

program.version('0.0.1');

programCommand('status')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
    } = cmd.opts();
    console.log('Solana config: ', env);
    await setClusterConfig(env);
    console.log(await getGlobalInfo());
  });

programCommand('user_status')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .option('-a, --address <string>', 'nft user pubkey')
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();
    console.log('Solana config: ', env);
    await setClusterConfig(env);
    if (address === undefined) {
      console.log("Error User Address input");
      return;
    }
    console.log(await getUserPoolInfo(new PublicKey(address)));
  });

programCommand('update_fee')
  .option('-s, --sol_fee <number>', 'marketplace trading by sol fee as permyraid')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      sol_fee,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (sol_fee === undefined || isNaN(parseInt(sol_fee))) {
      console.log("Error Sol Fee Input");
      return;
    }

    await updateFee(parseInt(sol_fee));
  });

programCommand('add_treasury')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .option('-a, --address <string>', 'team treasury account pubkey')
  .option('-r, --rate <number>', 'treasury distribution rate as permyraid')
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      rate,
    } = cmd.opts();
    console.log('Solana config: ', env);
    await setClusterConfig(env);
    if (address === undefined) {
      console.log("Error Treasury input");
      return;
    }
    if (rate === undefined || isNaN(parseInt(rate))) {
      console.log("Error Treasury Rate Input");
      return;
    }
    await addTreasury(new PublicKey(address), parseInt(rate));
  });

programCommand('remove_treasury')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .option('-a, --address <string>', 'team treasury account pubkey')
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();
    console.log('Solana config: ', env);
    await setClusterConfig(env);
    if (address === undefined) {
      console.log("Error Treasury input");
      return;
    }
    await removeTreasury(new PublicKey(address));
  });

programCommand('deposit')
  .option('-s, --sol <number>', 'deposit sol amount')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      sol,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (sol === undefined || isNaN(parseFloat(sol))) {
      console.log("Error Sol Amount input");
      return;
    }

    await depositEscrow(parseFloat(sol) * LAMPORTS_PER_SOL);
  });

programCommand('withdraw')
  .option('-s, --sol <number>', 'withdraw sol amount')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      sol,
      token,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (sol === undefined || isNaN(parseFloat(sol))) {
      console.log("Error Sol Amount input");
      return;
    }

    await withdrawEscrow(parseFloat(sol) * LAMPORTS_PER_SOL);
  });

programCommand('transfer')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-r, --recipient <string>', 'recipient user address')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      recipient,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }

    if (recipient === undefined) {
      console.log("Error Recipint Address input");
      return;
    }

    await transfer(new PublicKey(address), new PublicKey(recipient));
  });

programCommand('list')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-p, --price_sol <number>', 'sell sol price')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      price_sol,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    if (price_sol === undefined || isNaN(parseFloat(price_sol))) {
      console.log("Error Sol Price input");
      return;
    }

    await listNftForSale(new PublicKey(address), parseFloat(price_sol) * LAMPORTS_PER_SOL);
  });

programCommand('delist')
  .option('-a, --address <string>', 'nft mint pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }

    await delistNft(new PublicKey(address));
  });

programCommand('set_price')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-p, --price_sol <number>', 'new sell price')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      price_sol,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    if (price_sol === undefined || isNaN(parseFloat(price_sol))) {
      console.log("Error Sol Price input");
      return;
    }

    await setPrice(new PublicKey(address), parseFloat(price_sol) * LAMPORTS_PER_SOL);
  });

programCommand('purchase')
  .option('-a, --address <string>', 'nft mint pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }

    await purchase(new PublicKey(address));
  });

programCommand('make_offer')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-p, --price <number>', 'offer price')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      price,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    if (price === undefined || isNaN(parseFloat(price))) {
      console.log("Error Offer Price input");
      return;
    }

    await makeOffer(new PublicKey(address), parseFloat(price) * LAMPORTS_PER_SOL);
  });

programCommand('cancel_offer')
  .option('-a, --address <string>', 'nft mint pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }

    await cancelOffer(new PublicKey(address));
  });

programCommand('accept_offer')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-b, --buyer <string>', 'buyer address')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      buyer,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }

    if (buyer === undefined) {
      console.log("Error Buyer input");
      return;
    }

    await acceptOffer(new PublicKey(address), new PublicKey(buyer));
  });

programCommand('create_auction')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-p, --start_price <number>', 'start price')
  .option('-m, --min_increase <number>', 'min increase amount')
  .option('-d, --duration <number>', 'duration by second')
  .option('-r, --reserve <number>', 'reserved auction flag')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      start_price,
      min_increase,
      duration,
      reserve,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    if (start_price === undefined || isNaN(parseFloat(start_price))) {
      console.log("Error Auction Start Price input");
      return;
    }
    if (min_increase === undefined || isNaN(parseFloat(min_increase))) {
      console.log("Error Auction Min Increase Amount input");
      return;
    }
    if (duration === undefined || isNaN(parseInt(duration))) {
      console.log("Error Auction Duration input");
      return;
    }
    if (reserve === undefined || isNaN(parseInt(reserve)) || parseInt(reserve) > 1) {
      console.log("Error Reserve Flag input");
      return;
    }

    await createAuction(
      new PublicKey(address),
      parseFloat(start_price) * LAMPORTS_PER_SOL,
      parseFloat(min_increase) * LAMPORTS_PER_SOL,
      parseInt(duration),
      parseInt(reserve) == 1,
    );
  });

programCommand('place_bid')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-p, --price <number>', 'auction price')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      price,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    if (price === undefined || isNaN(parseFloat(price))) {
      console.log("Error Auction Price input");
      return;
    }

    await placeBid(new PublicKey(address), parseFloat(price) * 1e9);
  });


programCommand('claim_auction')
  .option('-a, --address <string>', 'nft mint pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }

    await claimAuction(new PublicKey(address));
  });

programCommand('update_reserve')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-p, --start_price <number>', 'start price')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      start_price,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    if (start_price === undefined || isNaN(parseFloat(start_price))) {
      console.log("Error Auction Start Price input");
      return;
    }

    await updateReserve(
      new PublicKey(address),
      parseFloat(start_price) * LAMPORTS_PER_SOL,
    );
  });

programCommand('cancel_auction')
  .option('-a, --address <string>', 'nft mint pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }

    await cancelAuction(new PublicKey(address));
  });

programCommand('listed_nft_data')
  .option('-a, --address <string>', 'nft mint pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error input");
      return;
    }
    console.log(await getNFTPoolInfo(new PublicKey(address)));
  });

programCommand('get_offer_data')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-b, --buyer <string>', 'buyer address pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      buyer,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    if (buyer === undefined) {
      console.log("Error Buyer input");
      return;
    }
    console.log(await getOfferDataInfo(new PublicKey(address), new PublicKey(buyer)));
  });

programCommand('get_auction_data')
  .option('-a, --address <string>', 'nft mint pubkey')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error Mint input");
      return;
    }
    console.log(await getAuctionDataInfo(new PublicKey(address)));
  });

programCommand('get_all_listed_nfts')
  .option('-r, --rpc <string>', 'custom rpc url')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      rpc,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    console.log(await getAllNFTs(rpc));
  });

programCommand('get_all_offers_for_nft')
  .option('-a, --address <string>', 'nft mint pubkey')
  .option('-r, --rpc <string>', 'custom rpc url')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      address,
      rpc,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    if (address === undefined) {
      console.log("Error input");
      return;
    }
    console.log(await getAllOffersForNFT(address, rpc));
  });

programCommand('get_all_auctions')
  .option('-r, --rpc <string>', 'custom rpc url')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
      rpc,
    } = cmd.opts();

    console.log('Solana config: ', env);
    await setClusterConfig(env);

    console.log(await getAllAuctions(rpc));
  });

programCommand('init')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
    } = cmd.opts();
    console.log('Solana config: ', env);
    await setClusterConfig(env);

    await initProject();
  });

programCommand('init_user')
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  .action(async (directory, cmd) => {
    const {
      env,
    } = cmd.opts();
    console.log('Solana config: ', env);
    await setClusterConfig(env);
    await initUserPool();
  });

function programCommand(name: string) {
  return program
    .command(name)
    .option(
      '-e, --env <string>',
      'Solana cluster env name',
      'devnet', //mainnet-beta, testnet, devnet
    )
}

program.parse(process.argv);
