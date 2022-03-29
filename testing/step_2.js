import {executeContract, queryContract} from "./utils.js";
import {walletTest1} from "./constants.js";


let gaming_contract_address = "terra1nkj868ysnsfmpld3edjs73dzn0kpxceees5krf"
console.log("Testing game lock once pool is filled/closed.")

let response = await executeContract(walletTest1, gaming_contract_address, {
    lock_game: {}
})
console.log(response)
console.log("Assert Success")