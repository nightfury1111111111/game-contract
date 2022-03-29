use cosmwasm_std::{Addr, Attribute, DepsMut, Env, MessageInfo, OverflowError, OverflowOperation, Response, StdError, StdResult, SubMsg, Timestamp, to_binary, Uint128, WasmMsg};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use crate::contract::{instantiate_category_vesting_schedules, VestingInfo};
use crate::error::ContractError;
use crate::msg::InstantiateVestingSchedulesInfo;

use crate::state::{CONFIG, VESTING_DETAILS, VestingDetails};

/// Cliff period unit (seconds in a week)
const CLIFF_PERIOD_UNIT: u64 = 7 * 24 * 60 * 60;

pub fn periodically_calculate_vesting(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let now = env.block.time;
    let config = CONFIG.load(deps.storage)?;
    //Check if the sender (one who is executing this contract) is admin
    if config.admin_wallet != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // let address = env.contract.address;

    // Fetch all tokens that can be vested as per vesting logic
    let vested_details = populate_vesting_details(&deps, now)?;
    // Calculate the total amount to be vested
    let total_vested_amount = calculate_total_distribution(&vested_details);
    //Get the balance available in admin wallet
    let balance_msg = Cw20QueryMsg::Balance {
        address: info.sender.clone().into_string(),
    };
    let balance_response: cw20::BalanceResponse = deps
        .querier
        .query_wasm_smart(config.fury_token_address.clone(), &balance_msg)?;

    let balance = balance_response.balance;
    if balance < total_vested_amount {
        return Err(ContractError::Std(StdError::overflow(OverflowError::new(
            OverflowOperation::Sub,
            balance,
            total_vested_amount,
        ))));
    }
    let mut sub_msgs: Vec<SubMsg> = Vec::new();
    let mut attribs: Vec<Attribute> = Vec::new();
    for elem in vested_details {
        if elem.amount.u128() > 0 {
            let spender_addr = deps.api.addr_validate(&elem.spender_address)?;
            // let category_address = elem.clone().parent_category_address.unwrap_or_default();
            // let owner_addr = deps.api.addr_validate(&category_address)?;
            //Move the tokens from admin wallet to vesting contract
            let transfer_from_msg = Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.clone().into_string(),
                recipient: env.contract.address.clone().into_string(),
                amount: elem.amount,
            };
            let exec_transfer_from_msg = WasmMsg::Execute {
                contract_addr: config.fury_token_address.to_string(),
                msg: to_binary(&transfer_from_msg).unwrap(),
                funds: vec![],
            };
            let send_transfer_from_msg: SubMsg = SubMsg::new(exec_transfer_from_msg);
            sub_msgs.push(send_transfer_from_msg);
            attribs.push(Attribute::new("action", "transfer_from"));
            attribs.push(Attribute::new("from", info.sender.clone().into_string()));
            attribs.push(Attribute::new("for", spender_addr.into_string()));
            attribs.push(Attribute::new("amount", elem.amount));
        }
        //Save the vesting details
        let res = update_vesting_details(
            &mut deps,
            elem.clone().spender_address,
            env.block.time,
            None,
            Some(elem),
        )?;
        for attrib in res.attributes {
            attribs.push(attrib);
        }
    }
    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_attributes(attribs))
}

pub fn calculate_total_distribution(distribution_details: &Vec<VestingInfo>) -> Uint128 {
    let mut total = Uint128::zero();
    for elem in distribution_details {
        total += elem.amount;
    }
    return total;
}

