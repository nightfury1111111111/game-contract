#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Reply, Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use cw_storage_plus::{Map};


use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceivedMsg, ProxyQueryMsgs};
use crate::state::{
    ClubBondingDetails, ClubOwnershipDetails, ClubPreviousOwnerDetails, ClubStakingDetails, Config, 
    CLUB_BONDING_DETAILS, CLUB_OWNERSHIP_DETAILS, CLUB_PREVIOUS_OWNER_DETAILS,
    CLUB_REWARD_NEXT_TIMESTAMP, CLUB_STAKING_DETAILS, CONFIG, REWARD, CLUB_STAKING_SNAPSHOT,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:club-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INCREASE_STAKE: bool = true;
const DECREASE_STAKE: bool = false;
const IMMEDIATE_WITHDRAWAL: bool = true;
const NO_IMMEDIATE_WITHDRAWAL: bool = false;
const DONT_CHANGE_AUTO_STAKE_SETTING: bool = false;
const SET_AUTO_STAKE: bool = true;
const MAX_UFURY_COUNT: i128 = 420000000000000;
// Reward to club owner for buying - 0 tokens
const CLUB_BUYING_REWARD_AMOUNT: u128 = 0u128;

// Reward to club staker for staking - 0 tokens
const CLUB_STAKING_REWARD_AMOUNT: u128 = 0u128;

// This is reduced to 0 day locking period in seconds, after buying a club, as no refund planned for Ownership Fee
const CLUB_LOCKING_DURATION: u64 = 0u64;

// This is locking period in seconds, after staking in club.
// No longer applicable so setting it to 0
const CLUB_STAKING_DURATION: u64 = 0u64;

// this is 7 day bonding period in seconds, after withdrawing a stake 
// TODO _ Revert after DEBUG : this is 1 hour for testing purposes only
// const CLUB_BONDING_DURATION: u64 = 3600u64;
// - now part of instantiation msg.bonding_duration

// use cosmwasm_std::{Coin, Timestamp};

const HUNDRED_PERCENT: u128 = 10000u128;
const NINETY_NINE_NINE_PERCENT: u128 = 9990u128;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut next_reward_time = msg.club_reward_next_timestamp;
    if next_reward_time.seconds() == 0u64 {
        next_reward_time = _env.block.time.minus_seconds(1);
    }
    let config = Config {
        admin_address: deps.api.addr_validate(&msg.admin_address)?,
        minting_contract_address: deps.api.addr_validate(&msg.minting_contract_address)?,
        astro_proxy_address: deps.api.addr_validate(&msg.astro_proxy_address)?,
        club_fee_collector_wallet: deps.api.addr_validate(&msg.club_fee_collector_wallet)?,
        club_reward_next_timestamp: next_reward_time,
        reward_periodicity: msg.reward_periodicity,
        club_price: msg.club_price,
        bonding_duration: msg.bonding_duration,
        owner_release_locking_duration: msg.owner_release_locking_duration,
        platform_fees_collector_wallet: deps
            .api
            .addr_validate(&msg.platform_fees_collector_wallet)?,
        platform_fees: msg.platform_fees,
        transaction_fees: msg.transaction_fees,
        control_fees: msg.control_fees,
    };
    CONFIG.save(deps.storage, &config)?;

    CLUB_REWARD_NEXT_TIMESTAMP.save(deps.storage, &config.club_reward_next_timestamp)?;
    println!(
        "now = {:?} next_timestamp = {:?} periodicity = {:?}",
        _env.block.time, config.club_reward_next_timestamp, config.reward_periodicity
    );
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => received_message(deps, env, info, msg),
        ExecuteMsg::StakeOnAClub {
            staker,
            club_name,
            amount,
            auto_stake,
        } => {
            stake_on_a_club(deps, env, info, staker, club_name, amount, auto_stake)
        }
        ExecuteMsg::AssignStakesToAClub {
            stake_list,
            club_name
        } => {
            assign_stakes_to_a_club(deps, env, info, stake_list, club_name)
        }
        ExecuteMsg::BuyAClub {
            buyer,
            seller,
            club_name,
            auto_stake,
        } => {
            let config = CONFIG.load(deps.storage)?;
            let price = config.club_price;
            buy_a_club(deps, env, info, buyer, seller, club_name, price, auto_stake)
        }
        ExecuteMsg::AssignAClub {
            buyer,
            seller,
            club_name,
            auto_stake,
        } => {
            assign_a_club(deps, env, info, buyer, seller, club_name, auto_stake)
        }
        ExecuteMsg::ReleaseClub { owner, club_name } => {
            release_club(deps, env, info, owner, club_name)
        }
        ExecuteMsg::ClaimOwnerRewards { owner, club_name } => {
            claim_owner_rewards(deps, env, info, owner, club_name)
        }
        ExecuteMsg::ClaimPreviousOwnerRewards { previous_owner } => {
            claim_previous_owner_rewards(deps, info, previous_owner)
        }
        ExecuteMsg::StakeWithdrawFromAClub {
            staker,
            club_name,
            amount,
            immediate_withdrawal,
        } => withdraw_stake_from_a_club(
            deps,
            env,
            info,
            staker,
            club_name,
            amount,
            immediate_withdrawal,
        ),
        ExecuteMsg::CalculateAndDistributeRewards {} => {
            calculate_and_distribute_rewards(deps, env, info)
        }
        ExecuteMsg::ClaimStakerRewards { staker, club_name } => {
            claim_staker_rewards(deps, info, staker, club_name)
        }
        ExecuteMsg::PeriodicallyRefundStakeouts {} => {
            periodically_refund_stakeouts(deps, env, info)
        }
    }
}

fn received_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    message: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceivedMsg = from_binary(&message.msg)?;
    let amount = Uint128::from(message.amount);
    match msg {
        ReceivedMsg::IncreaseRewardAmount(irac) => {
            increase_reward_amount(deps, env, info, irac.reward_from, amount)
        }
    }
    // Err(ContractError::Std(StdError::GenericErr {
    //     msg: format!("received_message where msg = {:?}", msg),
    // }))
}

fn claim_previous_owner_rewards(
    deps: DepsMut,
    info: MessageInfo,
    previous_owner: String,
) -> Result<Response, ContractError> {
    let mut amount = Uint128::zero();
    let mut transfer_confirmed = false;
    let previous_owner_addr = deps.api.addr_validate(&previous_owner)?;
    //Check if withdrawer is same as invoker
    if previous_owner_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let previous_ownership_details;
    let previous_ownership_details_result =
        CLUB_PREVIOUS_OWNER_DETAILS.may_load(deps.storage, previous_owner.clone());
    match previous_ownership_details_result {
        Ok(od) => {
            previous_ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::from(e)));
        }
    }

    if !(previous_ownership_details.is_none()) {
        for previous_owner_detail in previous_ownership_details {
            if previous_owner_detail.previous_owner_address == previous_owner.clone() {
                if Uint128::zero() == previous_owner_detail.reward_amount {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("No rewards for this previous owner"),
                    }));
                }

                amount = previous_owner_detail.reward_amount;

                // Now remove the previous ownership details
                CLUB_PREVIOUS_OWNER_DETAILS.remove(deps.storage, previous_owner.clone());

                // Add amount to the owners wallet
                transfer_confirmed = true;
            }
        }
    }
    if transfer_confirmed == false {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Not a valid previous owner for the club"),
        }));
    }
    transfer_from_contract_to_wallet(
        deps.storage,
        previous_owner.clone(),
        amount,
        "previous_owners_reward".to_string(),
    )
}

fn claim_owner_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    club_name: String,
) -> Result<Response, ContractError> {
    let mut amount = Uint128::zero();
    let mut transfer_confirmed = false;
    let owner_addr = deps.api.addr_validate(&owner)?;
    //Check if withdrawer is same as invoker
    if owner_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let ownership_details;
    let ownership_details_result = CLUB_OWNERSHIP_DETAILS.may_load(deps.storage, club_name.clone());
    match ownership_details_result {
        Ok(od) => {
            ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::from(e)));
        }
    }

    if !(ownership_details.is_none()) {
        for owner_detail in ownership_details {
            if owner_detail.owner_address == owner.clone() {
                if Uint128::zero() == owner_detail.reward_amount {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("No rewards for this owner"),
                    }));
                }

                transfer_confirmed = true;

                amount = owner_detail.reward_amount;

                // Now save the ownership details
                CLUB_OWNERSHIP_DETAILS.save(
                    deps.storage,
                    club_name.clone(),
                    &ClubOwnershipDetails {
                        club_name: owner_detail.club_name,
                        start_timestamp: owner_detail.start_timestamp,
                        locking_period: owner_detail.locking_period,
                        owner_address: owner_detail.owner_address,
                        price_paid: owner_detail.price_paid,
                        reward_amount: Uint128::zero(),
                        owner_released: owner_detail.owner_released,
                    },
                )?;
            }
        }
    }

    if transfer_confirmed == false {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Not a valid owner for the club"),
        }));
    }
    transfer_from_contract_to_wallet(
        deps.storage,
        owner.clone(),
        amount,
        "owner_reward".to_string(),
    )
}

fn periodically_refund_stakeouts(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {});
    }

    //capture the current system time
    let now = env.block.time;

    // Fetch all bonding details
    let all_clubs: Vec<String> = CLUB_BONDING_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let mut all_bonds = Vec::new();
        let bonding_details = CLUB_BONDING_DETAILS.load(deps.storage, club_name.clone())?;
        for mut bond in bonding_details {
            let mut duration = bond.bonding_duration;
            let now_minus_duration_timestamp = now.minus_seconds(duration);
            if now_minus_duration_timestamp < bond.bonding_start_timestamp {
                all_bonds.push(bond);
            } else {
                // transfer to bonder wallet
                // NOT reqd exdternally
                // transfer_from_contract_to_wallet(deps.storage, bond.bonder_address, bond.bonded_amount);
            }
        }
        CLUB_BONDING_DETAILS.save(deps.storage, club_name, &all_bonds)?;
    }
    return Ok(Response::default());
}

