import * as readline from 'node:readline';
import { promisify } from 'util';
import { ajay_wallet, ClubStakingContractPath, liquidity_wallet, marketing_wallet, 
    MintingContractPath, mintInitMessage, mint_wallet, nitin_wallet, sameer_wallet, team_wallet, 
    terraClient, treasury_wallet } from './constants.js';
import { primeAccountsWithFunds } from "./primeCustomAccounts.js";
import { executeContract, getGasUsed, instantiateContract, queryContract, readArtifact, // getContract, 
    storeCode, writeArtifact } from './utils.js';

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
const question = promisify(rl.question).bind(rl);

async function main() {
    try {
        terraClient.chainID = "localterra";
        let deploymentDetails = readArtifact(terraClient.chainID);
        console.log(`${JSON.stringify(deploymentDetails)}`);
        await performOperationsOnClubStaking(deploymentDetails);
    } catch (error) {
        console.log(error);
    } finally {
        rl.close();
        console.log(`Total gas used = ${getGasUsed()}`);
    }
}


async function transferFuryToWallets(deploymentDetails) {
    await transferFuryToTreasury(deploymentDetails);
    await transferFuryToLiquidity(deploymentDetails);
    await transferFuryToMarketing(deploymentDetails);
    await transferFuryToNitin(deploymentDetails);
    await transferFuryToAjay(deploymentDetails);
    await transferFuryToSameer(deploymentDetails);
}



async function performOperationsOnClubStaking(deploymentDetails) {
    let dep = deploymentDetails;
    // await queryAllClubOwnerships(dep);
    console.log("clubStakingAddress " + deploymentDetails.clubStakingAddress);
    //await queryAllClubOwnerships(dep);
    // let coResponse = await getContract(deploymentDetails.clubStakingAddress);
    // console.log("clubStaking " + JSON.stringify(coResponse));
    // https://bombay-fcd.terra.dev/v1/wasm/contract/terra1w872cg6mvktvvz7hpvn6t4g7zx3ly80l502uu2    console.log("All clubs ownership = " + JSON.stringify(coResponse));
    await queryBalances(dep, ajay_wallet.key.accAddress);
    await queryBalances(dep, deploymentDetails.clubStakingAddress);
    const startFresh = await question('Buy ClubS club? (y/N)');
    if (startFresh === 'Y' || startFresh === 'y') {
         await buyAClub(dep, ajay_wallet, "ClubS");
    }
    await queryClubStakes(dep, "ClubS");
    await queryBalances(dep, ajay_wallet.key.accAddress);
    await queryBalances(dep, deploymentDetails.clubStakingAddress);
    //await queryClubStakes(dep, "ClubS");
    await queryClubBonds(dep, "ClubS");
    await stakeOnAClub(dep, sameer_wallet, "ClubS", 300000);
	// await distributeRewards(dep, mint_wallet, 10000);
	// await queryClubStakes(dep, "ClubS");
	// await queryClubBonds(dep, "ClubS");
    // await queryAllClubOwnerships(dep);
    // await withdrawTwiceStakeFromAClub(dep, sameer_wallet, "ClubS", 30000, false);
    await withdrawStakeFromAClub(dep, sameer_wallet, "ClubS", 10000, false);
    await withdrawStakeFromAClub(dep, sameer_wallet, "ClubS", 40000, false);
    await queryClubStakes(dep, "ClubS");
    await queryClubBonds(dep, "ClubS");
    await new Promise(resolve => setTimeout(resolve, 120000));
    await withdrawStakeFromAClub(dep, sameer_wallet, "ClubS", 10000, true);
    await withdrawStakeFromAClub(dep, sameer_wallet, "ClubS", 40000, true);
    await queryClubStakes(dep, "ClubS");
    await queryClubBonds(dep, "ClubS");
    await queryBalances(dep, sameer_wallet.key.accAddress);
    await queryBalances(dep, deploymentDetails.clubStakingAddress);

    // console.log("Balances of staker after withdraw stake");
    // await queryBalances(dep, dep.sameerWallet);
    // console.log("Balances of platform_fee_collector after withdraw stake");
    // await queryBalances(dep, dep.adminWallet);
    // console.log("Balances of contract after withdraw stake");
    // await queryBalances(dep, dep.clubStakingAddress);
}

