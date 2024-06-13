import * as anchor from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Program, web3 } from '@project-serum/anchor';
import {
    PublicKey,
    Connection,
    SystemProgram,
    SYSVAR_RENT_PUBKEY,
    Transaction,
    PartiallyDecodedInstruction,
    ParsedInstruction
} from '@solana/web3.js';
import { Buffer } from 'node:buffer';
import fs from 'fs';
import path from 'path';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { decode } from '@project-serum/anchor/dist/cjs/utils/bytes/base64';
import { bs58 } from '@project-serum/anchor/dist/cjs/utils/bytes';

anchor.setProvider(anchor.AnchorProvider.local(web3.clusterApiUrl("devnet")));
const solConnection = anchor.getProvider().connection;
const payer = anchor.AnchorProvider.local().wallet;
var nonce = [
    'AsUkG8p1',
    '3BzKBJUk',
    'HY2XrSxn',
    'AJbPzu2U',
    'PcLYN6YP',
    '4G2WiD2C',
    '4eW8amCV',
    '6Ahuf6jr',
    'UJJfJRLD',
    '5p6KPRCQ',
    'QPHXB9jL',
    'BE4DeeJT', 
    'AnGz4SzU',
    'VcryEr2T',
    '3zENZMgC',
    '2BWUiNsL',
    '2SaBBC7P',
    'ZkCkv1Hg',
    'QMNFmXsk',
    'AtH7do4b',
    '882wV271',
    '9mtdhZPt',
    '9mE5kJpm',
    'Sc9FjYDt',
];

const main = async () => {

    await getAllTransactions(new PublicKey("C29hER4SXQr3atHsuCrRmLAkXBpxvfLMCNeXg2TRTd9o"));
    // 44GoMVetnJTUJUYzbVbydKVU2f6UoK73Hhew7WW76xBRuyqJZwRXNcCt2dmPhnvjPjZmAsVCLoQDTDuKjQoYFZq3
    // 5JUKdSyZBVtiFxapy54dLoUda1muBeLhQNQcMXTrfU3CmrChmvAFbGxoxZW3s8vmpDzm18fCHBn6gmAyStUDw7Dh
}

export const getAllTransactions = async (
    contractId: PublicKey,
) => {
    const data = await solConnection.getSignaturesForAddress(contractId, {}, "confirmed");
    data.map( async (datum) => {
        let tx = await getDataFromSignature(datum.signature);
        console.log(tx);
    })
}