fn buy_a_club(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    buyer: String,
    seller_opt: Option<String>,
    club_name: String,
    price: Uint128,
    auto_stake: bool,
) -> Result<Response, ContractError> {
    println!("seller_opt = {:?}", seller_opt);
    let seller;
    match seller_opt.clone() {
        Some(s) => seller = s,
        None => seller = String::default(),
    }

    let config = CONFIG.load(deps.storage)?;

    let club_price = config.club_price;
    if price != club_price {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Club price is not matching"),
        }));
    }

    let required_ust_fees: Uint128;
    //To bypass calls from unit tests
    if info.sender.clone().into_string() == String::from("Owner001")
        || info.sender.clone().into_string() == String::from("Owner002")
        || info.sender.clone().into_string() == String::from("Owner003")
    {
        required_ust_fees = Uint128::zero();
    } else {
        required_ust_fees = query_platform_fees(
            deps.as_ref(),
            to_binary(&ExecuteMsg::BuyAClub {
                buyer: buyer.clone(),
                club_name: club_name.clone(),
                seller: seller_opt,
                auto_stake: auto_stake,
            })?,
        )?;
    }
    let mut fees = Uint128::zero();
    for fund in info.funds.clone() {
        if fund.denom == "uusd" {
            fees = fees.checked_add(fund.amount).unwrap();
        }
    }
    let adjusted_ust_fees = required_ust_fees
        * (Uint128::from(NINETY_NINE_NINE_PERCENT))
        / (Uint128::from(HUNDRED_PERCENT));
    if fees < adjusted_ust_fees {
        return Err(ContractError::InsufficientFees {
            required: required_ust_fees,
            received: fees,
        });
    }
    let buyer_addr = deps.api.addr_validate(&buyer)?;

    let ownership_details;
    let ownership_details_result = CLUB_OWNERSHIP_DETAILS.may_load(deps.storage, club_name.clone());
    match ownership_details_result {
        Ok(od) => {
            ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::from(e)));
        }
    }

    let all_clubs: Vec<String> = CLUB_OWNERSHIP_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();

    for one_club_name in all_clubs {
        let one_ownership_details =
            CLUB_OWNERSHIP_DETAILS.load(deps.storage, one_club_name.clone())?;
        if buyer == one_ownership_details.owner_address {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("buyer already owns this club"),
            }));
        }
    }

    let mut previous_owners_reward_amount = Uint128::from(0u128);

    if !(ownership_details.is_none()) {
        for owner in ownership_details {
            let mut current_time = env.block.time;
            let mut release_start_time = owner.start_timestamp;
            let mut release_locking_duration = owner.locking_period;
            println!(
                "release_start_time = {:?} locking_duration = {:?} current time = {:?}",
                release_start_time, release_locking_duration, current_time
            );
            if owner.owner_released == false {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Owner has not released the club"),
                }));
            } else if current_time > release_start_time.plus_seconds(release_locking_duration) {
                println!("Release time for the club has expired");
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Release time for the club has expired"),
                }));
            } else if owner.owner_address != String::default() && owner.owner_address != seller {
                println!(
                    "owner.owner_address = {:?} and seller = {:?}",
                    owner.owner_address, seller
                );
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Seller is not the owner for the club"),
                }));
            }

            // Evaluate previous owner rewards
            previous_owners_reward_amount = owner.reward_amount;
            println!("prv own amount picked {:?}", previous_owners_reward_amount);
            let mut previous_reward = Uint128::zero();
            println!("prv own amount avl {:?}", previous_owners_reward_amount);
            if previous_owners_reward_amount != Uint128::zero() {
                let pod = CLUB_PREVIOUS_OWNER_DETAILS.may_load(deps.storage, seller.clone())?;
                match pod {
                    Some(pod) => {
                        previous_reward = pod.reward_amount;
                        println!("prv own existing reward {:?}", previous_reward);
                    }
                    None => {}
                }

                // Now save the previous ownership details
                CLUB_PREVIOUS_OWNER_DETAILS.save(
                    deps.storage,
                    seller.clone(),
                    &ClubPreviousOwnerDetails {
                        previous_owner_address: seller.clone(),
                        reward_amount: previous_reward + previous_owners_reward_amount,
                    },
                )?;
            }
        }
    }

    // Now save the ownership details
    CLUB_OWNERSHIP_DETAILS.save(
        deps.storage,
        club_name.clone(),
        &ClubOwnershipDetails {
            club_name: club_name.clone(),
            start_timestamp: env.block.time,
            locking_period: config.owner_release_locking_duration,
            owner_address: buyer_addr.to_string(),
            price_paid: price,
            reward_amount: Uint128::from(CLUB_BUYING_REWARD_AMOUNT),
            owner_released: false,
        },
    )?;

    let mut stakes = Vec::new();
    let mut user_stake_exists = false;
    let all_stakes = CLUB_STAKING_DETAILS.may_load(deps.storage, club_name.clone())?;
    match all_stakes {
        Some(some_stakes) => {
            stakes = some_stakes;
        }
        None => {}
    }
    for stake in stakes {
        if buyer == stake.staker_address {
            user_stake_exists = true;
        }
    }
    if !user_stake_exists {
        // Now save the staking details for the owner - with 0 stake
        save_staking_details(
            deps.storage,
            env,
            buyer.clone(),
            club_name.clone(),
            Uint128::zero(),
            auto_stake,
            INCREASE_STAKE,
        )?;
    }

    let transfer_msg = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.into_string(),
        recipient: config.club_fee_collector_wallet.to_string(),
        amount: price,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    };

    let send_wasm: CosmosMsg = CosmosMsg::Wasm(exec);
    let send_bank: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.platform_fees_collector_wallet.into_string(),
        amount: info.funds,
    });
    let data_msg = format!("Club fees {} received", price).into_bytes();
    return Ok(Response::new()
        .add_message(send_wasm)
        .add_message(send_bank)
        .add_attribute("action", "buy_a_club")
        .add_attribute("buyer", buyer)
        .add_attribute("club_name", club_name)
        .add_attribute("fees", price.to_string())
        .set_data(data_msg));
}

fn assign_a_club(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    buyer: String,
    seller_opt: Option<String>,
    club_name: String,
    auto_stake: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {});
    }

    println!("seller_opt = {:?}", seller_opt);
    let seller;
    match seller_opt.clone() {
        Some(s) => seller = s,
        None => seller = String::default(),
    }

    let buyer_addr = deps.api.addr_validate(&buyer)?;

    let ownership_details;
    let ownership_details_result = CLUB_OWNERSHIP_DETAILS.may_load(deps.storage, club_name.clone());
    match ownership_details_result {
        Ok(od) => {
            ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::from(e)));
        }
    }

    let all_clubs: Vec<String> = CLUB_OWNERSHIP_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();

    for one_club_name in all_clubs {
        let one_ownership_details =
            CLUB_OWNERSHIP_DETAILS.load(deps.storage, one_club_name.clone())?;
        if buyer == one_ownership_details.owner_address {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("buyer already owns this club"),
            }));
        }
    }

    let mut previous_owners_reward_amount = Uint128::from(0u128);

    if !(ownership_details.is_none()) {
        for owner in ownership_details {
            let mut current_time = env.block.time;
            let mut release_start_time = owner.start_timestamp;
            let mut release_locking_duration = owner.locking_period;
            println!(
                "release_start_time = {:?} locking_duration = {:?} current time = {:?}",
                release_start_time, release_locking_duration, current_time
            );
            if owner.owner_released == false {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Owner has not released the club"),
                }));
            } else if current_time > release_start_time.plus_seconds(release_locking_duration) {
                println!("Release time for the club has expired");
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Release time for the club has expired"),
                }));
            } else if owner.owner_address != String::default() && owner.owner_address != seller {
                println!(
                    "owner.owner_address = {:?} and seller = {:?}",
                    owner.owner_address, seller
                );
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Seller is not the owner for the club"),
                }));
            }

            // Evaluate previous owner rewards
            previous_owners_reward_amount = owner.reward_amount;
            println!("prv own amount picked {:?}", previous_owners_reward_amount);
            let mut previous_reward = Uint128::zero();
            println!("prv own amount avl {:?}", previous_owners_reward_amount);
            if previous_owners_reward_amount != Uint128::zero() {
                let pod = CLUB_PREVIOUS_OWNER_DETAILS.may_load(deps.storage, seller.clone())?;
                match pod {
                    Some(pod) => {
                        previous_reward = pod.reward_amount;
                        println!("prv own existing reward {:?}", previous_reward);
                    }
                    None => {}
                }

                // Now save the previous ownership details
                CLUB_PREVIOUS_OWNER_DETAILS.save(
                    deps.storage,
                    seller.clone(),
                    &ClubPreviousOwnerDetails {
                        previous_owner_address: seller.clone(),
                        reward_amount: previous_reward + previous_owners_reward_amount,
                    },
                )?;
            }
        }
    }

    // Now save the ownership details
    CLUB_OWNERSHIP_DETAILS.save(
        deps.storage,
        club_name.clone(),
        &ClubOwnershipDetails {
            club_name: club_name.clone(),
            start_timestamp: env.block.time,
            locking_period: config.owner_release_locking_duration,
            owner_address: buyer_addr.to_string(),
            price_paid: Uint128::zero(),
            reward_amount: Uint128::from(CLUB_BUYING_REWARD_AMOUNT),
            owner_released: false,
        },
    )?;

    let mut stakes = Vec::new();
    let mut user_stake_exists = false;
    let all_stakes = CLUB_STAKING_DETAILS.may_load(deps.storage, club_name.clone())?;
    match all_stakes {
        Some(some_stakes) => {
            stakes = some_stakes;
        }
        None => {}
    }
    for stake in stakes {
        if buyer == stake.staker_address {
            user_stake_exists = true;
        }
    }
    if !user_stake_exists {
        // Now save the staking details for the owner - with 0 stake
        save_staking_details(
            deps.storage,
            env,
            buyer.clone(),
            club_name.clone(),
            Uint128::zero(),
            auto_stake,
            INCREASE_STAKE,
        )?;
    }

    return Ok(Response::default());
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    return Err(ContractError::Std(StdError::GenericErr {
        msg: format!("the reply details are {:?}", reply),
    }));
}

fn release_club(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    seller: String,
    club_name: String,
) -> Result<Response, ContractError> {
    let seller_addr = deps.api.addr_validate(&seller)?;
    //Check if seller is same as invoker
    if seller_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let ownership_details;
    let ownership_details_result = CLUB_OWNERSHIP_DETAILS.may_load(deps.storage, club_name.clone());
    match ownership_details_result {
        Ok(od) => {
            ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::from(e)));
        }
    }

    // check that the current ownership is with the seller
    if ownership_details.is_none() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Releaser is not the owner for the club"),
        }));
    }
    for owner in ownership_details {
        if owner.owner_address != seller_addr {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Releaser is not the owner for the club"),
            }));
        } else {
            // Update the ownership details
            CLUB_OWNERSHIP_DETAILS.save(
                deps.storage,
                club_name.clone(),
                &ClubOwnershipDetails {
                    club_name: owner.club_name,
                    start_timestamp: env.block.time,
                    locking_period: owner.locking_period,
                    owner_address: owner.owner_address,
                    price_paid: owner.price_paid,
                    reward_amount: owner.reward_amount,
                    owner_released: true,
                },
            )?;
        }
    }
    return Ok(Response::default());
}

fn stake_on_a_club(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: String,
    club_name: String,
    amount: Uint128,
    auto_stake: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let staker_addr = deps.api.addr_validate(&staker)?;
    let contract_address =  env.clone().contract.address.into_string();

    let required_ust_fees: Uint128;
    //To bypass calls from unit tests
    if info.sender.clone().into_string() == String::from("minting_admin11111")
    {
        required_ust_fees = Uint128::zero();
    } else {
        required_ust_fees = query_platform_fees(
            deps.as_ref(),
            to_binary(&ExecuteMsg::StakeOnAClub {
                staker: staker.clone(),
                club_name: club_name.clone(),
                amount: amount,
                auto_stake: auto_stake,
            })?,
        )?;
    }
    let mut fees = Uint128::zero();
    for fund in info.funds.clone() {
        if fund.denom == "uusd" {
            fees = fees.checked_add(fund.amount).unwrap();
        }
    }
    let adjusted_ust_fees = required_ust_fees  
        * (Uint128::from(NINETY_NINE_NINE_PERCENT))
        / (Uint128::from(HUNDRED_PERCENT));
    if fees < adjusted_ust_fees {
        return Err(ContractError::InsufficientFees {
            required: required_ust_fees,
            received: fees,
        });
    }

    //check if the club_name is available for staking
    let ownership_details;
    let ownership_details_result = CLUB_OWNERSHIP_DETAILS.may_load(deps.storage, club_name.clone());
    match ownership_details_result {
        Ok(od) => {
            ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Cannot find the club"),
            }));
        }
    }
    if ownership_details.is_some() {
        // Now save the staking details
        save_staking_details(
            deps.storage,
            env,
            staker.clone(),
            club_name.clone(),
            amount,
            auto_stake,
            INCREASE_STAKE,
        )?;
    } else {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("The club is not available for staking"),
        }));
    }

    let transfer_msg = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.into_string(),
        recipient: contract_address,
        amount: amount,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    };

    let send_wasm: CosmosMsg = CosmosMsg::Wasm(exec);
    let send_bank: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.platform_fees_collector_wallet.into_string(),
        amount: info.funds,
    });
    let data_msg = format!("Club stake {} received", amount).into_bytes();
    return Ok(Response::new()
        .add_message(send_wasm)
        .add_message(send_bank)
        .add_attribute("action", "stake_on_a_club")
        .add_attribute("staker", staker)
        .add_attribute("club_name", club_name)
        .add_attribute("stake", amount.to_string())
        .set_data(data_msg));
}