async function queryAllClubOwnerships(deploymentDetails) {
    //Fetch club ownership details for all clubs
    let coResponse = await queryContract(deploymentDetails.clubStakingAddress, {
        all_club_ownership_details: {}
    });
    console.log("All clubs ownership = " + JSON.stringify(coResponse));

}

async function queryClubStakes(deploymentDetails, club_name) {
    //Fetch club Stakes details for all clubs
    let csResponse = await queryContract(deploymentDetails.clubStakingAddress, {
        club_staking_details: {
            club_name: club_name
        }
    });
    console.log("All " + club_name + " stakes = " + JSON.stringify(csResponse));

}

async function queryClubBonds(deploymentDetails, club_name) {
    //Fetch club Stakes details for all clubs
    try {
        let csResponse = await queryContract(deploymentDetails.clubStakingAddress, {
            club_bonding_details: {
                club_name: club_name
            }
        });
        console.log("All " + club_name + " bonds = " + JSON.stringify(csResponse));
    } catch (error) {
        console.log("Error No Bonds " + club_name );
    } finally {
        return
    }
}

async function queryBalances(deploymentDetails, accAddress) {
    let bankBalances = await queryBankUusd(accAddress);
    let furyBalance = await queryTokenBalance(deploymentDetails.furyContractAddress,accAddress);
    console.log(accAddress + " uFury: " + furyBalance.toString() + ", uusd: " + bankBalances.toString());
}

async function queryBankUusd(address) {
  let response =  await terraClient.bank.balance(address)
  let value;
  try {
    value = Number(response[0]._coins.uusd.amount);
  } catch {
    value = 0;
  } finally {
    return value
  }
}
async function queryTokenBalance(token_address,address) {
  let response = await queryContract(token_address,{
        balance: {address: address}
    });
  return Number(response.balance)
}


async function buyAClub(deploymentDetails, wallet, club_name) {
        console.log(`wallet ${wallet.key.accAddress}`);
       let increaseAllowanceMsg = {
            increase_allowance: {
                spender: deploymentDetails.clubStakingAddress,
                amount: "1000000"
            }
        };
        let incrAllowResp = await executeContract(wallet, deploymentDetails.furyContractAddress, increaseAllowanceMsg);
        console.log(`Increase allowance response hash = ${incrAllowResp['txhash']}`);

        let bacRequest = {
            buy_a_club: {
                buyer: wallet.key.accAddress,
                club_name: club_name,
                auto_stake: true
            }
        };
        let platformFees = await queryContract(deploymentDetails.clubStakingAddress, { query_platform_fees: { msg: Buffer.from(JSON.stringify(bacRequest)).toString('base64') } });
        console.log(`platformFees = ${JSON.stringify(platformFees)}`);
        let bacResponse = await executeContract(wallet, deploymentDetails.clubStakingAddress, bacRequest, { 'uusd': Number(platformFees) });
        console.log("Buy a club transaction hash = " + bacResponse['txhash']);
}

