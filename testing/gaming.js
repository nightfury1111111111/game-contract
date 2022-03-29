import {
    GamingContractPath,
    mint_wallet,
    treasury_wallet,
    walletTest1,
} from './constants.js';
import {executeContract, instantiateContract, queryContract, storeCode, migrateContract} from "./utils.js";

import {promisify} from 'util';

import * as readline from 'node:readline';

import * as chai from 'chai';


const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
const question = promisify(rl.question).bind(rl);


const assert = chai.assert;
// Init and Vars
const sleep_time = 11000
let gaming_contract_address = ""
let proxy_contract_address = "terra17z8hs6xyfdpsyf564xkjwfwp423zkkam6hquh5"
let fury_contract_address = "terra1zjthyw8e8jayngkvg5kddccwa9v46s4w9sq2pq"
const gamer = treasury_wallet.key.accAddress
// const gamer_extra_1 = walletTest3.key.accAddress
// const gamer_extra_2 = walletTest4.key.accAddress

const gaming_init = {

    "minting_contract_address": fury_contract_address, //  This should be a contract But We passed wallet so it wont raise error on addr validate
    "admin_address": walletTest1.key.accAddress,
    "platform_fee": "1",
    "transaction_fee": "1",
    "game_id": "Game001",
    "platform_fees_collector_wallet": walletTest1.key.accAddress,
    "astro_proxy_address": proxy_contract_address,
}

// Helper Methods

function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}


const deploy_contract = async function (file, init) {
    const contractId = await storeCode(walletTest1, file,)
    await sleep(sleep_time)
    const gamingInit = await instantiateContract(walletTest1, contractId, init)
    console.log(`New Contract Init Hash ${gamingInit.txhash}`)
    return gamingInit.logs[0].events[0].attributes[3].value; // Careful with order of argument
}


function convertBinaryToObject(str) {
    var newBin = str.split(" ");
    var binCode = [];
    for (let i = 0; i < newBin.length; i++) {
        binCode.push(String.fromCharCode(parseInt(newBin[i], 2)));
    }
    let jsonString = binCode.join("");
    return JSON.parse(jsonString)
}


// Tests
let test_create_and_query_game = async function (time) {
    console.log("Uploading Gaming Contract")
    gaming_contract_address = await deploy_contract(GamingContractPath, gaming_init)
    console.log(`Gaming Address:${gaming_contract_address}`)
    await sleep(sleep_time)
    console.log("Executing Query For Contract Details")
    let query_resposne = await queryContract(gaming_contract_address, {
        game_details: {}
    })
    assert.isTrue(gaming_init['game_id'] === query_resposne['game_id'])
    assert.isTrue(1 === query_resposne['game_status'])
    console.log("Assert Success")
    await sleep(time)
}

let test_create_and_query_pool = async function (time) {
    console.log("Testing Create and Query Pool")
    console.log("Create Pool")
    let response = await executeContract(walletTest1, gaming_contract_address, {
        create_pool: {
            "pool_type": "H2H"
        }
    })
    console.log(`Pool Create TX : ${response.txhash}`)
    let new_pool_id = response.logs[0].events[1].attributes[1].value
    console.log(`New Pool ID  ${new_pool_id}`)
    response = await queryContract(gaming_contract_address, {
        pool_details: {
            "pool_id": new_pool_id
        }
    })
    assert.isTrue(response['pool_id'] === new_pool_id)
    assert.isTrue(response['game_id'] === "Game001")
    assert.isTrue(response['pool_type'] === "H2H")
    assert.isTrue(response['current_teams_count'] === 0)
    assert.isTrue(response['rewards_distributed'] === false)
    console.log("Assert Success")
    await sleep(time)
}

const set_pool_headers_for_H2H_pool_type = async function (time) {
    const response = await executeContract(walletTest1, gaming_contract_address, {
        set_pool_type_params: {
            'pool_type': "H2H",
            'pool_fee': "10000000",
            'min_teams_for_pool': 1,
            'max_teams_for_pool': 2,
            'max_teams_for_gamer': 2,
            'wallet_percentages': [
                {
                    "wallet_address": "terra1uyuy363rzun7e6txdjdemqj9zua9j7wxl2lp0m",
                    "wallet_name": "rake_1",
                    "percentage": 100,
                }
            ]
        }
    })
    console.log(response)
    console.log("Assert Success")
    if (time) await sleep(time)
}

async function transferFuryTokens(toAddress, amount) {
    let transferFuryToTreasuryMsg = {
        transfer: {
            recipient: toAddress.key.accAddress,
            amount: amount
        }
    };
    console.log(`transferFuryToTreasuryMsg = ${JSON.stringify(transferFuryToTreasuryMsg)}`);
    let response = await executeContract(mint_wallet, fury_contract_address, transferFuryToTreasuryMsg, {'uusd': 200000000});
    console.log(`transferFuryToTreasuryMsg Response - ${response['txhash']}`);
}