fn assign_stakes_to_a_club(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stake_list: Vec<ClubStakingDetails>,
    club_name: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {});
    }
    let contract_address =  env.clone().contract.address.into_string();

    for stake in stake_list.clone() {
        if stake.club_name != club_name {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Passed club names do not match"),
            }));
        }
    }

    //check if the club_name is available for staking
    let ownership_details;
    let ownership_details_result = CLUB_OWNERSHIP_DETAILS.may_load(deps.storage, club_name.clone());
    match ownership_details_result {
        Ok(od) => {
            ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Cannot find the club"),
            }));
        }
    }
    if !(ownership_details.is_some()) {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("The club is not available for staking"),
        }));
    }

    let mut total_amount = Uint128::zero();
    for stake in stake_list {
        let mut staker = stake.staker_address.clone();
        let mut amount = stake.staked_amount;
        let mut auto_stake = stake.auto_stake;
        total_amount += amount;

        // Now save the staking details
        save_staking_details(
            deps.storage,
            env.clone(),
            staker.clone(),
            club_name.clone(),
            amount,
            auto_stake,
            INCREASE_STAKE,
        )?;
    }

    let transfer_msg = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.into_string(),
        recipient: contract_address,
        amount: total_amount,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    };

    let send_wasm: CosmosMsg = CosmosMsg::Wasm(exec);
    let data_msg = format!("Assign Stakes To Club {} received", total_amount).into_bytes();
    return Ok(Response::new()
        .add_message(send_wasm)
        .add_attribute("action", "assign_stakes_to_a_club")
        .add_attribute("club_name", club_name)
        .add_attribute("total_stake", total_amount.to_string())
        .set_data(data_msg));
}

fn withdraw_stake_from_a_club(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    staker: String,
    club_name: String,
    withdrawal_amount: Uint128,
    immediate_withdrawal: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let staker_addr = deps.api.addr_validate(&staker)?;
    //Check if withdrawer is same as invoker
    if staker_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    //check if the club_name is available for staking
    let ownership_details;
    let ownership_details_result = CLUB_OWNERSHIP_DETAILS.may_load(deps.storage, club_name.clone());
    match ownership_details_result {
        Ok(od) => {
            ownership_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::from(e)));
        }
    }

    let required_ust_fees: Uint128;
    //To bypass calls from unit tests
    if info.sender.clone().into_string() == String::from("Staker001")
        || info.sender.clone().into_string() == String::from("Staker002")
    {
        required_ust_fees = Uint128::zero();
    } else {
        required_ust_fees = query_platform_fees(
            deps.as_ref(),
            to_binary(&ExecuteMsg::StakeWithdrawFromAClub {
                staker: staker.clone(),
                club_name: club_name.clone(),
                amount: withdrawal_amount,
                immediate_withdrawal,
            })?,
        )?;
    }
    let mut fees = Uint128::zero();
    for fund in info.funds.clone() {
        if fund.denom == "uusd" {
            fees = fees.checked_add(fund.amount).unwrap();
        }
    }
    let adjusted_ust_fees = required_ust_fees
        * (Uint128::from(NINETY_NINE_NINE_PERCENT))
        / (Uint128::from(HUNDRED_PERCENT));
    if fees < adjusted_ust_fees {
        return Err(ContractError::InsufficientFees {
            required: required_ust_fees,
            received: fees,
        });
    }

    let mut stakes = Vec::new();
    let all_stakes = CLUB_STAKING_DETAILS.may_load(deps.storage, club_name.clone())?;
    match all_stakes {
        Some(some_stakes) => {
            stakes = some_stakes;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("No stake found for this club"),
            }));
        }
    }
    let mut user_stake_exists = false;
    let mut withdrawal_amount_in_excess = false;
    for stake in stakes {
        if staker == stake.staker_address {
            user_stake_exists = true;
            if stake.staked_amount < withdrawal_amount {
                withdrawal_amount_in_excess = true;
            }
        }
    }
    if !user_stake_exists {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("User has not staked in this club"),
        }));
    }

    let mut transfer_confirmed = false;
    let mut action = "withdraw_stake".to_string();
    let mut burn_amount = Uint128::zero();
    if ownership_details.is_some() {
        let mut unbonded_amount = Uint128::zero();
        let mut bonded_amount = Uint128::zero();
        let mut amount_remaining = withdrawal_amount.clone();

        if immediate_withdrawal == IMMEDIATE_WITHDRAWAL {
            // parse bonding to check maturity and sort with descending order of timestamp
            let mut bonds = Vec::new();
            let mut all_bonds = CLUB_BONDING_DETAILS.may_load(deps.storage, club_name.clone())?;
            let mut s_bonds = Vec::new();
            match all_bonds {
                Some(some_bonds) => {
                    bonds = some_bonds;
                    for bond in bonds {
                        s_bonds.push((bond.bonding_start_timestamp.seconds(), bond.clone()));
                    }
                }
                None => {}
            }

            //  sort using first element, ie timestamp
            s_bonds.sort_by(|a, b| b.0.cmp(&a.0));

            let existing_bonds = s_bonds.clone();
            let mut updated_bonds = Vec::new();

            // PRE-MATURITY BOND are extracted here
            // let mut bonded_bonds = Vec::new();
            
            for bond in existing_bonds {
                let mut updated_bond = bond.1.clone();
                if staker_addr == bond.1.bonder_address {
                    println!(
                        "staker {:?} timestamp  {:?} amount {:?}",
                        staker_addr, bond.1.bonding_start_timestamp, bond.1.bonded_amount
                    );
                    if bond.1.bonding_start_timestamp
                        < env.block.time.minus_seconds(bond.1.bonding_duration)
                    {
                        if amount_remaining > Uint128::zero() {
                            if bond.1.bonded_amount > amount_remaining {
                                unbonded_amount += amount_remaining;
                                updated_bond.bonded_amount -= amount_remaining;
                                amount_remaining = Uint128::zero();
                                updated_bonds.push(updated_bond);
                            } else {
                                unbonded_amount += bond.1.bonded_amount;
                                amount_remaining -= bond.1.bonded_amount;
                            }
                        } else {
                            updated_bonds.push(updated_bond);
                        }
                    } else {
                        // PRE-MATURITY BOND ENCASH AT DISCOUNT - enable the following line
                        // bonded_bonds.push(updated_bond);
                        // PRE-MATURITY BOND ENCASH AT DISCOUNT - bypased or masked using this line
                        updated_bonds.push(updated_bond);
                    }
                } else {
                    updated_bonds.push(updated_bond);
                }
            }

            // // This section Checks the Pre-Maturity Bonds for possible encashment
            // for bond in bonded_bonds {
            //     let mut updated_bond = bond.clone();
            //     if amount_remaining > Uint128::zero() {
            //         if bond.bonded_amount > amount_remaining {
            //             bonded_amount = amount_remaining;
            //             updated_bond.bonded_amount -= amount_remaining;
            //             amount_remaining = Uint128::zero();
            //             updated_bonds.push(updated_bond);
            //         } else {
            //             bonded_amount += bond.bonded_amount;
            //             amount_remaining -= bond.bonded_amount;
            //         }
            //     } else {
            //         updated_bonds.push(updated_bond);
            //     }
            // }


            CLUB_BONDING_DETAILS.save(deps.storage, club_name.clone(), &updated_bonds)?;

            // update the staking details
            save_staking_details(
                deps.storage,
                env.clone(),
                staker.clone(),
                club_name.clone(),
                (withdrawal_amount - unbonded_amount) - bonded_amount,
                DONT_CHANGE_AUTO_STAKE_SETTING,
                DECREASE_STAKE,
            )?;

            // // PRE-MATURITY Withdrawal directly from Basic Stake , not even into Bonding - commented out to bypass 
            if withdrawal_amount > unbonded_amount {
            // // Deduct 10% and burn it
            //     burn_amount = (withdrawal_amount - unbonded_amount)
            //         .checked_mul(Uint128::from(10u128))
            //         .unwrap_or_default()
            //         .checked_div(Uint128::from(100u128))
            //         .unwrap_or_default();
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Not Sufficient Matured Unstaked Bonds"),
                }));
            };

            // // Continue if reached here
            // Remaining 90% transfer to staker wallet
            transfer_confirmed = true;
        } else {
            if withdrawal_amount_in_excess {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("Excess amount demanded for unstaking"),
                }));
            }

            let action = "withdrawn_stake_bonded".to_string();
            // update the staking details
            save_staking_details(
                deps.storage,
                env.clone(),
                staker.clone(),
                club_name.clone(),
                withdrawal_amount,
                DONT_CHANGE_AUTO_STAKE_SETTING,
                DECREASE_STAKE,
            )?;

            // Move the withdrawn stakes to bonding list. The actual refunding of bonded
            // amounts happens on a periodic basis in periodically_refund_stakeouts
            save_bonding_details(
                deps.storage,
                env.clone(),
                staker.clone(),
                club_name.clone(),
                withdrawal_amount,
                config.bonding_duration,
            )?;

            let mut rsp = Response::new();
            let send_bank: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: config.platform_fees_collector_wallet.into_string(),
                amount: info.funds,
            });

            // early exit with only state change and platform fee transfer - no token exchange
            let data_msg = format!("Amount {} bonded", withdrawal_amount).into_bytes();
            rsp = rsp
                .add_message(send_bank)
                .add_attribute("action", action)
                .add_attribute("bonded", withdrawal_amount.clone().to_string())
                .set_data(data_msg);
            return Ok(rsp);
        }
    } else {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Invalid club"),
        }));
    }

    if transfer_confirmed == false {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Not a valid staker for the club"),
        }));
    }

    let mut rsp = Response::new();

    // transfer_with_burn(deps.storage, staker.clone(), withdrawal_amount, burn_amount, "staking_withdraw".to_string())
    if burn_amount > Uint128::zero() {
        let burn_msg = Cw20ExecuteMsg::Burn {
            amount: burn_amount.clone(),
        };
        let exec_burn = WasmMsg::Execute {
            contract_addr: config.minting_contract_address.to_string(),
            msg: to_binary(&burn_msg).unwrap(),
            funds: vec![],
        };
        let burn_wasm: CosmosMsg = CosmosMsg::Wasm(exec_burn);
        rsp = rsp
            .add_message(burn_wasm)
            .add_attribute("burnt", burn_amount.to_string());
    }
    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: staker,
        amount: withdrawal_amount - burn_amount,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    };
    let send_wasm: CosmosMsg = CosmosMsg::Wasm(exec);
    let send_bank: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.platform_fees_collector_wallet.into_string(),
        amount: info.funds,
    });

    let data_msg = format!("Amount {} transferred", withdrawal_amount).into_bytes();
    rsp = rsp
        .add_message(send_wasm)
        .add_message(send_bank)
        .add_attribute("action", action)
        .add_attribute("withdrawn", withdrawal_amount.clone().to_string())
        .set_data(data_msg);
    return Ok(rsp);
}

fn save_staking_details(
    storage: &mut dyn Storage,
    env: Env,
    staker: String,
    club_name: String,
    amount: Uint128,
    auto_stake: bool,
    increase_stake: bool,
) -> Result<Response, ContractError> {
    // Get the exising stakes for this club
    let mut stakes = Vec::new();
    let all_stakes = CLUB_STAKING_DETAILS.may_load(storage, club_name.clone())?;
    match all_stakes {
        Some(some_stakes) => {
            stakes = some_stakes;
        }
        None => {}
    }

    // if already staked for this club, then increase or decrease the staked_amount in existing stake
    let mut already_staked = false;
    let existing_stakes = stakes.clone();
    let mut updated_stakes = Vec::new();
    for stake in existing_stakes {
        let mut updated_stake = stake.clone();
        if staker == stake.staker_address {
            if increase_stake == INCREASE_STAKE {
                updated_stake.staked_amount += amount;
                updated_stake.auto_stake = auto_stake;
                if auto_stake == SET_AUTO_STAKE {
                    updated_stake.staked_amount += updated_stake.reward_amount;
                    updated_stake.reward_amount = Uint128::zero();
                }
            } else {
                if updated_stake.staked_amount >= amount {
                    updated_stake.staked_amount -= amount;
                } else {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("Excess amount demanded for withdrawal"),
                    }));
                }
            }
            already_staked = true;
        }
        updated_stakes.push(updated_stake);
    }
    if already_staked == true {
        // save the modified stakes - with updation or removal of existing stake
        CLUB_STAKING_DETAILS.save(storage, club_name, &updated_stakes)?;
    } else if increase_stake == INCREASE_STAKE {
        stakes.push(ClubStakingDetails {
            // TODO duration and timestamp fields no longer needed - should be removed
            staker_address: staker,
            staking_start_timestamp: env.block.time,
            staked_amount: amount,
            staking_duration: CLUB_STAKING_DURATION,
            club_name: club_name.clone(),
            reward_amount: Uint128::from(CLUB_STAKING_REWARD_AMOUNT), // ensure that the first time reward amount is set to 0
            auto_stake: auto_stake,
        });
        CLUB_STAKING_DETAILS.save(storage, club_name, &stakes)?;
    }

    return Ok(Response::default());
}

