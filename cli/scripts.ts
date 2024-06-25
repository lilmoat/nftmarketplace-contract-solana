import { Program, web3 } from "@project-serum/anchor";
import * as anchor from "@project-serum/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import fs from "fs";
import path from "path";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";

import {
  AuctionData,
  AUCTION_DATA_SEED,
  GlobalPool,
  GLOBAL_AUTHORITY_SEED,
  MARKETPLACE_PROGRAM_ID,
  OfferData,
  OFFER_DATA_SEED,
  SellData,
  SELL_DATA_SEED,
  UserData,
} from "../lib/types";
import { IDL as MarketplaceIDL } from "../target/types/mugs_marketplace";
import {
  createAcceptOfferTx,
  createAddTreasuryTx,
  createCancelAuctionTx,
  createCancelOfferTx,
  createClaimAuctionTx,
  createCreateAuctionTx,
  createDelistNftTx,
  createDelistPNftTx,
  createDepositTx,
  createInitAuctionDataTx,
  createInitializeTx,
  createInitOfferDataTx,
  createInitSellDataTx,
  createInitUserTx,
  createListForSellNftTx,
  createListForSellPNftTx,
  createMakeOfferTx,
  createPlaceBidTx,
  createPurchaseTx,
  createRemoveTreasuryTx,
  createSetPriceTx,
  createTransferFromVaultTx,
  createTransferTx,
  createUpdateFeeTx,
  createUpdateReserveTx,
  createWithdrawTx,
  getAllListedNFTs,
  getAllOffersForListedNFT,
  getAllStartedAuctions,
  getAuctionDataState,
  getGlobalState,
  getNFTPoolState,
  getOfferDataState,
  getUserPoolState,
} from "../lib/scripts";
import { isInitializedUser } from "../lib/utils";

let solConnection = null;
let payer = null;
let program: Program = null;

// Address of the deployed program.
let programId = new anchor.web3.PublicKey(MARKETPLACE_PROGRAM_ID);

export const setClusterConfig = async (cluster: web3.Cluster) => {
  solConnection = new web3.Connection(web3.clusterApiUrl(cluster));
  const walletKeypair = Keypair.fromSecretKey(
    Uint8Array.from(
      JSON.parse(
        fs.readFileSync(path.resolve(process.env.ANCHOR_WALLET), "utf-8")
      )
    ),
    { skipValidation: true }
  );
  const wallet = new NodeWallet(walletKeypair);
  // anchor.setProvider(anchor.AnchorProvider.local(web3.clusterApiUrl(cluster)));
  // Configure the client to use the local cluster.
  anchor.setProvider(
    new anchor.AnchorProvider(solConnection, wallet, {
      skipPreflight: true,
      commitment: "confirmed",
    })
  );
  payer = wallet;

  // Generate the program client from IDL.
  program = new anchor.Program(MarketplaceIDL as anchor.Idl, programId);
  console.log("ProgramId: ", program.programId.toBase58());

  const [globalAuthority, bump] = await PublicKey.findProgramAddress(
    [Buffer.from(GLOBAL_AUTHORITY_SEED)],
    program.programId
  );
  console.log("GlobalAuthority: ", globalAuthority.toBase58());

  // await main();
};

const main = async () => {
  // await initProject();
  // await addCollection(new PublicKey('B2FmSY81mionC1DYvKky5Y8nUXSWnhMmd5L5nbm99VHQ'));
  // await removeCollection(new PublicKey('B2FmSY81mionC1DYvKky5Y8nUXSWnhMmd5L5nbm99VHQ'));
  // const globalPool: GlobalPool = await getGlobalState();
  // console.log("globalPool =", globalPool.superAdmin.toBase58(), globalPool.fullStakePeriod.toNumber()
  // , globalPool.minStakePeriod.toNumber(), globalPool.rewardPeriod.toNumber()
  // , globalPool.rewardAmount.toNumber(), globalPool.totalStakedCount.toNumber(),
  // globalPool.collectionCount.toNumber(), globalPool.collections.slice(0, globalPool.collectionCount.toNumber()).map((addr) => addr.toBase58()));
  // await initUserPool(payer.publicKey);
  // await stakeNft(payer.publicKey, new PublicKey('554AJqCuVFWL7ZHtLqmh18NvuC4UYLP12Bc8fN4RvTex'));
  // await claimReward(payer.publicKey);
  // await withdrawNft(payer.publicKey, new PublicKey('554AJqCuVFWL7ZHtLqmh18NvuC4UYLP12Bc8fN4RvTex'));
  // await burnNft(payer.publicKey, new PublicKey('4Qw3PQqY3q8LbZAYXfwpjUGS4k7anqdRrPyDL9LWue4d'));
  // const userPool: UserPool = await getUserPoolState(new PublicKey('2EnGnSaf89uP6n7prHrKWK9Q41yAWbRkTN1b5ry8XxCw'));
  // console.log({
  //     ...userPool,
  //     owner: userPool.owner.toBase58(),
  //     stakedMints: userPool.stakedMints.slice(0, userPool.stakedCount.toNumber()).map((info) => {
  //         return {
  //             mint: info.mint.toBase58(),
  //             stakedTime: info.stakedTime.toNumber(),
  //         }
  //     })
  // });
  // calculateRewards(new PublicKey('GyjFWXkMn4AGrD5FPfBegP75zmodBeBxr9gBRJjr8qke'));
};

