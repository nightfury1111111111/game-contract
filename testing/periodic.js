import {
    gamified_airdrop_wallet,

    minting_wallet,
    terraClient,
    private_category_wallet,
} from './constants.js';
import {
    queryContract,
    executeContract,
    readArtifact,
} from "./utils.js";

import { primeAccountsWithFunds } from "./primeCustomAccounts.js";

import { promisify } from 'util';

import * as readline from 'node:readline';

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
const question = promisify(rl.question).bind(rl);


const main = async () => {
    try {
        terraClient.chainID = "localterra";
        let deploymentDetails = readArtifact(terraClient.chainID);
        const primeAccounts = await question('Do you want to preload custom accounts? (y/N) ');
        if (primeAccounts === 'Y' || primeAccounts === 'y') {
            primeAccountsWithFunds().then((txHash) => {
                console.log(txHash);
                proceedToSetup(deploymentDetails);
            });
        } else {
            proceedToSetup(deploymentDetails);
        }
    } catch (error) {
        console.log(error);
    }
}

const proceedToSetup = async (deploymentDetails) => {
    const startFresh = await question('Do you want to upload and deploy fresh? (y/N)');
    if (startFresh === 'Y' || startFresh === 'y') {
        deploymentDetails = {};
    }
    if (!deploymentDetails.adminWallet) {
        deploymentDetails.adminWallet = minting_wallet.key.accAddress;
    }

    performPeriodicDistribution(deploymentDetails).then(() => {
        performPeriodicVesting(deploymentDetails).then(() => {
            claimVestedTokens(deploymentDetails).then(() => {
                console.log("Finished!");
            });
        });
    });
}


const performPeriodicDistribution = async (deploymentDetails) => {
    console.log("Performing periodic distribution");
    let periodicDistributionMsg = { periodically_transfer_to_categories: {} }
    let periodicDistributionResp = await executeContract(minting_wallet, deploymentDetails.vndAddress, periodicDistributionMsg);
    console.log(periodicDistributionResp['txhash']);
}

const performPeriodicVesting = async (deploymentDetails) => {
    console.log("Performing periodic vesting");
    let periodicVestingMsg = { periodically_calculate_vesting: {} };
    let periodicVestingResp = await executeContract(minting_wallet, deploymentDetails.vndAddress, periodicVestingMsg);
    console.log(periodicVestingResp['txhash']);
}

const claimVestedTokens = async (deploymentDetails) => {
    //Get balance of private_category_wallet
    console.log(`Claiming vested tokens for ${private_category_wallet.key.accAddress}`);
    let vesting_details = await queryContract(deploymentDetails.vndAddress, {
        vesting_details: { address: private_category_wallet.key.accAddress }
    });
    console.log(`vesting details of ${private_category_wallet.key.accAddress} : ${JSON.stringify(vesting_details)}`);
    let vestable = vesting_details['tokens_available_to_claim']
    if (vestable > 0) {
        let claimVestedTokensMsg = { claim_vested_tokens: { amount: vestable } };
        let claimVestingResp = await executeContract(private_category_wallet, deploymentDetails.vndAddress, claimVestedTokensMsg);
        console.log(claimVestingResp['txhash']);
    } else {
        console.log("Number of tokens available for claiming = " + vestable);
    }
    //Get balance of private_category_wallet
}

const queryVestingDetailsForGaming = async (deploymentDetails) => {
    let result = await queryContract(deploymentDetails.vndAddress, {
        vesting_details: { address: gamified_airdrop_wallet.key.accAddress }
    });
    console.log(`vesting details of ${gamified_airdrop_wallet.key.accAddress} : ${JSON.stringify(result)}`);
}


const queryPool = async (deploymentDetails) => {
    console.log("querying pool details");
    let poolDetails = await queryContract(deploymentDetails.proxyContractAddress, {
        pool: {}
    });
    console.log(JSON.stringify(poolDetails));
}


main()