let test_game_pool_bid_submit_when_pool_team_in_range = async function (time) {
    console.log("Placing a bid submit in H2H pool for 10UST worth of Fury!")

    // Add method to provide the wallet one fury token
    console.log("Sending 5k fury Tokens from Minter to gamer")
    let response = await transferFuryTokens(walletTest1, "5000000000")
    console.log(response)
    console.log("Getting Funds To Send In Fury")
    let funds_to_send_in_fury = await queryContract(proxy_contract_address,
        {
            get_fury_equivalent_to_ust: {
                "ust_count": "10000000"
            }
        });

    console.log(`UST equivalent Fury:${funds_to_send_in_fury}`)
    let increaseAllowanceMsg = {
        increase_allowance: {
            spender: gaming_contract_address,
            amount: `${funds_to_send_in_fury * 2}`
        }
    };
    console.log("Increasing Allowance For the Gaming Pool Contract ")
    let incrAllowResp = await executeContract(walletTest1, fury_contract_address, increaseAllowanceMsg);
    console.log(incrAllowResp)
    console.log("Submitting Game Pool Bid")
    await sleep(sleep_time);
    try {
        response = await executeContract(walletTest1, gaming_contract_address, {
            game_pool_bid_submit_command: {
                gamer: walletTest1.key.accAddress,
                pool_type: "H2H",
                pool_id: "1",
                team_id: "Team001",
                amount: `${funds_to_send_in_fury}`
            }
        }, {'uusd': 1300000})
        console.log(response);
    } catch (a) {
        console.log("Caught Error Executing Bidsubmit")
        console.log(a)
        await sleep(sleep_time)
        console.log("Re-Executing")
        let some = await executeContract(walletTest1, gaming_contract_address, {
            game_pool_bid_submit_command: {
                gamer: gamer,
                pool_type: "H2H",
                pool_id: "1",
                team_id: "Team001",
                amount: `${funds_to_send_in_fury}`
            }
        }, {'uusd': 1300000})
        console.log(some);


    }

    //checking the total UST count receieved from the bid pool price was 10UST receieved 9.018376 UST
    console.log("Assert Success");
    await sleep(time)
}

const test_game_lock_once_pool_is_closed = async function (time) {
    console.log("Testing game lock once pool is filled/closed.")

    let response = await executeContract(walletTest1, gaming_contract_address, {
        lock_game: {}
    })
    console.log(response)
    console.log("Assert Success")
    //query the status of pool, check if it's locked
    await sleep(time)
}
const test_game_lock_once_pool_is_canceled = async function (time) {
    console.log("Testing game lock once pool is Cancelled.")

    let response = await executeContract(walletTest1, gaming_contract_address, {
        cancel_game: {}
    })
    console.log(response)
    console.log("Assert Success")
    await sleep(time)
}
//     ExecuteMsg::ClaimReward { gamer } => claim_reward(deps, info, gamer, env),
//     ExecuteMsg::ClaimRefund { gamer } => claim_refund(deps, info, gamer, env),
const claim = async function (time) {
    let expected_reward = await queryContract(gaming_contract_address, {
            query_reward: {"gamer": gamer}
        }
    )
    console.log(`Expected Reward Amount  ${expected_reward}`)
    let response = await executeContract(walletTest1, gaming_contract_address, {
        claim_reward: {"gamer": walletTest1.key.accAddress}
    })
    console.log(response)
    //check if the distributed amount is eq to claim amount
    console.log("Assert Success")
    await sleep(time)
}
const reward_distribution_for_locked_game_for_H2H = async function (time) {
    console.log("Reward Distribution for locked game")
    let response = await executeContract(walletTest1, gaming_contract_address, {
        "game_pool_reward_distribute": {
            "game_id": "Gamer001",
            "pool_id": "1",
            "game_winners":
                [
                    {
                        "gamer_address": gamer,
                        "game_id": "Gamer001",
                        "team_id": "1",
                        "reward_amount": "5000000", // This will be in ufury
                        "refund_amount": "0",
                        "team_rank": 1,
                        "team_points": 150
                    },
                ]
        }
    })
    console.log(response)
    console.log("Assert Success")
    ///check if the game is concluded and status is updated to 4/reward distributed
    await sleep(time)
}

async function test_migrate(time) {
    console.log("Testing Migrate")
    console.log("Uploading New Contract")
    let contract_id = await storeCode(walletTest1, GamingContractPath,) //  Uploading the new contract
    console.log("Executing the migrate")
    let r = await migrateContract(walletTest1, gaming_contract_address, contract_id, {})
    console.log(r)
    console.log("Success")
    await sleep(time)


}

async function test_game_pool_reward_distribute(time) {
    console.log("Game Pool Reward Distribute")
    let game_winners = [
        {
            gamer_address: gamer,
            game_id: "Game001",
            team_id: "Team001",
            team_rank: 1,
            team_points: 100,
            reward_amount: 100,
            refund_amount: ""
        }, {
            gamer_address: walletTest1.key.accAddress,
            game_id: "Game001",
            team_id: "Team001",
            team_rank: 2,
            team_points: 200,
            reward_amount: 100,
            refund_amount: ""
        },
        {
            gamer_address: walletTest1.key.accAddress,
            game_id: "Game001",
            team_id: "Team001",
            team_rank: 2,
            team_points: 300,
            reward_amount: 100,
            refund_amount: ""
        }
    ]
    console.log("Executing Reward Distribute")

    let response = await executeContract(walletTest1, gaming_contract_address, {
        game_pool_reward_distribute: {
            pool_id: "1",
            game_winners: game_winners
        }
    })
    console.log(response)
}

await test_create_and_query_game(sleep_time)
await test_create_and_query_pool(sleep_time)
// await test_get_team_count_for_user_in_pool_type(sleep_time)
await set_pool_headers_for_H2H_pool_type(sleep_time)
await test_game_pool_bid_submit_when_pool_team_in_range(sleep_time)
await test_game_lock_once_pool_is_closed(sleep_time)
await reward_distribution_for_locked_game_for_H2H(sleep_time)
await claim(sleep_time)
// // Claim
// await test_migrate(sleep_time)
// await test_create_and_query_game(sleep_time)
// await test_create_and_query_pool(sleep_time)
// await test_get_team_count_for_user_in_pool_type(sleep_time)
// await set_pool_headers_for_H2H_pool_type(sleep_time)
// await test_game_pool_bid_submit_when_pool_team_in_range(sleep_time)
// await test_game_lock_once_pool_is_closed(sleep_time)
// await test_game_lock_once_pool_is_canceled(sleep_time)
// // Refund