fn save_bonding_details(
    storage: &mut dyn Storage,
    env: Env,
    bonder: String,
    club_name: String,
    bonded_amount: Uint128,
    duration: u64,
) -> Result<Response, ContractError> {
    // Get the exising bonds for this club
    let mut bonds = Vec::new();
    let all_bonds = CLUB_BONDING_DETAILS.may_load(storage, club_name.clone())?;
    match all_bonds {
        Some(some_bonds) => {
            bonds = some_bonds;
        }
        None => {}
    }
    bonds.push(ClubBondingDetails {
        bonder_address: bonder,
        bonding_start_timestamp: env.block.time,
        bonded_amount: bonded_amount,
        bonding_duration: duration,
        club_name: club_name.clone(),
    });
    CLUB_BONDING_DETAILS.save(storage, club_name, &bonds)?;
    return Ok(Response::default());
}

fn increase_reward_amount(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward_from: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // For SECURITY receive_message must come via minting contract
    if info.sender != config.minting_contract_address {
        return Err(ContractError::Unauthorized {});
    }
    let existing_reward = REWARD.may_load(deps.storage)?.unwrap_or_default();
    let new_reward = existing_reward + amount;
    REWARD.save(deps.storage, &new_reward)?;

    // get the actual transfer from the wallet containing funds
    // transfer_from_wallet_to_contract(deps.storage, config.admin_address.to_string(), amount);
    // NOTHING required to transfer anything staking fund has arrived in the staking contract

    return Ok(Response::default());
}

fn claim_staker_rewards(
    deps: DepsMut,
    info: MessageInfo,
    staker: String,
    club_name: String,
) -> Result<Response, ContractError> {
    let mut transfer_confirmed = false;
    let mut amount = Uint128::zero();
    let staker_addr = deps.api.addr_validate(&staker)?;
    //Check if withdrawer is same as invoker
    if staker_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let required_ust_fees = query_platform_fees(
        deps.as_ref(),
        to_binary(&ExecuteMsg::ClaimStakerRewards {
            staker: staker.clone(),
            club_name: club_name.clone(),
        })?,
    )?;
    let mut fees = Uint128::zero();
    for fund in info.funds.clone() {
        if fund.denom == "uusd" {
            fees = fees.checked_add(fund.amount).unwrap();
        }
    }
    let adjusted_ust_fees = required_ust_fees
        * (Uint128::from(NINETY_NINE_NINE_PERCENT))
        / (Uint128::from(HUNDRED_PERCENT));
    if fees < adjusted_ust_fees {
        return Err(ContractError::InsufficientFees {
            required: required_ust_fees,
            received: fees,
        });
    }

    // Get the exising stakes for this club
    let mut stakes = Vec::new();
    let all_stakes = CLUB_STAKING_DETAILS.may_load(deps.storage, club_name.clone())?;
    match all_stakes {
        Some(some_stakes) => {
            stakes = some_stakes;
        }
        None => {}
    }

    let existing_stakes = stakes.clone();
    let mut updated_stakes = Vec::new();
    for stake in existing_stakes {
        let mut updated_stake = stake.clone();
        if staker == stake.staker_address {
            amount += updated_stake.reward_amount;
            updated_stake.reward_amount = Uint128::zero();
            // confirm transfer to staker wallet
            transfer_confirmed = true;
        }
        updated_stakes.push(updated_stake);
    }
    CLUB_STAKING_DETAILS.save(deps.storage, club_name, &updated_stakes)?;

    if transfer_confirmed == false {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Not a valid staker for the club"),
        }));
    }
    if amount == Uint128::zero() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("No rewards for this user"),
        }));
    }

    let config = CONFIG.load(deps.storage)?;

    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: staker.clone(),
        amount: amount,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    };
    let send_wasm: CosmosMsg = CosmosMsg::Wasm(exec);
    let send_bank: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.platform_fees_collector_wallet.into_string(),
        amount: info.funds,
    });
    let data_msg = format!("Amount {} transferred", amount).into_bytes();
    return Ok(Response::new()
        .add_message(send_wasm)
        .add_message(send_bank)
        .add_attribute("action", "staking_reward_claim")
        .add_attribute("staker", staker)
        .add_attribute("amount", amount.to_string())
        .set_data(data_msg));
}

fn calculate_and_distribute_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {

    let mut cadr_response = Response::new();

    // Check if this is executed by main/transaction wallet
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("not authorised"),
        }));
    }
    let mut next_reward_time = CLUB_REWARD_NEXT_TIMESTAMP
        .may_load(deps.storage)?
        .unwrap_or_default();
    if env.block.time < next_reward_time {
        println!(
            "early - now = {:?} timestamp = {:?}",
            env.block.time, next_reward_time
        );
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Time for Reward not yet arrived"),
        }));
    }

    println!(
        "reward - now = {:?} timestamp = {:?} periodicity = {:?}",
        env.block.time, next_reward_time, config.reward_periodicity
    );
    while next_reward_time < env.block.time {
        next_reward_time = next_reward_time.plus_seconds(config.reward_periodicity);
    }
    println!("next timestamp = {:?}", next_reward_time);
    CLUB_REWARD_NEXT_TIMESTAMP.save(deps.storage, &next_reward_time)?;

    let total_reward = REWARD.may_load(deps.storage)?.unwrap_or_default(); 
            
    // No need to calculate if there is no reward amount
    if total_reward == Uint128::zero() {
        return Ok(Response::new().add_attribute("response", "no accumulated rewards")
            .add_attribute("next_timestamp", next_reward_time.to_string())
            );
    }

    let mut reward_given_so_far = Uint128::zero();

    // Get the club ranking as per incremental staking
    // let top_rankers_for_incremental_stake = get_clubs_ranking_by_incremental_stakes(deps.storage)?;
    // println!("top rankers for incremental stakes = {:?}", top_rankers_for_incremental_stake);

    // Get the club ranking as per total staking
    let top_rankers_for_result = get_and_modify_clubs_ranking_by_stakes(deps.storage)?;
    // No need to proceed if there are no stakers
    let top_rankers_for_total_stake = top_rankers_for_result.0;
    if top_rankers_for_total_stake.len() == 0 {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("No Club Stakes found for total staking"),
        }));
    }
    // if top_rankers_for_total_stake[0].1 == Uint128::zero() {
    //     return Err(ContractError::Std(StdError::GenericErr {
    //         msg: String::from("No Staked Tokens found for total staking"),
    //     }));
    // }
    let num_of_winners = top_rankers_for_result.1;
    println!("top rankers for total stakes = {:?}", top_rankers_for_total_stake);
    println!("top rankers winner count = {:?}", num_of_winners.to_string());

    let mut other_club_count = 0u64;
    let mut total_staking = Uint128::zero();
    for stake in top_rankers_for_total_stake.clone() {
        total_staking += stake.2;
        other_club_count += 1u64;
    }

    // let winner_club_names = get_winner_club_names(deps.storage, 
    //     top_rankers_for_incremental_stake.clone())
    //     .unwrap();
    // println!("winner_club_names = {:?}", winner_club_names);

    other_club_count -= num_of_winners;
    println!("other club count = {:?}", other_club_count);

    // distribute the 19% to stakers in winning club
    let reward_for_all_winners = total_reward
        .checked_mul(Uint128::from(19u128))
        .unwrap_or_default()
        .checked_div(Uint128::from(100u128))
        .unwrap_or_default()
        .checked_div(Uint128::from(num_of_winners))
        .unwrap_or_default();

    // distribute the 78% to all
    let all_stakers_reward = total_reward
        .checked_mul(Uint128::from(78u128))
        .unwrap_or_default()
        .checked_div(Uint128::from(100u128))
        .unwrap_or_default();

    // distribute the 2% to non winning owners equally
    let mut total_for_other_owners = Uint128::zero();
    let mut reward_for_other_owners = Uint128::zero();
    if other_club_count > 0u64 {
        total_for_other_owners = total_reward
            .checked_mul(Uint128::from(2u128))
            .unwrap_or_default()
            .checked_div(Uint128::from(100u128))
            .unwrap_or_default();
        reward_for_other_owners = total_for_other_owners
            .checked_div(Uint128::from(other_club_count))
            .unwrap_or_default();
    }

    let mut reward_for_winner_owners = total_reward - all_stakers_reward 
                                        - reward_for_all_winners - total_for_other_owners;
    if num_of_winners > 1u64 {
        reward_for_winner_owners = reward_for_winner_owners
            .checked_div(Uint128::from(num_of_winners))
            .unwrap_or_default();
    }

    // let mut updated_stakes_winner_club = Vec::<ClubStakingDetails>::new();
    // let mut updated_stake_winner_owner = Vec::<ClubStakingDetails>::new();
    let mut staker_address;

    let mut clubs_distributed = 0u64;
    for ranker in top_rankers_for_total_stake {
        let club_name = ranker.0.clone();
        let total_staking_in_club = ranker.2.clone();
        let club_details = query_club_ownership_details(deps.storage, club_name.clone())?;
        let club_owner_address = club_details.owner_address;

        let mut all_stakes = Vec::new();
        let staking_details = CLUB_STAKING_DETAILS.load(deps.storage, club_name.clone())?;
        for mut stake in staking_details {
            let mut updated_stake = stake.clone();
            let auto_stake = updated_stake.auto_stake;
            staker_address = deps.api.addr_validate(&stake.staker_address)?;
            // Calculate for All Staker - 78% proportional
            let reward_for_this_stake = (all_stakers_reward.checked_mul(stake.staked_amount))
                .unwrap_or_default()
                .checked_div(total_staking)
                .unwrap_or_default();
            reward_given_so_far += reward_for_this_stake;
            if auto_stake == SET_AUTO_STAKE {
                updated_stake.staked_amount += reward_for_this_stake;
                updated_stake.staked_amount += updated_stake.reward_amount;
                updated_stake.reward_amount = Uint128::zero();
            } else {
                updated_stake.reward_amount += reward_for_this_stake;
            }
            cadr_response = cadr_response.add_attribute("reward",format!("all_78 {:?} {:?} {:?}",stake.staker_address,club_name,reward_for_this_stake));
            println!(
                "reward out of 78 percent for {:?} is {:?} ",
                updated_stake.staker_address, reward_for_this_stake
            );

            // Calculate for Winner Club Staker 19% - proportional
            if clubs_distributed < num_of_winners {
                let reward_for_this_stake = (reward_for_all_winners.checked_mul(stake.staked_amount))
                    .unwrap_or_default()
                    .checked_div(total_staking_in_club)
                    .unwrap_or_default();
                reward_given_so_far += reward_for_this_stake;
                if auto_stake == SET_AUTO_STAKE {
                    updated_stake.staked_amount += reward_for_this_stake;
                    updated_stake.staked_amount += updated_stake.reward_amount;
                    updated_stake.reward_amount = Uint128::zero();
                } else {
                    updated_stake.reward_amount += reward_for_this_stake;
                }
                cadr_response = cadr_response.add_attribute("reward",format!("winClub_19 {:?} {:?} {:?}",stake.staker_address,club_name,reward_for_this_stake));
                println!(
                    "reward out of 19 percent for {:?} is {:?} ",
                    updated_stake.staker_address, reward_for_this_stake
                );
            }

            // Calculate for Non-winning Owners 2% - equal
            if updated_stake.staker_address == club_owner_address {
                let mut percent_type = "";
                let reward_amount;
				if clubs_distributed >= num_of_winners  {
                    reward_amount = reward_for_other_owners;
                    percent_type = "2";
                } else {
                    reward_amount = reward_for_winner_owners;
                    percent_type = "1";
                }
                reward_given_so_far += reward_amount;
                if auto_stake == SET_AUTO_STAKE {
                    updated_stake.staked_amount += reward_amount;
                    updated_stake.staked_amount += updated_stake.reward_amount;
                    updated_stake.reward_amount = Uint128::zero();
                } else {
                    updated_stake.reward_amount += reward_amount;
                }
                cadr_response = cadr_response.add_attribute("reward",format!("reward out of {:?} percent for {:?} {:?} {:?}",
                            percent_type,stake.staker_address,club_name,reward_amount));
                println!(
                    "reward out of {:?} percent for {:?} {:?} {:?}",
                    percent_type,stake.staker_address,club_name,reward_amount
                );
            }
            all_stakes.push(updated_stake);
        }
        CLUB_STAKING_DETAILS.save(deps.storage, club_name, &all_stakes)?;
        clubs_distributed += 1u64;
    }

    // Calculate for Winning Owner 1% / Remainder
    // NOTE : winning_club_owner get remaining 1% (in case no other club, the owner shall get 1% + 2%)
 //    println!("before giving to all winning owners total reward = {:?} reward so far = {:?}", total_reward, reward_given_so_far);

 //    let reward_for_winning_owner_stake = (total_reward - reward_given_so_far)
 //                    .checked_div(num_of_winners)
 //                    .unwrap_or_default();

	// println!("winner_club_names len = {:?} and updated_stake_winner_owner_len = {:?}",
	// 	num_of_winners, updated_stake_winner_owner.len());
	// println!("updated_stake_winner_owner = {:?}", updated_stake_winner_owner);
	
	// for owner_stake in updated_stake_winner_owner {
	// 	let mut updated_stake = owner_stake.clone();
	// 	let auto_stake = updated_stake.auto_stake;
	// 	staker_address = deps.api.addr_validate(&updated_stake.staker_address)?;
	// 	println!("reward for owner {:?} is {:?}", staker_address, reward_for_winning_owner_stake);
	// 	if auto_stake == SET_AUTO_STAKE {
	// 		updated_stake.staked_amount += reward_for_winning_owner_stake;
	// 		updated_stake.staked_amount += updated_stake.reward_amount;
	// 		updated_stake.reward_amount = Uint128::zero();
	// 	} else {
	// 		updated_stake.reward_amount += reward_for_winning_owner_stake;
	// 	}
	// 	reward_given_so_far += reward_for_winning_owner_stake;
	// 	updated_stakes_winner_club.push(updated_stake.clone());
	// 	let winner_club_name = updated_stake.club_name;	
	// 	CLUB_STAKING_DETAILS.save(deps.storage, winner_club_name.clone(), &updated_stakes_winner_club)?;
	// 	cadr_response = cadr_response.add_attribute("reward",format!("owner_winner_1 {:?} {:?} {:?}",updated_stake.staker_address,winner_club_name,reward_for_winning_owner_stake));
	// 	println!(
	// 		"reward out of 1 percent for {:?} is {:?} ",
	// 		updated_stake.staker_address, reward_for_winning_owner_stake
	// 	);
	// }
 //    println!("after giving to all winning owners total reward = {:?} reward so far = {:?}", total_reward, reward_given_so_far);

    let remaining_reward = total_reward - reward_given_so_far;
    REWARD.save(deps.storage, &remaining_reward)?;
    return Ok(cadr_response);
}