pub fn update_vesting_details(
    deps: &mut DepsMut,
    address: String,
    execution_timestamp: Timestamp,
    transferred: Option<VestingInfo>,
    vestable: Option<VestingInfo>, //mandatory
) -> Result<Response, ContractError> {
    let addr = deps.api.addr_validate(&address)?;

    //replace the optional to required
    match transferred {
        Some(transferred) => {
            VESTING_DETAILS.update(deps.storage, &addr, |vd| -> StdResult<_> {
                //replace the optional to required
                match vd {
                    Some(mut v) => {
                        let new_count = v.total_claimed_tokens_till_now + transferred.amount;
                        if new_count <= v.total_vesting_token_count {
                            v.total_claimed_tokens_till_now = new_count;
                            v.last_vesting_timestamp = Some(execution_timestamp);
                            v.last_claimed_timestamp = Some(execution_timestamp);
                        }
                        v.initial_vesting_consumed = v.initial_vesting_count;
                        Ok(v)
                    }
                    None => Err(StdError::GenericErr {
                        msg: String::from("Vesting Details not found"),
                    }),
                }
            })?;
        }
        None => (),
    }
    match vestable {
        Some(vestable) => {
            VESTING_DETAILS.update(deps.storage, &addr, |vd| -> StdResult<_> {
                match vd {
                    Some(mut v) => {
                        let new_count = v.tokens_available_to_claim + vestable.amount;
                        let mut new_vestable_tokens = new_count;
                        if v.total_claimed_tokens_till_now + new_count > v.total_vesting_token_count
                        {
                            new_vestable_tokens =
                                v.total_vesting_token_count - v.total_claimed_tokens_till_now;
                        }
                        v.tokens_available_to_claim = new_vestable_tokens;
                        if v.last_vesting_timestamp.is_none() {
                            // v.tokens_available_to_claim += v.initial_vesting_count;
                            v.initial_vesting_consumed = v.initial_vesting_count;
                        }
                        v.last_vesting_timestamp = Some(execution_timestamp);
                        Ok(v)
                    }
                    None => Err(StdError::GenericErr {
                        msg: String::from("Vesting Details not found"),
                    }),
                }
            })?;
        }
        None => (),
    }
    Ok(Response::default())
}

pub fn populate_vesting_details(
    deps: &DepsMut,
    now: Timestamp,
) -> Result<Vec<VestingInfo>, ContractError> {
    let vester_addresses: Vec<String> = VESTING_DETAILS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();

    let mut distribution_details: Vec<VestingInfo> = Vec::new();

    for addr in vester_addresses {
        let wallet_address = deps.api.addr_validate(&addr)?;
        let vested_detais = VESTING_DETAILS.may_load(deps.storage, &wallet_address);
        match vested_detais {
            Ok(vested_detais) => {
                let vd = vested_detais.unwrap();
                if !vd.should_transfer {
                    let vesting_info = calculate_tokens_for_this_period(wallet_address, now, vd)?;
                    if vesting_info.amount.u128() > 0 {
                        distribution_details.push(vesting_info);
                    }
                }
            }
            Err(_) => {}
        }
    }

    // // For Nitin
    // let nitin_address = String::from(NITIN_WALLET);
    // let nitin_vesting_info = calculate_vesting_for_now(deps, nitin_address, now)?;
    // if nitin_vesting_info.amount.u128() > 0 {
    //     distribution_details.push(nitin_vesting_info);
    // }

    // // For Ajay
    // let ajay_address = String::from(AJAY_WALLET);
    // let ajay_vesting_info = calculate_vesting_for_now(deps, ajay_address, now)?;
    // if ajay_vesting_info.amount.u128() > 0 {
    //     distribution_details.push(ajay_vesting_info);
    // }

    // // For Sameer
    // let sameer_address = String::from(SAMEER_WALLET);
    // let sameer_vesting_info = calculate_vesting_for_now(deps, sameer_address, now)?;
    // if sameer_vesting_info.amount.u128() > 0 {
    //     distribution_details.push(sameer_vesting_info);
    // }
    Ok(distribution_details)
}

