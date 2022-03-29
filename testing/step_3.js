import {executeContract, queryContract} from "./utils.js";
import {deployer, walletTest1} from "./constants.js";


let gaming_contract_address = "terra1nkj868ysnsfmpld3edjs73dzn0kpxceees5krf"
console.log("Reward Distribution for locked game")
let response = await executeContract(walletTest1, gaming_contract_address, {
    "game_pool_reward_distribute": {
        "game_id": "Gamer001",
        "pool_id": "1",
        "game_winners":
            [
                {
                    "gamer_address": deployer.key.accAddress,
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