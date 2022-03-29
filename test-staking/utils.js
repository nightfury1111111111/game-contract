import fs from "fs";
import chalk from "chalk";
import {isTxError} from "@terra-money/terra.js/dist/client/lcd/api/TxAPI.js";
import {
  MsgExecuteContract,
  MsgInstantiateContract,
  MsgStoreCode
} from "@terra-money/terra.js/dist/core/wasm/msgs/index.js";
import {terraClient,} from "./constants.js";

import {
  readFileSync,
  writeFileSync,
} from 'fs';
import path from 'path';

export const ARTIFACTS_PATH = 'artifacts'

var gas_used = 0;

export function getGasUsed() {
  return gas_used;
}

export function writeArtifact(data, name = 'artifact') {
  writeFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), JSON.stringify(data, null, 2))
}


export function readArtifact(name = 'artifact') {
    try {
        const data = readFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), 'utf8')
        return JSON.parse(data)
    } catch (e) {
        return {}
    }
}


export function readDistantArtifact(distantPath, name = 'artifact') {
    try {
        console.log(`trying path : ${path.join(distantPath, ARTIFACTS_PATH, `${name}.json`)}`)
        const data = readFileSync(path.join(distantPath, ARTIFACTS_PATH, `${name}.json`), 'utf8')
        return JSON.parse(data)
    } catch (e) {
        return {}
    }
}

/**
 * @notice Upload contract code to LocalTerra. Return code ID.
 */
export async function storeCode(deployerWallet, filepath) {
  const code = fs.readFileSync(filepath).toString("base64");
  const result = await sendTransaction(deployerWallet, [
    new MsgStoreCode(deployerWallet.key.accAddress, code),
  ]);
  return parseInt(result.logs[0].eventsByType.store_code.code_id[0]);
}

/**
 * @notice Execute a contract
 */
export async function executeContract(senderWallet, contractAddress, msg, coins, verbose = false) {
  let msg_list = []

  if (Array.isArray(msg)) {
    msg.forEach((msg) => {
      msg_list.push(new MsgExecuteContract(senderWallet.key.accAddress, contractAddress, msg, coins))
    })

  } else {
    msg_list = [
      new MsgExecuteContract(senderWallet.key.accAddress, contractAddress, msg, coins),
    ]
  }
  return await sendTransaction(senderWallet, msg_list, verbose);
}

/**
 * @notice Send a transaction. Return result if successful, throw error if failed.
 */
export async function sendTransaction(senderWallet, msgs, verbose = false) {
  // todo estimate fee
  // console.log(msgs)
  // // https://fcd.terra.dev/v1/txs/gas_prices
  // const fee = terraClient.tx.estimateFee(msgs, {
  //   gasPrices: { ['uluna']: 0.013199 },
  // });
  // console.log(fee)
  // fees = gas * gas_prices
  const tx = await senderWallet.createAndSignTx({
    msgs,
    gasPrices: {uusd: 0.15},
    gasAdjustment: 1.75
  });

  const result = await terraClient.tx.broadcast(tx);

  // Print the log info
  if (verbose) {
    console.log(chalk.magenta("\nTxHash:"), result.txhash);
    try {
      console.log(
        chalk.magenta("Raw log:"),
        JSON.stringify(JSON.parse(result.raw_log), null, 2)
      );
    } catch {
      console.log(chalk.magenta("Failed to parse log! Raw log:"), result.raw_log);
    }
  }

  if (isTxError(result)) {
    throw new Error(
      chalk.red("Transaction failed!") +
      `\n${chalk.yellow("code")}: ${result.code}` +
      `\n${chalk.yellow("codespace")}: ${result.codespace}` +
      `\n${chalk.yellow("raw_log")}: ${result.raw_log}`
    );
  }
  gas_used += Number(result['gas_used']);
  console.log("Gas = " + result['gas_used']);
  return result;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
export async function instantiateContract(deployer, codeId, instantiateMsg) {
  return await sendTransaction(deployer, [
    new MsgInstantiateContract(
      deployer.key.accAddress,
      deployer.key.accAddress,
      codeId,
      instantiateMsg
    ),
  ]);
}

export async function queryContract(contractAddress, query) {
  return await terraClient.wasm.contractQuery(contractAddress, query)
}

export async function queryContractInfo(contractAddress) {
  const d = await terraClient.wasm.contractInfo(contractAddress);
  return d
}

export async function get_server_epoch_seconds() {
  const blockInfo = await terraClient.tendermint.blockInfo()
  const time = blockInfo['block']['header']['time']
  let dateObject = new Date(time);
  let epoch = dateObject.getTime();
  return Math.round(epoch / 1000)
}

export async function queryBankUusd(address) {
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


export async function queryTokenBalance(token_address,address) {
  let response = await queryContract(token_address,{
        balance: {address: address}
    });
  return Number(response.balance)
}