pub fn calculate_tokens_for_this_period(
    wallet_address: Addr,
    now: Timestamp,
    vd: VestingDetails,
) -> Result<VestingInfo, ContractError> {
    // println!("entered calculate_vesting_for_now: ");
    let mut seconds_lapsed = 0;
    let now_seconds: u64 = now.seconds();
    // println!("now_seconds = {}", now_seconds);
    let vesting_start_seconds = vd.vesting_start_timestamp.seconds();
    // println!("vesting_start_seconds = {:?}", vesting_start_seconds);
    // println!("vd.vesting_periodicity = {}", vd.vesting_periodicity);
    if vd.vesting_periodicity > 0 {
        let mut vesting_intervals = 0;
        if now_seconds >= (vesting_start_seconds + (vd.cliff_period * CLIFF_PERIOD_UNIT)) {
            // the now time is greater (ahead) of vesting start + cliff
            seconds_lapsed =
                now_seconds - (vesting_start_seconds + (vd.cliff_period * CLIFF_PERIOD_UNIT));
            println!("seconds_lapsed_1 = {}", seconds_lapsed);
            let total_vesting_intervals = seconds_lapsed / vd.vesting_periodicity;
            // println!("total_vesting_intervals = {}", total_vesting_intervals);
            // println!(
            //     "vd.last_vesting_timestamp.seconds() = {:?}",
            //     vd.last_vesting_timestamp
            // );
            // println!("vesting_start_seconds = {}", vesting_start_seconds);
            // println!("vd.cliff_period = {}", vd.cliff_period);
            let mut seconds_till_last_vesting = 0;
            if vd.last_vesting_timestamp.is_some() {
                seconds_till_last_vesting = vd.last_vesting_timestamp.unwrap().seconds()
                    - (vesting_start_seconds + vd.cliff_period * CLIFF_PERIOD_UNIT);
            }
            // println!("seconds_till_last_vesting = {}", seconds_till_last_vesting);
            let total_vested_intervals = (seconds_till_last_vesting) / vd.vesting_periodicity;
            // println!("total_vested_intervals = {}", total_vested_intervals);

            vesting_intervals = total_vesting_intervals - total_vested_intervals;
            // println!("vesting_intervals = {}", vesting_intervals);
        }
        let tokens_for_this_period_result = vd
            .vesting_count_per_period
            .checked_mul(Uint128::from(vesting_intervals));
        let mut tokens_for_this_period: Uint128;
        match tokens_for_this_period_result {
            Ok(tokens) => {
                // println!("tokens = {}", tokens);
                //Add the initial vested tokens that are not yet claimed
                tokens_for_this_period = tokens;
            }
            Err(e) => {
                // println!("error = {:?}", e);
                let mut message = String::from("error = ");
                message.push_str(e.to_string().as_str());
                tokens_for_this_period = Uint128::zero();
            }
        }
        if vd.total_vesting_token_count
            < (tokens_for_this_period
            + vd.total_claimed_tokens_till_now
            + vd.tokens_available_to_claim)
        {
            tokens_for_this_period = vd.total_vesting_token_count
                - (vd.total_claimed_tokens_till_now + vd.tokens_available_to_claim);
        }
        // println!("tokens_for_this_period = {}", tokens_for_this_period);
        //add the initial seed if cliff period is over
        if now_seconds >= (vesting_start_seconds + (vd.cliff_period * CLIFF_PERIOD_UNIT)) {
            tokens_for_this_period += vd.initial_vesting_count - vd.initial_vesting_consumed;
            // println!(
            //     "tokens_for_this_period after adding= {}",
            //     tokens_for_this_period
            // );
        }
        Ok(VestingInfo {
            spender_address: wallet_address.to_string(),
            parent_category_address: vd.parent_category_address,
            amount: tokens_for_this_period,
        })
    } else {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Vesting periodicity for {:?} address is {:?}",
            wallet_address, vd.vesting_periodicity
        ))));
    }
}


pub fn add_vesting_schedules(
    deps: DepsMut,
    env: Env,
    schedules: InstantiateVestingSchedulesInfo,
) -> Result<Response, ContractError> {
    instantiate_category_vesting_schedules(deps, env, schedules, Option::from(true))
}

pub fn claim_vested_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    //Get vesting information for the sender of this message

    //in this scenerio do we need to have vesting details as optional?

    let vd = VESTING_DETAILS.may_load(deps.storage, &info.sender)?;
    match vd {
        Some(vd) => {
            let owner_addr_str = vd.parent_category_address;
            match owner_addr_str {
                Some(owner_addr_str) => {
                    let _owner_addr = deps.api.addr_validate(&owner_addr_str)?;
                    // deduct allowance before doing anything else have enough allowance
                    //in our case do we have to deduct?
                    // deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;
                    let config = CONFIG.load(deps.storage)?;

                    let transfer_msg = Cw20ExecuteMsg::Transfer {
                        recipient: info.sender.clone().into_string(),
                        amount: amount,
                    };
                    let exec_transfer = WasmMsg::Execute {
                        contract_addr: config.fury_token_address.to_string(),
                        msg: to_binary(&transfer_msg).unwrap(),
                        funds: vec![],
                    };
                    let send_transfer: SubMsg = SubMsg::new(exec_transfer);
                    let res = Response::new().add_submessage(send_transfer);

                    //Update vesting info for sender
                    VESTING_DETAILS.update(deps.storage, &info.sender, |vd| -> StdResult<_> {
                        match vd {
                            Some(mut v) => {
                                v.total_claimed_tokens_till_now =
                                    v.total_claimed_tokens_till_now + amount;
                                v.tokens_available_to_claim = v.tokens_available_to_claim - amount;
                                v.last_claimed_timestamp = Some(env.block.time);
                                Ok(v)
                            }
                            None => Err(StdError::GenericErr {
                                msg: String::from("Vesting Details not found"),
                            }),
                        }
                    })?;
                    return Ok(res);
                }
                None => {
                    return Err(ContractError::Std(StdError::NotFound {
                        kind: String::from("No parent category found"),
                    }));
                }
            }
        }
        None => {
            return Err(ContractError::Std(StdError::NotFound {
                kind: String::from("No vesting details found"),
            }));
        }
    };
}