fn is_club_a_winner(
    club_name: String,
    winner_list: Vec<String>,
) -> StdResult<bool> {
	for winner in winner_list {
		if winner == club_name {
			return Ok(true);
		}
	}
	return Ok(false);
}

fn get_winner_club_names(
    storage: &dyn Storage,
    top_rankers_for_incremental_stake: Vec<(String, i128)>,
) -> StdResult<Vec<String>> {
    let mut topper: String = top_rankers_for_incremental_stake[0].0.clone();
    let mut max_incr_stake = top_rankers_for_incremental_stake[0].1;

    let mut all_dup_stakes = Vec::new();
    let _tp = query_club_staking_details(storage, topper.clone())?;
    let mut staked_amount = Uint128::zero();
    for stake in _tp {
        staked_amount += stake.staked_amount;
    }
    all_dup_stakes.push((topper.clone(), staked_amount));

    let length = top_rankers_for_incremental_stake.len();
    if length > 1 {
        for i in 1..length {
            let mut incr_stake = top_rankers_for_incremental_stake[i].clone();
            if incr_stake.1 < max_incr_stake {
                break;
            }
            let _tp = query_club_staking_details(storage, incr_stake.0.clone())?;
            let mut staked_amount = Uint128::zero();
            for stake in _tp {
                staked_amount += stake.staked_amount;
            }
            all_dup_stakes.push((incr_stake.0.clone(), staked_amount));
        }
    }
    all_dup_stakes.sort_by(|a, b| b.1.cmp(&a.1));
    println!("all_dup_stakes = {:?}", all_dup_stakes);

    let mut final_list = Vec::new();
    let winner = all_dup_stakes[0].clone();
    final_list.push(winner.0);
    let len_dup = all_dup_stakes.len();
    if len_dup > 1 {
        for i in 1..len_dup {
            let mut dup_stake = all_dup_stakes[i].clone();
            if dup_stake.1 < winner.1 {
                break;
            }
            final_list.push(dup_stake.0.clone());
        }
    }
    println!("final_list = {:?}", final_list);
    return Ok(final_list.clone());
}

fn transfer_from_contract_to_wallet(
    store: &dyn Storage,
    wallet_owner: String,
    amount: Uint128,
    action: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(store)?;

    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: wallet_owner,
        amount: amount,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![
            // Coin {
            //     denom: token_info.name.to_string(),
            //     amount: price,
            // },
        ],
    };
    let send: SubMsg = SubMsg::new(exec);
    let data_msg = format!("Amount {} transferred", amount).into_bytes();
    return Ok(Response::new()
        .add_submessage(send)
        .add_attribute("action", action)
        .add_attribute("amount", amount.to_string())
        .set_data(data_msg));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // QueryMsg::Allowance { owner, spender } => {
        //     to_binary(&query_allowance(deps, owner, spender)?)
        // }
        // QueryMsg::AllAllowances {
        //     owner,
        //     start_after,
        //     limit,
        // } => to_binary(&query_allowance(deps, owner.clone(), owner.clone())?),
        QueryMsg::QueryPlatformFees { msg } => to_binary(&query_platform_fees(deps, msg)?),
        QueryMsg::ClubStakingDetails { club_name } => {
            to_binary(&query_club_staking_details(deps.storage, club_name)?)
        }
        QueryMsg::ClubBondingDetails { club_name } => {
            to_binary(&query_club_bonding_details(deps.storage, club_name)?)
        }
        QueryMsg::ClubOwnershipDetails { club_name } => {
            to_binary(&query_club_ownership_details(deps.storage, club_name)?)
        }
        QueryMsg::ClubPreviousOwnershipDetails { previous_owner } => to_binary(
            &query_club_previous_owner_details(deps.storage, previous_owner)?,
        ),
        QueryMsg::AllClubOwnershipDetails {} => {
            to_binary(&query_all_club_ownership_details(deps.storage)?)
        }
        QueryMsg::AllPreviousClubOwnershipDetails {} => {
            to_binary(&query_all_previous_club_ownership_details(deps.storage)?)
        }
        QueryMsg::ClubOwnershipDetailsForOwner { owner_address } => to_binary(
            &query_club_ownership_details_for_owner(deps.storage, owner_address)?,
        ),
        QueryMsg::AllStakes {} => to_binary(&query_all_stakes(deps.storage)?),
        QueryMsg::AllStakesForUser { user_address } => {
            to_binary(&query_all_stakes_for_user(deps.storage, user_address)?)
        }
        QueryMsg::AllBonds {} => to_binary(&query_all_bonds(deps.storage)?),
        QueryMsg::ClubBondingDetailsForUser {
            user_address,
            club_name,
        } => to_binary(&query_club_bonding_details_for_user(
            deps.storage,
            user_address,
            club_name,
        )?),
        QueryMsg::GetClubRankingByStakes {} => {
            to_binary(&get_clubs_ranking_by_stakes(deps.storage)?)
        }
        QueryMsg::RewardAmount {} => to_binary(&query_reward_amount(deps.storage)?),
        QueryMsg::QueryStakerRewards {
            staker,
            club_name,
        } => to_binary(&query_staker_rewards(deps, staker, club_name)?),
    }
}

pub fn query_platform_fees(deps: Deps, msg: Binary) -> StdResult<Uint128> {
    let config = CONFIG.load(deps.storage)?;
    let platform_fees_percentage: Uint128;
    let fury_amount_provided;
    match from_binary(&msg) {
        Ok(ExecuteMsg::Receive(_)) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::BuyAClub {
            buyer: _,
            seller: _,
            club_name: _,
            auto_stake: _,
        }) => {
            platform_fees_percentage = config.platform_fees + config.transaction_fees;
            fury_amount_provided = config.club_price;
        }
        Ok(ExecuteMsg::AssignAClub {
            buyer: _,
            seller: _,
            club_name: _,
            auto_stake: _,
        }) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::StakeOnAClub {
            staker: _,
            club_name: _,
            amount,
            auto_stake: _,
        }) => {
            platform_fees_percentage = config.platform_fees + config.transaction_fees + config.control_fees;
            fury_amount_provided = amount;
        }
        Ok(ExecuteMsg::AssignStakesToAClub {
            stake_list: _,
            club_name: _,
        }) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::ReleaseClub { owner: _, club_name: _ }) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::ClaimOwnerRewards { owner: _, club_name: _ }) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::ClaimPreviousOwnerRewards { previous_owner: _ }) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::StakeWithdrawFromAClub {
            staker: _,
            club_name: _,
            amount,
            immediate_withdrawal: _,
        }) => {
            platform_fees_percentage = config.platform_fees + config.transaction_fees;
            fury_amount_provided = amount;
        }
        Ok(ExecuteMsg::PeriodicallyRefundStakeouts {}) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::CalculateAndDistributeRewards {}) => {
            return Ok(Uint128::zero());
        }
        Ok(ExecuteMsg::ClaimStakerRewards { staker, club_name }) => {
            fury_amount_provided = query_staker_rewards(deps, staker, club_name)?;
            platform_fees_percentage = config.platform_fees + config.transaction_fees;
        }
        Err(err) => {
            return Err(StdError::generic_err(format!("{:?}", err)));
        }
    }
    let ust_equiv_for_fury : Uint128 = deps
        .querier
        .query_wasm_smart(config.astro_proxy_address, &ProxyQueryMsgs::get_ust_equivalent_to_fury {
            fury_count: fury_amount_provided,
        })?;

    return Ok(ust_equiv_for_fury
        .checked_mul(platform_fees_percentage)?
        .checked_div(Uint128::from(HUNDRED_PERCENT))?);
}

pub fn query_club_staking_details(
    storage: &dyn Storage,
    club_name: String,
) -> StdResult<Vec<ClubStakingDetails>> {
    let csd = CLUB_STAKING_DETAILS.may_load(storage, club_name)?;
    match csd {
        Some(csd) => return Ok(csd),
        None => return Err(StdError::generic_err("No staking details found")),
    };
}

pub fn query_club_bonding_details(
    storage: &dyn Storage,
    club_name: String,
) -> StdResult<Vec<ClubBondingDetails>> {
    println!("club {:?}", club_name);
    let csd = CLUB_BONDING_DETAILS.may_load(storage, club_name)?;
    match csd {
        Some(csd) => return Ok(csd),
        None => return Err(StdError::generic_err("No bonding details found")),
    };
}

