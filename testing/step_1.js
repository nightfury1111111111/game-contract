import {executeContract, queryContract} from "./utils.js";
import {walletTest1} from "./constants.js";


let gaming_contract_address = "terra1nkj868ysnsfmpld3edjs73dzn0kpxceees5krf"
let funds_to_send_in_fury = 104640284
console.log("Executing")
let some = await executeContract(walletTest1, gaming_contract_address, {
    game_pool_bid_submit_command: {
        gamer: walletTest1.key.accAddress,
        pool_type: "H2H",
        pool_id: "1",
        team_id: "Team001",
        amount: `${funds_to_send_in_fury}`
    }
}, {'uusd': 1300000})
console.log(some);