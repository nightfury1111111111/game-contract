import {
    GamingContractPath,
    walletTest1,
} from './constants.js';
import {
    storeCode,
} from "./utils.js";

import {increasePOLRewardAllowance} from './increasePOLRewardAllowance.js';


import {astroport_setup} from "./astroport.js";
import {vesting_and_distribution} from './index.js';

const sleep_time = 31000;

function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}


const upload_contract = async function (file) {
    const contractId = await storeCode(walletTest1, file,)
    console.log(`New Contract Id For Gaming ${contractId}`)
}
console.log("Initiating Total Deployment");
await vesting_and_distribution().then(() => {
     deployment().then(r => {
         console.log("Deployment Complete")
     })
})
//check bonded wallet balance
//check allowances spender(proxy) and owner (bonded wallet add) through query 
//to get the exact amount delta of balance amt & allowed amt 

export async function deployment() {
    await sleep(sleep_time)
    await increasePOLRewardAllowance(sleep_time);
    await sleep(sleep_time)
    await astroport_setup()
    await sleep(sleep_time)
    await upload_contract(GamingContractPath)
}