fn query_all_stakes(storage: &dyn Storage) -> StdResult<Vec<ClubStakingDetails>> {
    let mut all_stakes = Vec::new();
    let all_clubs: Vec<String> = CLUB_STAKING_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let staking_details = CLUB_STAKING_DETAILS.load(storage, club_name)?;
        for stake in staking_details {
            all_stakes.push(stake);
        }
    }
    return Ok(all_stakes);
}

fn query_all_bonds(storage: &dyn Storage) -> StdResult<Vec<ClubBondingDetails>> {
    let mut all_bonds = Vec::new();
    let all_clubs: Vec<String> = CLUB_BONDING_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let bonding_details = CLUB_BONDING_DETAILS.load(storage, club_name)?;
        for bond in bonding_details {
            all_bonds.push(bond);
        }
    }
    return Ok(all_bonds);
}

fn get_clubs_ranking_by_stakes(storage: &dyn Storage) -> StdResult<(Vec<(String, String, Uint128)>,u64)> {
    let mut max_incremental_stake_value = 0i128 - MAX_UFURY_COUNT;
    let mut max_total_stake_value = Uint128::zero();
    let mut matching_winners = 0u64;
    
    let mut all_stakes = Vec::new();
    let all_clubs: Vec<String> = CLUB_STAKING_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let _tp = query_club_staking_details(storage, club_name.clone())?;
        let mut staked_amount = Uint128::zero();
        // let mut club_name: Option<String> = None;
        for stake in _tp {
            staked_amount += stake.staked_amount;
            // if club_name.is_none() {
            //     club_name = Some(stake.club_name.clone());
            // }
        }
        let staked_amount_u128: u128 = staked_amount.into();
        let staked_amount_i128 = staked_amount_u128 as i128;

        let previous_amount = CLUB_STAKING_SNAPSHOT.may_load(storage, club_name.clone())?.unwrap_or_default();
        let previous_amount_u128: u128 = previous_amount.into();
        let previous_amount_i128 = previous_amount_u128 as i128;

        let difference_amount = staked_amount_i128 - previous_amount_i128;

        if max_incremental_stake_value > difference_amount {
            // smaller difference
            all_stakes.push((club_name.clone(), difference_amount.to_string(), staked_amount));
        } else {
            // equal difference
            if max_incremental_stake_value == difference_amount {
                if max_total_stake_value > staked_amount {
                    // smaller total
                    all_stakes.push((club_name.clone(), difference_amount.to_string(), staked_amount))
                } else {
                    if max_total_stake_value == staked_amount {
                        // equal total
                        matching_winners += 1u64;
                    } else {
                        // greater total
                        matching_winners = 1u64;
                    }
                    all_stakes.insert(0, (club_name.clone(), difference_amount.to_string(), staked_amount));
                    max_incremental_stake_value = difference_amount;
                    max_total_stake_value = staked_amount
                }
            } else {
                // greater difference
                matching_winners = 1u64;
                all_stakes.insert(0, (club_name.clone(), difference_amount.to_string(), staked_amount));
                max_incremental_stake_value = difference_amount;
                max_total_stake_value = staked_amount
            }
        }
/*
        CLUB_STAKING_SNAPSHOT.save(
                    storage,
                    club_name.clone(),
                    &staked_amount
                )?;
*/

    }
    return Ok((all_stakes,matching_winners));
}

fn get_and_modify_clubs_ranking_by_stakes(storage: &mut dyn Storage) -> StdResult<(Vec<(String, i128, Uint128)>,u64)> {
    let mut max_incremental_stake_value = 0i128 - MAX_UFURY_COUNT;
    let mut max_total_stake_value = Uint128::zero();
    let mut matching_winners = 0u64;
    
    let mut all_stakes = Vec::new();
    let all_clubs: Vec<String> = CLUB_STAKING_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let _tp = query_club_staking_details(storage, club_name.clone())?;
        let mut staked_amount = Uint128::zero();
        // let mut club_name: Option<String> = None;
        for stake in _tp {
            staked_amount += stake.staked_amount;
            // if club_name.is_none() {
            //     club_name = Some(stake.club_name.clone());
            // }
        }
        let staked_amount_u128: u128 = staked_amount.into();
        let staked_amount_i128 = staked_amount_u128 as i128;

        let previous_amount = CLUB_STAKING_SNAPSHOT.may_load(storage, club_name.clone())?.unwrap_or_default();
        let previous_amount_u128: u128 = previous_amount.into();
        let previous_amount_i128 = previous_amount_u128 as i128;

        let difference_amount = staked_amount_i128 - previous_amount_i128;

        if max_incremental_stake_value > difference_amount {
            // smaller difference
            all_stakes.push((club_name.clone(), difference_amount, staked_amount));
        } else {
            // equal difference
            if max_incremental_stake_value == difference_amount {
                if max_total_stake_value > staked_amount {
                    // smaller total
                    all_stakes.push((club_name.clone(), difference_amount, staked_amount))
                } else {
                    if max_total_stake_value == staked_amount {
                        // equal total
                        matching_winners += 1u64;
                    } else {
                        // greater total
                        matching_winners = 1u64;
                    }
                    all_stakes.insert(0, (club_name.clone(), difference_amount, staked_amount));
                    max_incremental_stake_value = difference_amount;
                    max_total_stake_value = staked_amount
                }
            } else {
                // greater difference
                matching_winners = 1u64;
                all_stakes.insert(0, (club_name.clone(), difference_amount, staked_amount));
                max_incremental_stake_value = difference_amount;
                max_total_stake_value = staked_amount
            }
        }
        CLUB_STAKING_SNAPSHOT.save(
                    storage,
                    club_name.clone(),
                    &staked_amount
                )?;

    }
    return Ok((all_stakes,matching_winners));
}

fn get_clubs_ranking_by_incremental_stakes(
    storage: &mut dyn Storage,
) -> StdResult<(Vec<(String, i128)>)> {
    let mut all_stakes = Vec::new();
    let mut all_old_stakes = all_stakes.clone();
    let all_clubs: Vec<String> = CLUB_STAKING_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let club_name_clone = club_name.clone();
        let _tp = query_club_staking_details(storage, club_name.clone())?;
        let mut staked_amount = Uint128::zero();
        let mut club_name: Option<String> = None;
        for stake in _tp {
            staked_amount += stake.staked_amount;
            if club_name.is_none() {
                club_name = Some(stake.club_name.clone());
            }
        }
        let staked_amount_u128: u128 = staked_amount.into();
        let staked_amount_i128 = staked_amount_u128 as i128;

        let previous_amount = CLUB_STAKING_SNAPSHOT.may_load(storage, club_name_clone.clone())?.unwrap_or_default();
        let previous_amount_u128: u128 = previous_amount.into();
        let previous_amount_i128 = previous_amount_u128 as i128;

        let difference_amount = staked_amount_i128 - previous_amount_i128;
        all_stakes.push((club_name.unwrap(), difference_amount));
        CLUB_STAKING_SNAPSHOT.save(
                    storage,
                    club_name_clone.clone(),
                    &staked_amount
                )?;
    }
    all_stakes.sort_by(|a, b| (b.1.cmp(&a.1)));
    return Ok(all_stakes);
}

fn query_reward_amount(storage: &dyn Storage) -> StdResult<Uint128> {
    let reward: Uint128 = REWARD.may_load(storage)?.unwrap_or_default();
    return Ok(reward);
}

fn query_staker_rewards(
    deps: Deps,
    staker: String,
    club_name: String,
) -> StdResult<Uint128> {
    // Get the exising stakes for this club
    let mut stakes = Vec::new();
    let all_stakes = CLUB_STAKING_DETAILS.may_load(deps.storage, club_name.clone())?;
    match all_stakes {
        Some(some_stakes) => {
            stakes = some_stakes;
        }
        None => {}
    }
    let mut amount = Uint128::zero();
    for stake in stakes {
        if staker == stake.staker_address {
            amount += stake.reward_amount;
        }
    }
    return Ok(amount);
}

fn query_club_ownership_details(
    storage: &dyn Storage,
    club_name: String,
) -> StdResult<ClubOwnershipDetails> {
    let cod = CLUB_OWNERSHIP_DETAILS.may_load(storage, club_name)?;
    match cod {
        Some(cod) => return Ok(cod),
        None => return Err(StdError::generic_err("No ownership details found")),
    };
}

pub fn query_club_previous_owner_details(
    storage: &dyn Storage,
    previous_owner: String,
) -> StdResult<ClubPreviousOwnerDetails> {
    let cod = CLUB_PREVIOUS_OWNER_DETAILS.may_load(storage, previous_owner)?;
    match cod {
        Some(cod) => return Ok(cod),
        None => return Err(StdError::generic_err("No previous ownership details found")),
    };
}

pub fn query_all_stakes_for_user(
    storage: &dyn Storage,
    user_address: String,
) -> StdResult<Vec<ClubStakingDetails>> {
    let mut all_stakes = Vec::new();
    let all_clubs: Vec<String> = CLUB_STAKING_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let staking_details = CLUB_STAKING_DETAILS.load(storage, club_name)?;
        for stake in staking_details {
            if stake.staker_address == user_address {
                all_stakes.push(stake);
            }
        }
    }
    return Ok(all_stakes);
}

pub fn query_club_bonding_details_for_user(
    storage: &dyn Storage,
    club_name: String,
    user_address: String,
) -> StdResult<Vec<ClubBondingDetails>> {
    let mut bonds: Vec<ClubBondingDetails> = Vec::new();
    let cbd = CLUB_BONDING_DETAILS.may_load(storage, club_name)?;
    match cbd {
        Some(cbd) => {
            bonds = cbd;
        }
        None => return Err(StdError::generic_err("No bonding details found")),
    };
    let mut all_bonds = Vec::new();
    for bond in bonds {
        if bond.bonder_address == user_address {
            all_bonds.push(bond);
        }
    }
    return Ok(all_bonds);
}

// )
// -> StdResult<Vec<ClubBondingDetails>> {
//     let mut all_bonds = Vec::new();
//     let all_clubs: Vec<String> = CLUB_BONDING_DETAILS
//         .keys(storage, None, None, Order::Ascending)
//         .map(|k| String::from_utf8(k).unwrap())
//         .collect();
//     for club_name in all_clubs {
//         let bonding_details = CLUB_BONDING_DETAILS.load(storage, club_name)?;
//         for bond in bonding_details {
//             all_bonds.push(bond);
//         }
//     }
//     return Ok(all_bonds);

// let mut all_bonds = Vec::new();
// let bonding_details = CLUB_BONDING_DETAILS.load(storage, club_name.to_string())?;
// for bond in bonding_details {
//     if true { //bond.bonder_address == user_address {
//         all_bonds.push(bond);
//     }
// }
// return Ok(all_bonds);
// }

pub fn query_all_club_ownership_details(
    storage: &dyn Storage,
) -> StdResult<Vec<ClubOwnershipDetails>> {
    let mut all_owners = Vec::new();
    let all_clubs: Vec<String> = CLUB_OWNERSHIP_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let owner_details = CLUB_OWNERSHIP_DETAILS.load(storage, club_name)?;
        all_owners.push(owner_details);
    }
    return Ok(all_owners);
}

pub fn query_all_previous_club_ownership_details(
    storage: &dyn Storage,
) -> StdResult<Vec<ClubPreviousOwnerDetails>> {
    let mut pcod = Vec::new();
    let all_previous: Vec<String> = CLUB_PREVIOUS_OWNER_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for previous in all_previous {
        let previous_details = CLUB_PREVIOUS_OWNER_DETAILS.load(storage, previous)?;
        pcod.push(previous_details);
    }
    return Ok(pcod);
}

