import {
    queryContract,
    executeContract,
} from "./utils.js";


export const increasePOLRewardAllowance = async (deploymentDetails) => {

        let response = await queryContract(deploymentDetails.furyContractAddress, {
            balance: {address: deploymentDetails.native_investment_reward_wallet}
        });

        let respBalance = Number(`${response.balance}`);

        response = await queryContract(deploymentDetails.furyContractAddress, {
            allowance: {owner: native_investment_reward_wallet.key.address,
                        spender:deploymentDetails.proxyContractAddress}
        });

        let respAllowance = Number(`${response.allowance}`);

        if (respBalance > respAllowance) {
            let increase_amount = respBalance - respAllowance;
            let execMsg = {increase_allowance: { spender : deploymentDetails.proxyContractAddress, amount: increase_amount.toString()}};
            let execResponse = await executeContract (native_investment_reward_wallet, deploymentDetails.furyContractAddress, execMsg);
            console_log("POL increase allowance by ${increase_amount} for proxy in Native reward wallet ", execResponse['txhash']);
        }

        response = await queryContract(deploymentDetails.furyContractAddress, {
            balance: {address: deploymentDetails.pair_investment_reward_wallet}
        });

        respBalance = Number($`{response.balance}`);

        response = await queryContract(deploymentDetails.furyContractAddress, {
            allowance: {owner: pair_investment_reward_wallet.key.address,
                        spender:deploymentDetails.proxyContractAddress}
        });
        respAllowance = Number(`${response.allowance}`);

        if (respBalance > respAllowance) {
            let increase_amount = respBalance - respAllowance;
            let execMsg = {increase_allowance: { spender : deploymentDetails.proxyContractAddress, amount: increase_amount.toString()}};
            let execResponse = await executeContract (pair_investment_reward_wallet, deploymentDetails.furyContractAddress, execMsg);
            console_log("POL increase allowance by ${increase_amount} for proxy in Pair reward wallet ", execResponse['txhash']);
        }
    }
