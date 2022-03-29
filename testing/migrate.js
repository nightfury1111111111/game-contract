import {
    VnDContractPath,
    minting_wallet,
} from './constants.js';
import {
    storeCode,
    migrateContract
} from "./utils.js";

let current_address = "terra1rws5tqe6fxl3hgmvywq76c6200rpqsy5tqvyuy"

export function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}

let new_code_id = await storeCode(minting_wallet, VnDContractPath);
await sleep(15000)
let response = await migrateContract(minting_wallet, current_address, new_code_id, {})
console.log(response)