pub fn populate_transfer_details(
    deps: &DepsMut,
    now: Timestamp,
) -> Result<Vec<VestingInfo>, ContractError> {
    let vester_addresses: Vec<String> = VESTING_DETAILS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();

    let mut distribution_details: Vec<VestingInfo> = Vec::new();

    for addr in vester_addresses {
        let wallet_address = deps.api.addr_validate(&addr)?;
        let vested_detais = VESTING_DETAILS.may_load(deps.storage, &wallet_address);
        match vested_detais {
            Ok(vested_detais) => {
                let vd = vested_detais.unwrap();
                if vd.should_transfer {
                    let vesting_info = calculate_tokens_for_this_period(wallet_address, now, vd)?;
                    if vesting_info.amount.u128() > 0 {
                        distribution_details.push(vesting_info);
                    }
                }
            }
            Err(_) => {}
        }
    }

    // let ga_address = String::from(GAMIFIED_AIRDROP_WALLET);
    // let ga_vesting_info = calculate_vesting_for_now(deps, ga_address, now)?;
    // distribution_details.push(ga_vesting_info);

    //Tokens to be transferred to Private Sale wallet
    // let ps_address = String::from(PRIVATE_SALE_WALLET);
    // let ps_vesting_info = calculate_vesting_for_now(deps, ps_address, now)?;
    // distribution_details.push(ps_vesting_info);
    Ok(distribution_details)
}

pub fn distribute_vested(
    deps: &mut DepsMut,
    sender: String,
    recipient: String,
    amount: Uint128,
) -> Result<SubMsg, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }
    let config = CONFIG.load(deps.storage)?;

    let transfer_from_msg = Cw20ExecuteMsg::TransferFrom {
        owner: sender.clone(),
        recipient: recipient.clone(),
        amount: amount,
    };

    let exec_transfer_from = WasmMsg::Execute {
        contract_addr: config.fury_token_address.to_string(),
        msg: to_binary(&transfer_from_msg).unwrap(),
        funds: vec![],
    };

    let send_transfer_from: SubMsg = SubMsg::new(exec_transfer_from);
    Ok(send_transfer_from)
}

pub fn periodically_transfer_to_categories(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    //capture the current system time
    let now = env.block.time;
    let config = CONFIG.load(deps.storage)?;
    //Check if the sender (one who is executing this contract) is admin
    if config.admin_wallet != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    // Fetch all tokens that can be distributed as per vesting logic
    let distribution_details = populate_transfer_details(&deps, now)?;

    // Calculate the total amount to be vested
    let total_transfer_amount = calculate_total_distribution(&distribution_details);
    //Get the balance available in main wallet
    //is it similar to querying on chain where we can query the contract BALANCE via address
    let balance_query_msg = Cw20QueryMsg::Balance {
        address: info.sender.clone().into_string(),
    };
    let balance_response: BalanceResponse = deps
        .querier
        .query_wasm_smart(config.fury_token_address, &balance_query_msg)?;
    //Check if there is sufficient balance with main wallet
    // return error otherwise
    if balance_response.balance < total_transfer_amount {
        return Err(ContractError::Std(StdError::overflow(OverflowError::new(
            OverflowOperation::Sub,
            balance_response.balance,
            total_transfer_amount,
        ))));
    }

    //this one is understandable, related to with the above one
    let distribute_from = info.sender.clone().into_string();
    let mut sub_msgs: Vec<SubMsg> = Vec::new();
    let mut attribs: Vec<Attribute> = Vec::new();
    for elem in distribution_details {
        // Transfer the funds
        let res = distribute_vested(
            &mut deps,
            distribute_from.clone(),
            elem.spender_address.clone(),
            elem.amount,
        )?;
        sub_msgs.push(res);
        attribs.push(Attribute {
            key: "action".to_string(),
            value: "transfer".to_string(),
        });
        attribs.push(Attribute {
            key: "from".to_string(),
            value: distribute_from.clone(),
        });
        attribs.push(Attribute {
            key: "to".to_string(),
            value: elem.spender_address.clone(),
        });
        attribs.push(Attribute {
            key: "amount".to_string(),
            value: elem.amount.to_string(),
        });
        // Save distribution information
        update_vesting_details(
            &mut deps,
            elem.spender_address.clone(),
            env.block.time,
            Some(elem),
            None,
        )?;
    }
    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_attributes(attribs))
}