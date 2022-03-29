import {
    walletTest1,
    walletTest2,
    walletTest3,
    walletTest4,
    walletTest5,
    walletTest6,
    walletTest7,
    mint_wallet,
    treasury_wallet,
    liquidity_wallet,
    marketing_wallet,
    nitin_wallet,
    ajay_wallet,
    sameer_wallet
} from './constants.js';

import { MsgSend, LCDClient } from '@terra-money/terra.js';

// To use LocalTerra
const terra = new LCDClient({
    URL: 'http://localhost:1317',
    chainID: 'localterra'
});

export const primeAccountsWithFunds = async () => {
    var txHash = [];
    txHash.push(await fundMintingWallet());
    txHash.push(await fundTreasuryWallet());
    txHash.push(await fundLiquidityWallet());
    txHash.push(await fundMarketingWallet());
    txHash.push(await fundNitinWallet());
    txHash.push(await fundAjayWallet());
    txHash.push(await fundSameerWallet());
    console.log("leaving primeCustomAccounts");
    return txHash;
}

function fundMintingWallet() {
    console.log(`Funding ${mint_wallet.key.accAddress}`);
    return new Promise(resolve => {
        // create a simple message that moves coin balances
        const send1 = new MsgSend(
            walletTest1.key.accAddress,
            mint_wallet.key.accAddress,
            { uluna: 50000000000, uusd: 50000000000 }
        );

        walletTest1
            .createAndSignTx({
                msgs: [send1],
                memo: 'Initial Funding!',
            })
            .then(tx => terra.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}

function fundTreasuryWallet() {
    console.log(`Funding ${treasury_wallet.key.accAddress}`);
    return new Promise(resolve => {
        const send2 = new MsgSend(
            walletTest2.key.accAddress,
            treasury_wallet.key.accAddress,
            { uluna: 50000000000, uusd: 50000000000 }
        );

        walletTest2
            .createAndSignTx({
                msgs: [send2],
                memo: 'Initial Funding!',
            })
            .then(tx => terra.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}

function fundLiquidityWallet() {
    console.log(`Funding ${liquidity_wallet.key.accAddress}`);
    return new Promise(resolve => {
        const send = new MsgSend(
            walletTest3.key.accAddress,
            liquidity_wallet.key.accAddress,
            { uluna: 50000000000, uusd: 50000000000 }
        );

        walletTest3
            .createAndSignTx({
                msgs: [send],
                memo: 'Initial Funding!',
            })
            .then(tx => terra.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}

function fundMarketingWallet() {
    console.log(`Funding ${marketing_wallet.key.accAddress}`);
    return new Promise(resolve => {
        const send = new MsgSend(
            walletTest4.key.accAddress,
            marketing_wallet.key.accAddress,
            { uluna: 50000000000, uusd: 50000000000 }
        );

        walletTest4
            .createAndSignTx({
                msgs: [send],
                memo: 'Initial Funding!',
            })
            .then(tx => terra.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}

function fundNitinWallet() {
    console.log(`Funding ${nitin_wallet.key.accAddress}`);
    return new Promise(resolve => {
        const send = new MsgSend(
            walletTest5.key.accAddress,
            nitin_wallet.key.accAddress,
            { uluna: 50000000000, uusd: 50000000000 }
        );

        walletTest5
            .createAndSignTx({
                msgs: [send],
                memo: 'Initial Funding!',
            })
            .then(tx => terra.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}

function fundAjayWallet() {
    console.log(`Funding ${ajay_wallet.key.accAddress}`);
    return new Promise(resolve => {
        const send = new MsgSend(
            walletTest6.key.accAddress,
            ajay_wallet.key.accAddress,
            { uluna: 50000000000, uusd: 50000000000 }
        );

        walletTest6
            .createAndSignTx({
                msgs: [send],
                memo: 'Initial Funding!',
            })
            .then(tx => terra.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}

function fundSameerWallet() {
    console.log(`Funding ${sameer_wallet.key.accAddress}`);
    return new Promise(resolve => {
        const send = new MsgSend(
            walletTest7.key.accAddress,
            sameer_wallet.key.accAddress,
            { uluna: 50000000000, uusd: 50000000000 }
        );

        walletTest7
            .createAndSignTx({
                msgs: [send],
                memo: 'Initial Funding!',
            })
            .then(tx => terra.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}