async function stakeOnAClub(deploymentDetails, staker, club_name, amount) {
    let increaseAllowanceMsg = {
        increase_allowance: {
            spender: deploymentDetails.clubStakingAddress,
            amount: amount.toString()
        }
    };
    let incrAllowResp = await executeContract(staker, deploymentDetails.furyContractAddress, increaseAllowanceMsg);
    console.log(`Increase allowance response hash = ${incrAllowResp['txhash']}`);

    let soacRequest = {
        stake_on_a_club: {
            staker: staker.key.accAddress,
            club_name: club_name,
            amount: amount.toString(),
            auto_stake: true
        }
    };
    let platformFees = await queryContract(deploymentDetails.clubStakingAddress, { query_platform_fees: { msg: Buffer.from(JSON.stringify(soacRequest)).toString('base64') } });
    console.log(`platformFees = ${JSON.stringify(platformFees)}`);
    let soacResponse = await executeContract(staker, deploymentDetails.clubStakingAddress, soacRequest, { 'uusd': Number(platformFees) });
    console.log("Stake on a club transaction hash = " + soacResponse['txhash']);
}

async function withdrawTwiceStakeFromAClub(deploymentDetails, staker, club_name, amount, immediate) {
    try {
        let wsfacRequest = {
            stake_withdraw_from_a_club: {
                staker: staker.key.accAddress,
                club_name: club_name,
                amount: amount.toString(),
                immediate_withdrawal: immediate
            }
        };
        let platformFees = await queryContract(deploymentDetails.clubStakingAddress, { query_platform_fees: { msg: Buffer.from(JSON.stringify(wsfacRequest)).toString('base64') } });
        console.log(`platformFees = ${JSON.stringify(platformFees)}`);

        let wsfacResponse = await executeContract(staker, deploymentDetails.clubStakingAddress, wsfacRequest, { 'uusd': Number(platformFees) });
        console.log("Withdraw Stake on a club transaction hash = " + wsfacResponse['txhash']);

        try {
            wsfacResponse = await executeContract(staker, deploymentDetails.clubStakingAddress, wsfacRequest, { 'uusd': Number(platformFees) });
            console.log("Withdraw Stake on a club transaction hash = " + wsfacResponse['txhash']);
        } catch (error) {
            console.log(error.response.config.data)
            console.log(error.response.data.message)
            console.log("Hit an error - maybe expected")
        } finally {
            return
        }
    } catch (error) {
        console.log(error.response.config.data)
        console.log(error.response.data.message)
        console.log("Continuing after error")
    } finally {
        return
    }
}

async function withdrawStakeFromAClub(deploymentDetails, staker, club_name, amount, immediate) {
    try {
        let wsfacRequest = {
            stake_withdraw_from_a_club: {
                staker: staker.key.accAddress,
                club_name: club_name,
                amount: amount.toString(),
                immediate_withdrawal: immediate
            }
        };
        let platformFees = await queryContract(deploymentDetails.clubStakingAddress, { query_platform_fees: { msg: Buffer.from(JSON.stringify(wsfacRequest)).toString('base64') } });
        console.log(`platformFees = ${JSON.stringify(platformFees)}`);

        let wsfacResponse = await executeContract(staker, deploymentDetails.clubStakingAddress, wsfacRequest, { 'uusd': Number(platformFees) });
        console.log("Withdraw Stake on a club transaction hash = " + wsfacResponse['txhash']);
    } catch (error) {
        console.log(error.response.config.data)
        console.log(error.response.data.message)
        console.log("Continuing after error")
    } finally {
        return
    }
}

async function distributeRewards(deploymentDetails, wallet, amount) {
    let iraRequest = {
        increase_reward_amount: {
            reward_from: wallet.key.accAddress
        }
    };
	let msgString = Buffer.from(JSON.stringify(iraRequest)).toString('base64');
            
	let viaMsg = {
		send : {
			contract: deploymentDetails.clubStakingAddress, 
			amount: amount.toString(),
			msg: msgString
		}
	};

    let iraResponse = await executeContract(wallet, deploymentDetails.furyContractAddress, viaMsg);

    let cadrRequest = {
        calculate_and_distribute_rewards: {
        }
    };

	let cadrResponse = await executeContract(wallet, deploymentDetails.clubStakingAddress, cadrRequest);
	console.log("distribute reward transaction hash = " + cadrResponse['txhash']);
}

main();
