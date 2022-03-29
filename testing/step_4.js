import {executeContract, queryContract} from "./utils.js";
import {deployer, walletTest1} from "./constants.js";


let gaming_contract_address = "terra1nkj868ysnsfmpld3edjs73dzn0kpxceees5krf"
console.log("Executing")
let expected_reward = await queryContract(gaming_contract_address, {
        query_reward: {"gamer": deployer.key.accAddress}
    }
)
console.log(`Expected Reward Amount  ${expected_reward}`)
if (expected_reward !== 0) {
    let response = await executeContract(walletTest1, gaming_contract_address, {
        claim_reward: {"gamer": walletTest1.key.accAddress}
    })
    console.log(response)
//check if the distributed amount is eq to claim amount
    console.log("Assert Success")

}