export const initProject = async () => {
  const tx = await createInitializeTx(payer.publicKey, program);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  const simulatieTx = await solConnection.simulateTransaction(tx);
  console.log("tx =====>", simulatieTx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("txHash =", txId);
};

export const initSellData = async (mint: PublicKey) => {
  const tx = await createInitSellDataTx(mint, payer.publicKey, program);
  const { blockhash } = await solConnection.getRecentBlockhash("finalized");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "finalized");
  console.log("Your transaction signature", txId);
};

export const initUserPool = async () => {
  const tx = await createInitUserTx(payer.publicKey, program);
  const { blockhash } = await solConnection.getRecentBlockhash("finalized");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "finalized");
  console.log("Your transaction signature", txId);
};

export const initAuctionData = async (mint: PublicKey) => {
  const tx = await createInitAuctionDataTx(mint, payer.publicKey, program);
  const { blockhash } = await solConnection.getRecentBlockhash("finalized");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "finalized");
  console.log("Your transaction signature", txId);
};

export const updateFee = async (solFee: number) => {
  console.log(solFee);
  const tx = await createUpdateFeeTx(payer.publicKey, program, solFee);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const addTreasury = async (treasury: PublicKey, rate: number) => {
  console.log(treasury.toBase58(), rate);

  const tx = await createAddTreasuryTx(
    payer.publicKey,
    treasury,
    rate,
    program
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const removeTreasury = async (treasury: PublicKey) => {
  console.log(treasury.toBase58());

  const tx = await createRemoveTreasuryTx(payer.publicKey, program, treasury);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const depositEscrow = async (sol: number) => {
  let userAddress = payer.publicKey;
  console.log(userAddress.toBase58(), sol);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createDepositTx(userAddress, sol, program);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const withdrawEscrow = async (sol: number) => {
  let userAddress = payer.publicKey;
  console.log(userAddress.toBase58(), sol);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createWithdrawTx(userAddress, sol, program);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const transfer = async (mint: PublicKey, recipient: PublicKey) => {
  console.log(mint.toBase58(), recipient.toBase58());

  const [sellData, _] = await PublicKey.findProgramAddress(
    [Buffer.from(SELL_DATA_SEED), mint.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Sell Data PDA: ", sellData.toBase58());

  let fromVault = 1;
  let poolAccount = await solConnection.getAccountInfo(sellData);
  if (poolAccount === null || poolAccount.data === null) {
    fromVault = 0;
  } else {
    const data = await getNFTPoolInfo(mint);
    if (data.active != 1) fromVault = 0;
  }

  let tx;
  if (fromVault)
    tx = await createTransferFromVaultTx(
      mint,
      payer.publicKey,
      recipient,
      program,
      solConnection
    );
  else
    tx = await createTransferTx(
      mint,
      payer.publicKey,
      recipient,
      program,
      solConnection
    );

  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const listNftForSale = async (mint: PublicKey, priceSol: number) => {
  console.log(mint.toBase58(), priceSol);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    await initUserPool();
  }

  const [sellData, _] = await PublicKey.findProgramAddress(
    [Buffer.from(SELL_DATA_SEED), mint.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Sell Data PDA: ", sellData.toBase58());

  let poolAccount = await solConnection.getAccountInfo(sellData);
  if (poolAccount === null || poolAccount.data === null) {
    await initSellData(mint);
  }

  const [auctionData] = await PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_DATA_SEED), mint.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Auction Data PDA: ", auctionData.toBase58());

  poolAccount = await solConnection.getAccountInfo(auctionData);
  if (poolAccount === null || poolAccount.data === null) {
    await initAuctionData(mint);
  }

  const tx = await createListForSellNftTx(
    mint,
    payer.publicKey,
    program,
    solConnection,
    priceSol
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const listPNftForSale = async (mint: PublicKey, priceSol: number) => {
  console.log(mint.toBase58(), priceSol);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    await initUserPool();
  }

  const [sellData, _] = await PublicKey.findProgramAddress(
    [Buffer.from(SELL_DATA_SEED), mint.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Sell Data PDA: ", sellData.toBase58());

  let poolAccount = await solConnection.getAccountInfo(sellData);
  if (poolAccount === null || poolAccount.data === null) {
    await initSellData(mint);
  }

  const [auctionData] = await PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_DATA_SEED), mint.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Auction Data PDA: ", auctionData.toBase58());

  poolAccount = await solConnection.getAccountInfo(auctionData);
  if (poolAccount === null || poolAccount.data === null) {
    await initAuctionData(mint);
  }

  const tx = await createListForSellPNftTx(
    mint,
    payer.publicKey,
    program,
    solConnection,
    priceSol
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  const simulatieTx = await solConnection.simulateTransaction(tx);
  console.log("simulatieTx =====>", simulatieTx);
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const delistNft = async (mint: PublicKey) => {
  console.log(mint.toBase58());
  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createDelistNftTx(
    mint,
    payer.publicKey,
    program,
    solConnection
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const pNftDelist = async (mint: PublicKey) => {
  console.log(mint.toBase58());
  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  console.log("delist start");
  const tx = await createDelistPNftTx(
    mint,
    payer.publicKey,
    program,
    solConnection
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const setPrice = async (mint: PublicKey, newPrice: number) => {
  console.log(mint.toBase58(), newPrice);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createSetPriceTx(mint, payer.publicKey, newPrice, program);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const purchase = async (mint: PublicKey) => {
  console.log(mint.toBase58());

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const globalPool: GlobalPool = await getGlobalState(program);

  const tx = await createPurchaseTx(
    mint,
    payer.publicKey,
    globalPool.teamTreasury.slice(0, globalPool.teamCount.toNumber()),
    program,
    solConnection
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const initOfferData = async (mint: PublicKey) => {
  const tx = await createInitOfferDataTx(mint, payer.publicKey, program);
  const { blockhash } = await solConnection.getRecentBlockhash("finalized");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "finalized");
  console.log("Your transaction signature", txId);
};

export const makeOffer = async (mint: PublicKey, price: number) => {
  console.log(mint.toBase58(), price);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const [offerData, _] = await PublicKey.findProgramAddress(
    [Buffer.from(OFFER_DATA_SEED), mint.toBuffer(), payer.publicKey.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Offer Data PDA: ", offerData.toBase58());

  let poolAccount = await solConnection.getAccountInfo(offerData);
  if (poolAccount === null || poolAccount.data === null) {
    await initOfferData(mint);
  }

  const tx = await createMakeOfferTx(mint, payer.publicKey, price, program);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const cancelOffer = async (mint: PublicKey) => {
  console.log(mint.toBase58());

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createCancelOfferTx(mint, payer.publicKey, program);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const acceptOffer = async (mint: PublicKey, buyer: PublicKey) => {
  console.log(mint.toBase58(), buyer.toBase58());

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const globalPool: GlobalPool = await getGlobalState(program);

  const tx = await createAcceptOfferTx(
    mint,
    buyer,
    globalPool.teamTreasury.slice(0, globalPool.teamCount.toNumber()),
    program,
    solConnection
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const createAuction = async (
  mint: PublicKey,
  startPrice: number,
  minIncrease: number,
  duration: number,
  reserved: boolean
) => {
  console.log(mint.toBase58(), startPrice, minIncrease, duration, reserved);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const [sellData, _] = await PublicKey.findProgramAddress(
    [Buffer.from(SELL_DATA_SEED), mint.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Sell Data PDA: ", sellData.toBase58());

  let poolAccount = await solConnection.getAccountInfo(sellData);
  if (poolAccount === null || poolAccount.data === null) {
    await initSellData(mint);
  }

  const [auctionData] = await PublicKey.findProgramAddress(
    [Buffer.from(AUCTION_DATA_SEED), mint.toBuffer()],
    MARKETPLACE_PROGRAM_ID
  );
  console.log("Auction Data PDA: ", auctionData.toBase58());

  poolAccount = await solConnection.getAccountInfo(auctionData);
  if (poolAccount === null || poolAccount.data === null) {
    await initAuctionData(mint);
  }

  const tx = await createCreateAuctionTx(
    mint,
    payer.publicKey,
    startPrice,
    minIncrease,
    duration,
    reserved,
    program,
    solConnection
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const placeBid = async (mint: PublicKey, price: number) => {
  console.log(mint.toBase58(), price);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createPlaceBidTx(mint, payer.publicKey, price, program);
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const claimAuction = async (mint: PublicKey) => {
  console.log(mint.toBase58());

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const globalPool: GlobalPool = await getGlobalState(program);

  const tx = await createClaimAuctionTx(
    mint,
    payer.publicKey,
    globalPool.teamTreasury.slice(0, globalPool.teamCount.toNumber()),
    program,
    solConnection
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const updateReserve = async (mint: PublicKey, newPrice: number) => {
  console.log(mint.toBase58(), newPrice);

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createUpdateReserveTx(
    mint,
    payer.publicKey,
    newPrice,
    program
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const cancelAuction = async (mint: PublicKey) => {
  console.log(mint.toBase58());

  if (!(await isInitializedUser(payer.publicKey, solConnection))) {
    console.log(
      "User PDA is not Initialized. Should Init User PDA for first usage"
    );
    await initUserPool();
  }

  const tx = await createCancelAuctionTx(
    mint,
    payer.publicKey,
    program,
    solConnection
  );
  const { blockhash } = await solConnection.getRecentBlockhash("confirmed");
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = blockhash;
  payer.signTransaction(tx);
  let txId = await solConnection.sendTransaction(tx, [
    (payer as NodeWallet).payer,
  ]);
  await solConnection.confirmTransaction(txId, "confirmed");
  console.log("Your transaction signature", txId);
};

export const getNFTPoolInfo = async (mint: PublicKey) => {
  const nftData: SellData = await getNFTPoolState(mint, program);
  return {
    mint: nftData.mint.toBase58(),
    seller: nftData.seller.toBase58(),
    collection: nftData.collection.toBase58(),
    priceSol: nftData.priceSol.toNumber(),
    listedDate: nftData.listedDate.toNumber(),
    active: nftData.active.toNumber(),
  };
};

export const getOfferDataInfo = async (
  mint: PublicKey,
  userAddress: PublicKey
) => {
  const offerData: OfferData = await getOfferDataState(
    mint,
    userAddress,
    program
  );
  return {
    mint: offerData.mint.toBase58(),
    buyer: offerData.buyer.toBase58(),
    offerPrice: offerData.offerPrice.toNumber(),
    offerListingDate: offerData.offerListingDate.toNumber(),
    active: offerData.active.toNumber(),
  };
};

export const getAuctionDataInfo = async (mint: PublicKey) => {
  const auctionData: AuctionData = await getAuctionDataState(mint, program);
  return {
    mint: auctionData.mint.toBase58(),
    creator: auctionData.creator.toBase58(),
    startPrice: auctionData.startPrice.toNumber(),
    minIncreaseAmount: auctionData.minIncreaseAmount.toNumber(),
    startDate: auctionData.startDate.toNumber(),
    lastBidder: auctionData.lastBidder.toBase58(),
    lastBidDate: auctionData.lastBidDate.toNumber(),
    highestBid: auctionData.highestBid.toNumber(),
    duration: auctionData.duration.toNumber(),
    status: auctionData.status.toNumber(),
  };
};

export const getUserPoolInfo = async (userAddress: PublicKey) => {
  const userData: UserData = await getUserPoolState(userAddress, program);
  return {
    address: userData.address.toBase58(),
    escrowSol: userData.escrowSolBalance.toNumber(),
    tradedVolume: userData.tradedVolume.toNumber(),
  };
};

export const getGlobalInfo = async () => {
  const globalPool: GlobalPool = await getGlobalState(program);
  const result = {
    admin: globalPool.superAdmin.toBase58(),
    marketFeeSol: globalPool.marketFeeSol.toNumber(),
    teamCount: globalPool.teamCount.toNumber(),
    teamTreasury: globalPool.teamTreasury
      .slice(0, globalPool.teamCount.toNumber())
      .map((info) => info.toBase58()),
    treasuryRate: globalPool.treasuryRate
      .slice(0, globalPool.teamCount.toNumber())
      .map((info) => info.toNumber()),
  };

  return result;
};

export const getAllNFTs = async (rpc?: string) => {
  return await getAllListedNFTs(solConnection, rpc);
};

export const getAllOffersForNFT = async (address: string, rpc?: string) => {
  return await getAllOffersForListedNFT(address, solConnection, rpc);
};

export const getAllAuctions = async (rpc?: string) => {
  return await getAllStartedAuctions(solConnection, rpc);
};