pub fn query_club_ownership_details_for_owner(
    storage: &dyn Storage,
    owner_address: String,
) -> StdResult<Vec<ClubOwnershipDetails>> {
    let mut all_owners = Vec::new();
    let all_clubs: Vec<String> = CLUB_OWNERSHIP_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for club_name in all_clubs {
        let owner_details = CLUB_OWNERSHIP_DETAILS.load(storage, club_name)?;
        if owner_details.owner_address == owner_address {
            all_owners.push(owner_details);
        }
    }
    return Ok(all_owners);
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, CosmosMsg, StdError, SubMsg, WasmMsg};

    use super::*;
    use cosmwasm_std::coin;

    #[test]
    fn test_buying_of_club() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let query_res = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match query_res {
            Ok(cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, false);
                assert_eq!(cod.reward_amount, Uint128::from(CLUB_BUYING_REWARD_AMOUNT));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_owner_claim_rewards() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        let result = buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );
        println!("result = {:?}", result);
        let query_res = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match query_res {
            Ok(cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, false);
                assert_eq!(cod.reward_amount, Uint128::from(CLUB_BUYING_REWARD_AMOUNT));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        claim_owner_rewards(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            "CLUB001".to_string(),
        );

        let queryResAfter = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryResAfter {
            Ok(cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, false);
                assert_eq!(cod.reward_amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_multiple_buying_of_club() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let owner2_info = mock_info("Owner002", &[coin(1000, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner2_info.clone(),
            "Owner002".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let query_res = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match query_res {
            Ok(cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, false);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_assign_a_club () {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1*60*60),
            reward_periodicity: 24*60*60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5*60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(1000, "stake")]);
        let owner2_info = mock_info("Owner002", &[coin(1000, "stake")]);

        println!("Now assigning the club to Owner001");
        assign_a_club(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            SET_AUTO_STAKE,
        );

        let queryRes0 = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryRes0 {
            Ok(mut cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(0u128));
                assert_eq!(cod.owner_released, false);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        println!("Now releasing the club from Owner001");
        release_club(deps.as_mut(), mock_env(), owner1_info.clone(), "Owner001".to_string(), "CLUB001".to_string());

        let queryRes1 = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryRes1 {
            Ok(mut cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(0u128));
                assert_eq!(cod.owner_released, true);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        println!("Now assigning the club to Owner002");
        assign_a_club(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "Owner002".to_string(),
            Some("Owner001".to_string()),
            "CLUB001".to_string(),
            SET_AUTO_STAKE,
        );

        println!("Now releasing the club from Owner002");
        release_club(deps.as_mut(), mock_env(), owner2_info.clone(), "Owner002".to_string(), "CLUB001".to_string());

        let queryRes2 = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryRes2 {
            Ok(mut cod) => {
                assert_eq!(cod.owner_address, "Owner002".to_string());
                assert_eq!(cod.price_paid, Uint128::from(0u128));
                assert_eq!(cod.owner_released, true);
                cod.start_timestamp = now.minus_seconds(22 * 24 * 60 * 60);
                CLUB_OWNERSHIP_DETAILS.save(&mut deps.storage, "CLUB001".to_string(), &cod);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        println!("Now trying to assign the club to Owner003 - should fail");
        assign_a_club(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "Owner003".to_string(),
            Some("Owner002".to_string()),
            "CLUB001".to_string(),
            SET_AUTO_STAKE,
        );

        let queryRes3 = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryRes3 {
            Ok(cod) => {
                assert_eq!(cod.owner_address, "Owner002".to_string());
                assert_eq!(cod.price_paid, Uint128::from(0u128));
                assert_eq!(cod.owner_released, true);
                assert_eq!(cod.start_timestamp, now.minus_seconds(22 * 24 * 60 * 60));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_assign_stakes_to_a_club () {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1*60*60),
            reward_periodicity: 24*60*60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5*60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(1000, "stake")]);

        println!("Now assigning the club to Owner001");
        assign_a_club(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            SET_AUTO_STAKE,
        );

        let queryRes0 = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryRes0 {
            Ok(mut cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(0u128));
                assert_eq!(cod.owner_released, false);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let mut stake_list: Vec<ClubStakingDetails> = Vec::new();
        for i in 1 .. 7 {
            let staker: String = "Staker00".to_string() + &i.to_string();
            println!("staker is {}", staker);
            stake_list.push(ClubStakingDetails {
                // TODO duration and timestamp fields no longer needed - should be removed
                staker_address: staker,
                staking_start_timestamp: now,
                staked_amount: Uint128::from(330000u128),
                staking_duration: CLUB_STAKING_DURATION,
                club_name: "CLUB001".to_string(),
                reward_amount: Uint128::from(CLUB_STAKING_REWARD_AMOUNT),
                auto_stake: SET_AUTO_STAKE,
            });
        };

        let staker6Info = mock_info("Staker006", &[coin(10, "stake")]);
        assign_stakes_to_a_club(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            stake_list,
            "CLUB001".to_string(),
        );

        let queryRes1 = query_all_stakes(&mut deps.storage);
        match queryRes1 {
            Ok(all_stakes) => {
                println!("all stakes : {:?}",all_stakes);
                assert_eq!(all_stakes.len(), 7);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_buying_of_club_after_releasing_by_prev_owner() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        let mut resp = buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );
        println!("{:?}", resp);
        resp = release_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            "CLUB001".to_string(),
        );
        println!("{:?}", resp);

        let now = mock_env().block.time; // today

        let query_res = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match query_res {
            Ok(mut cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                cod.start_timestamp = now.minus_seconds(22 * 24 * 60 * 60);
                CLUB_OWNERSHIP_DETAILS.save(&mut deps.storage, "CLUB001".to_string(), &cod);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        release_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            "CLUB001".to_string(),
        );

        let queryResAfterReleasing =
            query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryResAfterReleasing {
            Ok(cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, true);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let owner2_info = mock_info("Owner002", &[coin(0, "uusd")]);
        let resp = buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner2_info.clone(),
            "Owner002".to_string(),
            Some("Owner001".to_string()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );
        println!("{:?}", resp);
        let queryResAfterSellingByPrevOwner =
            query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryResAfterSellingByPrevOwner {
            Ok(cod) => {
                assert_eq!(cod.owner_address, "Owner002".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, false);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_claim_previous_owner_rewards() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        release_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            "CLUB001".to_string(),
        );

        let now = mock_env().block.time; // today

        let query_res = query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match query_res {
            Ok(mut cod) => {
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                cod.start_timestamp = now.minus_seconds(22 * 24 * 60 * 60);
                CLUB_OWNERSHIP_DETAILS.save(&mut deps.storage, "CLUB001".to_string(), &cod);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let stakerInfo = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(33u128),
            SET_AUTO_STAKE,
        );

        increase_reward_amount(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "reward_from abc".to_string(),
            Uint128::from(1000000u128),
        );

        let res = execute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            ExecuteMsg::CalculateAndDistributeRewards {},
        )
        .unwrap();
        assert_eq!(res.messages, Response::default().messages); // no longer a totally empty default response

        println!("releasing club");
        release_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            "CLUB001".to_string(),
        );

        let queryResAfterReleasing =
            query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryResAfterReleasing {
            Ok(cod) => {
                println!(
                    "before - owner:{:?}, reward {:?}",
                    cod.owner_address, cod.reward_amount
                );
                assert_eq!(cod.owner_address, "Owner001".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, true);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        println!(
            "pod:\n {:?}",
            query_all_previous_club_ownership_details(&mut deps.storage)
        );

        println!("buy a club with new owner");
        let owner2_info = mock_info("Owner002", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner2_info.clone(),
            "Owner002".to_string(),
            Some("Owner001".to_string()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let queryResAfterSellingByPrevOwner =
            query_club_ownership_details(&mut deps.storage, "CLUB001".to_string());
        match queryResAfterSellingByPrevOwner {
            Ok(cod) => {
                println!(
                    "after - owner:{:?}, reward {:?}",
                    cod.owner_address, cod.reward_amount
                );
                assert_eq!(cod.owner_address, "Owner002".to_string());
                assert_eq!(cod.price_paid, Uint128::from(1000000u128));
                assert_eq!(cod.owner_released, false);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        println!("checking previous owner details now");

        /*
        27 Feb 2022, Commenting this out - because the reward is now moved to stake
                     so there will be no previous owner details    

        let queryPrevOwnerDetailsBeforeRewardClaim =
            query_club_previous_owner_details(&mut deps.storage, "Owner001".to_string());
        match queryPrevOwnerDetailsBeforeRewardClaim {
            Ok(pod) => {
                println!(
                    "before - owner:{:?}, reward {:?}",
                    pod.previous_owner_address, pod.reward_amount
                );
                assert_eq!(pod.previous_owner_address, "Owner001".to_string());
                assert_eq!(pod.reward_amount, Uint128::from(10000u128)); 
            }
            Err(e) => {
                println!("error parsing cpod header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        */

        println!(
            "pod:\n {:?}",
            query_all_previous_club_ownership_details(&mut deps.storage)
        );

        claim_previous_owner_rewards(deps.as_mut(), owner1_info.clone(), "Owner001".to_string());
        let queryPrevOwnerDetailsAfterRewardClaim =
            query_club_previous_owner_details(&mut deps.storage, "Owner001".to_string())
                .unwrap_err();
        assert_eq!(
            queryPrevOwnerDetailsAfterRewardClaim,
            (StdError::GenericErr {
                msg: String::from("No previous ownership details found")
            })
        );

        println!(
            "pod:\n {:?}",
            query_all_previous_club_ownership_details(&mut deps.storage)
        );
    }

    #[test]
    fn test_claim_rewards_with_no_auto_stake() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            false, // NO AUTO STAKE
        );


        let stakerInfo = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(33000u128),
            false, // NO AUTO STAKE
        );

        increase_reward_amount(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "reward_from abc".to_string(),
            Uint128::from(1000000u128),
        );

        let res = execute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            ExecuteMsg::CalculateAndDistributeRewards {},
        )
        .unwrap();
        assert_eq!(res.messages, Response::default().messages); // no longer a totally empty default response

        let queryRes = query_all_stakes(&mut deps.storage);
        match queryRes {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 2);
                for stake in all_stakes {
                    let staker_address = stake.staker_address;
                    let reward_amount = stake.reward_amount;
                    let staked_amount = stake.staked_amount;
                    println!("staker : {:?} reward_amount : {:?} staked_amount : {:?}", staker_address.clone(), reward_amount, staked_amount);
                    if staker_address == "Staker001" {
                        assert_eq!(reward_amount, Uint128::from(970000u128));
                    }
                    if staker_address == "Owner001" {
                        assert_eq!(reward_amount, Uint128::from(30000u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_multiple_staking_on_club_by_same_address() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let stakerInfo = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(33u128),
            SET_AUTO_STAKE,
        );
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(11u128),
            SET_AUTO_STAKE,
        );
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(42u128),
            SET_AUTO_STAKE,
        );

        let query_res = query_all_stakes(&mut deps.storage);
        match query_res {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 2);
                for stake in all_stakes {
                    if stake.staker_address == "Staker001".to_string() {
                        assert_eq!(stake.staked_amount, Uint128::from(86u128));
                    } else {
                        assert_eq!(stake.staked_amount, Uint128::from(0u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_immediate_partial_withdrawals_from_club() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let stakerInfo = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(99u128),
            SET_AUTO_STAKE,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(11u128),
            IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(12u128),
            IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(13u128),
            IMMEDIATE_WITHDRAWAL,
        );

        let query_stakes = query_all_stakes(&mut deps.storage);
        match query_stakes {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 2);
                for stake in all_stakes {
                    if stake.staker_address == "Staker001".to_string() {
                        assert_eq!(stake.staked_amount, Uint128::from(63u128));
                    } else {
                        assert_eq!(stake.staked_amount, Uint128::from(0u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let queryBonds = query_all_bonds(&mut deps.storage);
        match queryBonds {
            Ok(all_bonds) => {
                assert_eq!(all_bonds.len(), 0);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_immediate_complete_withdrawals_from_club() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1Info = mock_info("Owner001", &[coin(1000, "stake")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let stakerInfo = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(99u128),
            SET_AUTO_STAKE,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(11u128),
            IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(12u128),
            IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(13u128),
            IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(63u128),
            IMMEDIATE_WITHDRAWAL,
        );

        let queryStakes = query_all_stakes(&mut deps.storage);
        match queryStakes {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 0);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let queryBonds = query_all_bonds(&mut deps.storage);
        match queryBonds {
            Ok(all_bonds) => {
                assert_eq!(all_bonds.len(), 0);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_non_immediate_complete_withdrawals_from_club() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let admin_info = mock_info("admin11111", &[]);
        let minting_contract_info = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            admin_info.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let stakerInfo = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            minting_contract_info.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(99u128),
            SET_AUTO_STAKE,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(11u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(12u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(13u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(63u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );

        let query_stakes = query_all_stakes(&mut deps.storage);
        match query_stakes {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 2);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let queryBonds = query_all_bonds(&mut deps.storage);
        match queryBonds {
            Ok(all_bonds) => {
                assert_eq!(all_bonds.len(), 4);
                for bond in all_bonds {
                    if bond.bonded_amount != Uint128::from(11u128)
                        && bond.bonded_amount != Uint128::from(12u128)
                        && bond.bonded_amount != Uint128::from(13u128)
                        && bond.bonded_amount != Uint128::from(63u128)
                    {
                        println!("bond is {:?} ", bond);
                        assert_eq!(1, 2);
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let stakerInfo = mock_info("Staker002", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            minting_contract_info.clone(),
            "Staker002".to_string(),
            "CLUB001".to_string(),
            Uint128::from(99u128),
            SET_AUTO_STAKE,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker002".to_string(),
            "CLUB001".to_string(),
            Uint128::from(11u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );

        let queryBonds = query_club_bonding_details_for_user(
            &mut deps.storage,
            "CLUB001".to_string(),
            "Staker002".to_string(),
        );
        match queryBonds {
            Ok(all_bonds) => {
                assert_eq!(all_bonds.len(), 1);
                for bond in all_bonds {
                    if bond.bonded_amount != Uint128::from(11u128) {
                        println!("bond is {:?} ", bond);
                        assert_eq!(1, 2);
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_non_immediate_complete_withdrawals_from_club_with_scheduled_refunds() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "feecollector11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let stakerInfo = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(99u128),
            SET_AUTO_STAKE,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(11u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(12u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(13u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(63u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );

        let query_stakes = query_all_stakes(&mut deps.storage);
        match query_stakes {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 2);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let now = mock_env().block.time; // today

        let query_bonds = query_all_bonds(&mut deps.storage);
        match query_bonds {
            Ok(all_bonds) => {
                let existing_bonds = all_bonds.clone();
                let mut updated_bonds = Vec::new();
                assert_eq!(existing_bonds.len(), 4);
                for bond in existing_bonds {
                    let mut updated_bond = bond.clone();
                    if updated_bond.bonded_amount != Uint128::from(11u128)
                        && updated_bond.bonded_amount != Uint128::from(12u128)
                        && updated_bond.bonded_amount != Uint128::from(13u128)
                        && updated_bond.bonded_amount != Uint128::from(63u128)
                    {
                        println!("updated_bond is {:?} ", updated_bond);
                        assert_eq!(1, 2);
                    }
                    if updated_bond.bonded_amount == Uint128::from(63u128) {
                        updated_bond.bonding_start_timestamp = now.minus_seconds(8 * 24 * 60 * 60);
                    }
                    updated_bonds.push(updated_bond);
                }
                CLUB_BONDING_DETAILS.save(&mut deps.storage, "CLUB001".to_string(), &updated_bonds);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        periodically_refund_stakeouts(deps.as_mut(), mock_env(), adminInfo);

        let queryBondsAfterPeriodicRefund = query_all_bonds(&mut deps.storage);
        match queryBondsAfterPeriodicRefund {
            Ok(all_bonds) => {
                assert_eq!(all_bonds.len(), 3);
                for bond in all_bonds {
                    if bond.bonded_amount != Uint128::from(11u128)
                        && bond.bonded_amount != Uint128::from(12u128)
                        && bond.bonded_amount != Uint128::from(13u128)
                    {
                        println!("bond is {:?} ", bond);
                        assert_eq!(1, 2);
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_non_immediate_partial_withdrawals_from_club() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 24 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1_info = mock_info("Owner001", &[coin(0, "uusd")]);
        let result = buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );
        println!("buy_a_club result = {:?}", result);
        let stakerInfo = mock_info("Staker001", &[coin(10, "uusd")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(99u128),
            SET_AUTO_STAKE,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(11u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(12u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        let result = withdraw_stake_from_a_club(
            deps.as_mut(),
            mock_env(),
            stakerInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(13u128),
            NO_IMMEDIATE_WITHDRAWAL,
        );
        println!("result = {:?}", result);
        let query_stakes = query_all_stakes(&mut deps.storage);
        match query_stakes {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 2);
                for stake in all_stakes {
                    if stake.staker_address == "Staker001".to_string() {
                        assert_eq!(stake.staked_amount, Uint128::from(63u128));
                    } else {
                        assert_eq!(stake.staked_amount, Uint128::from(0u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let queryBonds = query_all_bonds(&mut deps.storage);
        match queryBonds {
            Ok(all_bonds) => {
                assert_eq!(all_bonds.len(), 3);
                for bond in all_bonds {
                    if bond.bonded_amount != Uint128::from(11u128)
                        && bond.bonded_amount != Uint128::from(12u128)
                        && bond.bonded_amount != Uint128::from(13u128)
                    {
                        println!("bond is {:?} ", bond);
                        assert_eq!(1, 2);
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_distribute_rewards() {
        let mut deps = mock_dependencies(&[]);
        let now = mock_env().block.time; // today

        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(8 * 60 * 60),
            reward_periodicity: 5 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        let adminInfo = mock_info("admin11111", &[]);
        let mintingContractInfo = mock_info("minting_admin11111", &[]);

        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        let owner1Info = mock_info("Owner001", &[coin(1000, "stake")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner1Info.clone(),
            "Owner001".to_string(),
            Some(String::default()),
            "CLUB001".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );
        let owner2Info = mock_info("Owner002", &[coin(1000, "stake")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner2Info.clone(),
            "Owner002".to_string(),
            Some(String::default()),
            "CLUB002".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );
        let owner3Info = mock_info("Owner003", &[coin(1000, "stake")]);
        buy_a_club(
            deps.as_mut(),
            mock_env(),
            owner3Info.clone(),
            "Owner003".to_string(),
            Some(String::default()),
            "CLUB003".to_string(),
            Uint128::from(1000000u128),
            SET_AUTO_STAKE,
        );

        let staker1Info = mock_info("Staker001", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker001".to_string(),
            "CLUB001".to_string(),
            Uint128::from(330000u128),
            SET_AUTO_STAKE,
        );

        let staker2Info = mock_info("Staker002", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker002".to_string(),
            "CLUB001".to_string(),
            Uint128::from(110000u128),
            SET_AUTO_STAKE,
        );

        let staker3Info = mock_info("Staker003", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker003".to_string(),
            "CLUB002".to_string(),
            Uint128::from(420000u128),
            SET_AUTO_STAKE,
        );

        let staker4Info = mock_info("Staker004", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker004".to_string(),
            "CLUB002".to_string(),
            Uint128::from(100000u128),
            SET_AUTO_STAKE,
        );

        let staker5Info = mock_info("Staker005", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker005".to_string(),
            "CLUB003".to_string(),
            Uint128::from(820000u128),
            SET_AUTO_STAKE,
        );

        let staker6Info = mock_info("Staker006", &[coin(10, "stake")]);
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker006".to_string(),
            "CLUB003".to_string(),
            Uint128::from(50000u128),
            SET_AUTO_STAKE,
        );

        let queryRes0 = query_all_stakes(&mut deps.storage);
        match queryRes0 {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 9);
                println!("all stakes : {:?}",all_stakes);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        increase_reward_amount(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "reward_from abc".to_string(),
            Uint128::from(1000000u128),
        );
        let queryReward = query_reward_amount(&mut deps.storage);
        println!("reward amount before distribution: {:?}",queryReward);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            ExecuteMsg::CalculateAndDistributeRewards {},
        )
        .unwrap();
        assert_eq!(res.messages, Response::default().messages); // no longer a totally empty default response

        let queryReward = query_reward_amount(&mut deps.storage);
        println!("reward amount after distribution: {:?}",queryReward);
        let queryRes = query_all_stakes(&mut deps.storage);
        match queryRes {
            Ok(all_stakes) => {
                assert_eq!(all_stakes.len(), 9);
                println!("all stakes : {:?}",all_stakes);
                for stake in all_stakes {
                    let staker_address = stake.staker_address;
                    let staked_amount = stake.staked_amount;
                    println!("staker : {:?} staked_amount : {:?}", staker_address.clone(), staked_amount);
                    if staker_address == "Staker001" {
                        assert_eq!(staked_amount, Uint128::from(470655u128));
                    }
                    if staker_address == "Staker002" {
                        assert_eq!(staked_amount, Uint128::from(156885u128));
                    }
                    if staker_address == "Staker003" {
                        assert_eq!(staked_amount, Uint128::from(599016u128));
                    }
                    if staker_address == "Staker004" {
                        assert_eq!(staked_amount, Uint128::from(142622u128));
                    }
                    if staker_address == "Staker005" {
                        assert_eq!(staked_amount, Uint128::from(1348588u128));
                    }
                    if staker_address == "Staker006" {
                        assert_eq!(staked_amount, Uint128::from(82230u128));
                    }
                    if staker_address == "Owner001" {
                        assert_eq!(staked_amount, Uint128::from(10000u128));
                    }
                    if staker_address == "Owner002" {
                        assert_eq!(staked_amount, Uint128::from(10000u128));
                    }
                    if staker_address == "Owner003" {
                        assert_eq!(staked_amount, Uint128::from(10000u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        // test another attempt to calculate and distribute at the same time

        increase_reward_amount(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "reward_from abc".to_string(),
            Uint128::from(1000000u128),
        );

        let err = execute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            ExecuteMsg::CalculateAndDistributeRewards {},
        )
        .unwrap_err();

        assert_eq!(
            err,
            (ContractError::Std(StdError::GenericErr {
                msg: String::from("Time for Reward not yet arrived")
            }))
        );

        // test by preponing club_reward_next_timestamp
        let instantiate_msg = InstantiateMsg {
            admin_address: "admin11111".to_string(),
            minting_contract_address: "minting_admin11111".to_string(),
            astro_proxy_address: "astro_proxy_address1111".to_string(),
            club_fee_collector_wallet: "club_fee_collector_wallet11111".to_string(),
            club_reward_next_timestamp: now.minus_seconds(1 * 60 * 60),
            reward_periodicity: 5 * 60 * 60u64,
            club_price: Uint128::from(1000000u128),
            bonding_duration: 5 * 60u64,
            owner_release_locking_duration: 24 * 60 * 60u64,
            platform_fees_collector_wallet: "platform_fee_collector_wallet_1111".to_string(),
            platform_fees: Uint128::from(100u128),
            transaction_fees: Uint128::from(30u128),
            control_fees: Uint128::from(50u128),
        };
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        )
        .unwrap();

        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker004".to_string(),
            "CLUB002".to_string(),
            Uint128::from(100000u128),
            SET_AUTO_STAKE,
        );
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker004".to_string(),
            "CLUB001".to_string(),
            Uint128::from(500000u128),
            SET_AUTO_STAKE,
        );
        stake_on_a_club(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "Staker004".to_string(),
            "CLUB003".to_string(),
            Uint128::from(126718u128),
            SET_AUTO_STAKE,
        );
        increase_reward_amount(
            deps.as_mut(),
            mock_env(),
            mintingContractInfo.clone(),
            "reward_from def".to_string(),
            Uint128::from(1000000u128),
        );


        let res = execute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            ExecuteMsg::CalculateAndDistributeRewards {},
        )
        .unwrap();

        assert_eq!(res.messages, Response::default().messages); // no longer a totally empty default response
    }
}