export const getDataFromSignature = async (
    sig: string
) => {
    const tx = await solConnection.getParsedTransaction(sig, 'confirmed');
    let length = tx.transaction.message.instructions.length;
    let valid = -1;
    let hash;
    let ixId;

    for (let i = 0; i < length; i ++) {
        for (let j = 0; j < nonce.length; j ++) {
            hash = (tx.transaction.message.instructions[i] as PartiallyDecodedInstruction).data;
            if (hash != undefined && hash.slice(0, 8) == nonce[j]) {
                valid = j;
                break;
            }
        }
        if (valid > -1) {
            ixId = i;
            break;
        }
    }

    let ts = tx.blockTime;
    let date = new Date(ts * 1000)
    if (valid == -1) return;
        // return {
        //     'type': 'Unknown',
        //     'address': tx.transaction.message.accountKeys[0].pubkey.toBase58(),
        //     'timestamp': ts,
        //     'date': date,
        //     'signature': sig,
        // };
    
    let innerIx;
    if (tx.meta.innerInstructions.length !== 0) {
        innerIx = tx.meta.innerInstructions[ixId].instructions;
    }
    
    let accountKeys = (tx.transaction.message.instructions[ixId] as PartiallyDecodedInstruction).accounts;
    let signer = accountKeys[0].toBase58();
    let result;
    switch (valid) {
        case 0:
            console.log("Initialize");
            result = {'type': "Initialize"}
            break;
        case 1: {
            console.log("UpdateFee");
            result = {'type': "UpdateFee"}
            break;
        }
        case 2:
            console.log("AddTeamTreasury");
            result = {'type': 'AddTeamTreasury'}
            break;
        case 3:
            console.log("RemoveTeamTreasury");
            result = {'type': 'RemoveTeamTreasury'}
            break;
        case 4:{
            console.log("InitUserPool");
            result = {'type': 'InitUserPool'}
            break;
        }
        case 5:
            console.log("InitSellData");
            break;
        case 6: {
            console.log("ListNftForSale");
            let bytes = bs58.decode(hash)
            let b = bytes.slice(10, 18).reverse();
            let sol_price = new anchor.BN(b).toNumber()
            
            result = {
                'type': 'ListNftForSale',
                'address': signer,
                'mint': accountKeys[5].toBase58(),
                'sol_price': sol_price,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 7: { 
            console.log("DelistNft");
            result = {
                'type': 'DelistNft',
                'address': signer,
                'mint': accountKeys[5].toBase58(),
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 8: {
            console.log("Transfer");
            result = {
                'type': 'Transfer',
                'address': signer,
                'recipient': accountKeys[2].toBase58(),
                'mint': accountKeys[4].toBase58(),
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 9:
            console.log("TransferFromVault");
            result = {
                'type': 'TransferFromVault',
                'address': signer,
                'recipient': accountKeys[3].toBase58(),
                'mint': accountKeys[6].toBase58(),
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        case 10:
            console.log("Purchase");
            let price = 0;
            for(let i = 0; i<innerIx.length; i++) {
                let ins = innerIx[i] as ParsedInstruction;
                if (ins.program === 'system' && ins.parsed.type === 'transfer' && ins.parsed.info.source === accountKeys[0].toBase58()) {
                    price += ins.parsed.info.lamports;
                }
            }
            result =  {
                "type": "Purchase",
                "buyer": signer,
                "seller": accountKeys[6].toBase58(),
                "mint": accountKeys[8].toBase58(),
                "price": price,
                "timestamp": ts,
                "date": date,
                "signature": sig
            };
            break;
        case 11:{
            console.log("DepositToEscrow");
            let bytes = bs58.decode(hash)
            let b = bytes.slice(10, 18).reverse();
            let sol_amount = new anchor.BN(b).toNumber();

            result = {
                'type': 'DepositToEscrow',
                'address': signer,
                'sol_amount': sol_amount,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 12:{
            console.log("WithdrawFromEscrow");
            let bytes = bs58.decode(hash)
            let b = bytes.slice(10, 18).reverse();
            let sol_amount = new anchor.BN(b).toNumber();

            result = {
                'type': 'Withdraw',
                'address': signer,
                'sol_amount': sol_amount,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 13: {
            console.log("InitOfferData");
            result = {'type': 'InitOfferData'}
            break;
        }
        case 14:{
            console.log("MakeOffer");
            let price = 0;
            for(let i = 0; i<innerIx.length; i++) {
                let ins = innerIx[i] as ParsedInstruction;
                if (ins.program === 'system' && ins.parsed.type === 'transfer' && ins.parsed.info.source === accountKeys[0].toBase58()) {
                    price += ins.parsed.info.lamports;
                }
            }
            result = {
                'type': 'MakeOffer',
                'address': signer,
                'mint': accountKeys[3].toBase58(),
                'price': price,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 15:{
            console.log("CancelOffer");
            result = {
                'type': 'CancelOffer',
                'address': signer,
                'mint': accountKeys[2].toBase58(),
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 16:{
            console.log("AcceptOffer");
            let price = 0;
            for(let i = 0; i<innerIx.length; i++) {
                let ins = innerIx[i] as ParsedInstruction;
                if (ins.program === 'system' && ins.parsed.type === 'transfer' && ins.parsed.info.source === accountKeys[10].toBase58()) {
                    price += ins.parsed.info.lamports;
                }
            }
            result = {
                'type': 'AcceptOffer',
                'buyer': accountKeys[2].toBase58(),
                'seller': signer,
                'mint': accountKeys[5].toBase58(),
                'price': price,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 17:{
            console.log("InitAuctionData");
            result = {'type': 'InitAuctionData'}
            break;
        }
        case 18: {
            console.log("CreateAuction");
            let bytes = bs58.decode(hash)
            let b = bytes.slice(10, 18).reverse();
            let c = bytes.slice(18, 26).reverse();
            let d = bytes.slice(26, 34).reverse();

            let start_price = new anchor.BN(b).toNumber();
            let min_increase = new anchor.BN(c).toNumber();
            let duration = new anchor.BN(d).toNumber();
            let end_date = ts + duration;

            result = {
                'type': 'CreateAuction',
                'address': signer,
                'mint': accountKeys[5].toBase58(),
                'start_price': start_price,
                'min_increase': min_increase,
                'timestamp': ts,
                'date': date,
                'end_date': end_date,
                'signature': sig,
            }
            break;
        }
        case 19:{
            console.log("PlaceBid");
            let price = 0;
            for(let i = 0; i<innerIx.length; i++) {
                let ins = innerIx[i] as ParsedInstruction;
                if (ins.program === 'system' && ins.parsed.type === 'transfer' && ins.parsed.info.source === accountKeys[0].toBase58()) {
                    price += ins.parsed.info.lamports;
                }
            }
            result = {
                'type': 'PlaceBid',
                'address': signer,
                'mint': accountKeys[2].toBase58(),
                'price': price,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 20: {
            console.log("ClaimAuction");
            let price = 0;
            for(let i = 0; i<innerIx.length; i++) {
                let ins = innerIx[i] as ParsedInstruction;
                if (ins.program === 'system' && ins.parsed.type === 'transfer' && ins.parsed.info.source === accountKeys[6].toBase58()) {
                    price += ins.parsed.info.lamports;
                }
            }
            result = {
                'type' : 'ClaimAuction',
                'buyer': signer,
                'seller': accountKeys[8].toBase58(),
                'mint': accountKeys[5].toBase58(),
                'price': price,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break;
        }
        case 21:{
            console.log("CancelAuction");
            result = {
                'type': 'CancelAuction',
                'address': signer,
                'mint': accountKeys[5].toBase58(),
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break; 
        }
        case 22:{
            console.log("SetPrice");
            let bytes = bs58.decode(hash)
            let b = bytes.slice(9, 17).reverse();
            let price = new anchor.BN(b).toNumber();
            result = {
                'type': 'SetPrice',
                'address': signer,
                'mint': accountKeys[2].toBase58(),
                'price': price,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break; 
        }
        case 23:{
            console.log("UpdateReverse");
            let bytes = bs58.decode(hash)
            let b = bytes.slice(9, 17).reverse();
            let price = new anchor.BN(b).toNumber();
            result = {
                'type': 'UpdateReverse',
                'address': signer,
                'mint': accountKeys[2].toBase58(),
                'price': price,
                'timestamp': ts,
                'date': date,
                'signature': sig,
            }
            break; 
        }
    }
    return result;
}


main()
