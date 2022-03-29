#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
    Storage, Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw20::{
    AllowanceResponse, BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration,
};

use crate::allowances::{
    deduct_allowance, execute_burn_from, execute_decrease_allowance, execute_increase_allowance,
    execute_send_from, execute_transfer_from, query_allowance,
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, CONFIG,
    LunaUserDetails, LUNA_USER_DETAILS, 
    ActivityDetails, ACTIVITY_DETAILS, 
    UserActivityDetails, USER_ACTIVITY_DETAILS, 
    AIRDROP_CONTRACT_WALLET, 
    CONTRACT_LOCK_STATUS,
    UserRewardInfo,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:crll-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAIN_WALLET: &str = "terra1t3czdl5h4w4qwgkzs80fdstj0z7rfv9v2j6uh3";

// Note that Luna activity is just a placeholder - so 3 activities
const NUM_OF_USER_ACTIVITIES: usize = 4; 
const LUNA_ACTIVITY: &str = "LUNA_ACTIVITY";
const GAMING_ACTIVITY: &str = "GAMING_ACTIVITY";
const STAKING_ACTIVITY: &str = "STAKING_ACTIVITY";
const LIQUIDITY_ACTIVITY: &str = "LIQUIDITY_ACTIVITY";

const QUALIFIED_FOR_REWARD: bool = true;
const NOT_QUALIFIED_FOR_REWARD: bool = false;

const LOCKED: u128 = 1u128;
const UNLOCKED: u128 = 0u128;

/*
Flow of contract
----------------
instantiate
lock
set_activity_reward_amount (activity_name, amount) 
clear_qualified_flag - for luna users and their gaming activities
update_luna_user_list_detail 
  -- this will update reward for each luna user
  -- this will also update reward for staking activity for each user, if luna is qualified for that user
unlock

After unlock, user will randomly call
update_user_activity(user_name, activity_name, activity_qualified)
claim_user_rewards (user_name) 
  -- only unclaimed reward will be claimed. This will be both luna and all activity rewards
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        cw20_token_address: deps.api.addr_validate(&msg.cw20_token_address)?,
        admin_address: deps.api.addr_validate(&msg.admin_address)?,
    };
    CONFIG.save(deps.storage, &config)?;

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
        ExecuteMsg::UpdateUserRewardAmount {
            activity_name,
            user_reward_list,
        } => {
            update_activity_reward_for_users (deps, env, info, activity_name, user_reward_list)
        }
        ExecuteMsg::ClaimUserRewards { 
            user_name, 
        } => {
            claim_user_rewards (deps, env, info, user_name)
        }
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => execute_send_from(deps, env, info, owner, contract, amount, msg),
    }
}


fn get_contract_lock_status (
    deps: DepsMut,
) -> StdResult<Uint128> {
    let distribute_from = String::from(MAIN_WALLET);
    let address = deps.api.addr_validate(distribute_from.clone().as_str())?;
    let cls = CONTRACT_LOCK_STATUS.may_load(deps.storage, &address)?;
    match cls {
        Some(cls) => return Ok(cls),
        None => return Err(StdError::generic_err("No lock status found")),
    };
}


fn set_contract_lock_status (
    deps: DepsMut,
    lock_status: Uint128,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    if lock_status != Uint128::from(LOCKED) && lock_status != Uint128::from(UNLOCKED) {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Invalid argument"),
        }));
    }

    let distribute_from = String::from(MAIN_WALLET);
    let address = deps.api.addr_validate(distribute_from.clone().as_str())?;
    if lock_status == Uint128::from(LOCKED) {
        CONTRACT_LOCK_STATUS.update(
            deps.storage,
            &address,
            |lock_status: Option<Uint128>| -> StdResult<_> { Ok(Uint128::from(LOCKED)) },
        )?;
    } else {
        CONTRACT_LOCK_STATUS.update(
            deps.storage,
            &address,
            |lock_status: Option<Uint128>| -> StdResult<_> { Ok(Uint128::from(UNLOCKED)) },
        )?;
    }

    return Ok(Response::default());
}

fn claim_user_rewards (
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_name: String,
) -> Result<Response, ContractError> {
/*
    Nov 30, 2021: LOCKING is no longer needed - so commenting this out

    // it needs to get initialized to unlock
    let distribute_from = String::from(MAIN_WALLET);
    let address = deps.api.addr_validate(distribute_from.clone().as_str())?;
    let cls = CONTRACT_LOCK_STATUS.may_load(deps.storage, &address)?;
    match cls {
        Some(cls) => {
            if cls == Uint128::from(LOCKED) {    
                return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("Contract is locked"),
                }));
            }
        }
        None => {
                return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("Cant get Contract lock status"),
                }));
        }
    }
*/

    let user_addr = deps.api.addr_validate(&user_name)?;
    //Check if withdrawer is same as invoker
    if user_addr != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut total_amount = Uint128::zero();

/*
    Nov 30, 2021: No longer needed as LUNA is also an activity now - so commenting this out
    
    let res = query_luna_user_details(deps.storage, user_name.clone());
    match res {
        Ok(user) => {
            let mut modified_user_details = user.clone();
            if user.luna_airdrop_reward_amount > Uint128::zero() {
                total_amount += user.luna_airdrop_reward_amount;
                modified_user_details.luna_airdrop_reward_amount = Uint128::zero();
                LUNA_USER_DETAILS.save(deps.storage, user_name.clone(), &modified_user_details)?;
            }
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("No Luna user found"),
            }));
        }
    }
*/

    // Get the existing rewards for this user activities
    let mut activities = Vec::new();
    let all_activities = USER_ACTIVITY_DETAILS.may_load(deps.storage, user_name.clone())?;
    match all_activities {
        Some(some_activities) => {
            activities = some_activities;
        }
        None => {}
    }

    let existing_activities = activities.clone();
    let mut updated_activities = Vec::new();
    for activity in existing_activities {
        let mut updated_activity = activity.clone();
        if activity.activity_reward_amount_accrued > Uint128::zero() {
            total_amount += activity.activity_reward_amount_accrued;
            updated_activity.activity_reward_amount_accrued = Uint128::zero();
        }
        updated_activities.push(updated_activity);
    }
    USER_ACTIVITY_DETAILS.save(deps.storage, user_name, &updated_activities)?;

    // TODO: transfer total amount to user wallet

    return Ok(Response::new().add_attribute("reward", total_amount));
}

fn create_luna_user_details(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_name: String,
    luna_airdrop_qualified: bool,
    luna_airdrop_reward_amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    let luna_details;
    let luna_details_result = LUNA_USER_DETAILS.may_load(deps.storage, user_name.clone());
    match luna_details_result {
        Ok(od) => {
            luna_details = od;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::from(e)));
        }
    }

    // Now save the luna user details
    LUNA_USER_DETAILS.save(
        deps.storage,
        user_name.clone(),
        &LunaUserDetails {
            user_name: user_name.clone(),
            luna_airdrop_qualified: luna_airdrop_qualified,
            luna_airdrop_reward_amount: Uint128::zero(),
        },
    )?;

    // also create activities : gaming, staking, liquidity
    let mut activities = Vec::new();
    activities.push(UserActivityDetails {
        user_name: user_name.clone(),
        activity_name: LUNA_ACTIVITY.to_string(),
        activity_qualified: luna_airdrop_qualified,
        activity_reward_amount_accrued: Uint128::zero(),
    });
    activities.push(UserActivityDetails {
        user_name: user_name.clone(),
        activity_name: GAMING_ACTIVITY.to_string(),
        activity_qualified: NOT_QUALIFIED_FOR_REWARD,
        activity_reward_amount_accrued: Uint128::zero(),
    });
    activities.push(UserActivityDetails {
        user_name: user_name.clone(),
        activity_name: STAKING_ACTIVITY.to_string(),
        activity_qualified: NOT_QUALIFIED_FOR_REWARD,
        activity_reward_amount_accrued: Uint128::zero(),
    });
    activities.push(UserActivityDetails {
        user_name: user_name.clone(),
        activity_name: LIQUIDITY_ACTIVITY.to_string(),
        activity_qualified: NOT_QUALIFIED_FOR_REWARD,
        activity_reward_amount_accrued: Uint128::zero(),
    });
    USER_ACTIVITY_DETAILS.save(deps.storage, user_name, &activities)?;

    return Ok(Response::default());
}

fn update_luna_user_list_details(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    count: usize,
    user_name_list: Vec<String>,
    luna_airdrop_qualified_list: Vec<bool>,
    luna_airdrop_reward_amount_list: Vec<Uint128>,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    // TODO: Lot of code duplication, how to call the fn update_luna_user_details
    // it gives issues with deps getting moved and you cant clone it also ??

    for i in 0..count {
        let user_name = user_name_list[i].clone();
        let luna_airdrop_qualified = luna_airdrop_qualified_list[i];
        let luna_airdrop_reward_amount = luna_airdrop_reward_amount_list[i];

        println!("I {:?} username {:?} qualified {:?} reward {:?}",
            i, user_name.clone(), luna_airdrop_qualified, luna_airdrop_reward_amount);    

        let res = query_luna_user_details(deps.storage, user_name.clone());
        match res {
            Ok(user) => {
                let mut modified_user_details = user.clone();
                modified_user_details.luna_airdrop_qualified = luna_airdrop_qualified;
                modified_user_details.luna_airdrop_reward_amount = luna_airdrop_reward_amount;
                LUNA_USER_DETAILS.save(deps.storage, user_name.clone(), &modified_user_details)?;
            }
            Err(e) => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("No Luna user found"),
                }));
            }
        }

        // if setting luna as qualified, also update staking rewards
        if luna_airdrop_qualified == QUALIFIED_FOR_REWARD {
            let mut activity_eligible_reward_amount = Uint128::zero();
            let mut activities = Vec::new();
            let all_activities = ACTIVITY_DETAILS.may_load(deps.storage, STAKING_ACTIVITY.to_string())?;
            match all_activities {
                Some(some_activities) => {
                    activities = some_activities;
                }
                None => {}
            }
            let mut activity_eligible_reward_amount = Uint128::zero();
            for activity in activities {
                if activity.activity_name == STAKING_ACTIVITY.to_string() {
                    activity_eligible_reward_amount = activity.eligible_activity_reward_amount;
                }
            }
            let mut user_activities = Vec::new();
            let all_user_activities = USER_ACTIVITY_DETAILS.may_load(deps.storage, user_name.clone())?;
            match all_user_activities {
                Some(some_user_activities) => {
                    user_activities = some_user_activities;
                }
                None => {}
            }
            let existing_user_activities = user_activities.clone();
            let mut updated_user_activities = Vec::new();
            for user_activity in existing_user_activities {
                let mut updated_user_activity = user_activity.clone();
                if user_activity.activity_name == STAKING_ACTIVITY.to_string() {
                    if user_activity.activity_qualified == QUALIFIED_FOR_REWARD {
                        updated_user_activity.activity_reward_amount_accrued += activity_eligible_reward_amount;
                    }
                }
                updated_user_activities.push(updated_user_activity);
            }
            USER_ACTIVITY_DETAILS.save(deps.storage, user_name, &updated_user_activities)?;
        }
    }
    return Ok(Response::default());
}

fn update_activity_reward_for_users (
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    activity_name: String,
    user_reward_list: Vec<UserRewardInfo>,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    for user_reward in user_reward_list {
        let user_name = user_reward.user_name;
        let reward_amount = user_reward.reward_amount;

        println!("username {:?} activity {:?} reward {:?}", user_name.clone(), activity_name, reward_amount);    

        let res = query_luna_user_details(deps.storage, user_name.clone());
        match res {
            Ok(user) => {
                // User is already created - do nothing
            }
            Err(e) => {
                // Create the user 
                LUNA_USER_DETAILS.save(
                    deps.storage,
                    user_name.clone(),
                    &LunaUserDetails {
                        user_name: user_name.clone(),
                        luna_airdrop_qualified: QUALIFIED_FOR_REWARD,
                        luna_airdrop_reward_amount: Uint128::zero(),
                    },
                )?;
                // also create activities : luna, gaming, staking, liquidity
                let mut activities = Vec::new();
                activities.push(UserActivityDetails {
                    user_name: user_name.clone(),
                    activity_name: LUNA_ACTIVITY.to_string(),
                    activity_qualified: NOT_QUALIFIED_FOR_REWARD,
                    activity_reward_amount_accrued: Uint128::zero(),
                });
                activities.push(UserActivityDetails {
                    user_name: user_name.clone(),
                    activity_name: GAMING_ACTIVITY.to_string(),
                    activity_qualified: NOT_QUALIFIED_FOR_REWARD,
                    activity_reward_amount_accrued: Uint128::zero(),
                });
                activities.push(UserActivityDetails {
                    user_name: user_name.clone(),
                    activity_name: STAKING_ACTIVITY.to_string(),
                    activity_qualified: NOT_QUALIFIED_FOR_REWARD,
                    activity_reward_amount_accrued: Uint128::zero(),
                });
                activities.push(UserActivityDetails {
                    user_name: user_name.clone(),
                    activity_name: LIQUIDITY_ACTIVITY.to_string(),
                    activity_qualified: NOT_QUALIFIED_FOR_REWARD,
                    activity_reward_amount_accrued: Uint128::zero(),
                });
                USER_ACTIVITY_DETAILS.save(deps.storage, user_name.clone(), &activities)?;
            }
        }

        let mut user_activities = Vec::new();
        let all_user_activities = USER_ACTIVITY_DETAILS.may_load(deps.storage, user_name.clone())?;
        match all_user_activities {
            Some(some_user_activities) => {
                user_activities = some_user_activities;
            }
            None => {}
        }
        let existing_user_activities = user_activities.clone();
        let mut updated_user_activities = Vec::new();
        for user_activity in existing_user_activities {
            let mut updated_user_activity = user_activity.clone();
            if user_activity.activity_name == activity_name {
                updated_user_activity.activity_reward_amount_accrued += reward_amount;
            }
            updated_user_activities.push(updated_user_activity);
        }
        USER_ACTIVITY_DETAILS.save(deps.storage, user_name, &updated_user_activities)?;
    }
    return Ok(Response::default());
}

fn update_luna_user_details(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_name: String,
    luna_airdrop_qualified: bool,
    luna_airdrop_reward_amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    let res = query_luna_user_details(deps.storage, user_name.clone());
    match res {
        Ok(user) => {
            let mut modified_user_details = user.clone();
            modified_user_details.luna_airdrop_qualified = luna_airdrop_qualified;
            modified_user_details.luna_airdrop_reward_amount = luna_airdrop_reward_amount;
            LUNA_USER_DETAILS.save(deps.storage, user_name.clone(), &modified_user_details)?;
        }
        Err(e) => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("No Luna user found"),
            }));
        }
    }

    // if setting luna as qualified, also update staking rewards
    if luna_airdrop_qualified == QUALIFIED_FOR_REWARD {
        let mut activity_eligible_reward_amount = Uint128::zero();
        let mut activities = Vec::new();
        let all_activities = ACTIVITY_DETAILS.may_load(deps.storage, STAKING_ACTIVITY.to_string())?;
        match all_activities {
            Some(some_activities) => {
                activities = some_activities;
            }
            None => {}
        }
        let mut activity_eligible_reward_amount = Uint128::zero();
        for activity in activities {
            if activity.activity_name == STAKING_ACTIVITY.to_string() {
                activity_eligible_reward_amount = activity.eligible_activity_reward_amount;
            }
        }
        let mut user_activities = Vec::new();
        let all_user_activities = USER_ACTIVITY_DETAILS.may_load(deps.storage, user_name.clone())?;
        match all_user_activities {
            Some(some_user_activities) => {
                user_activities = some_user_activities;
            }
            None => {}
        }
        let existing_user_activities = user_activities.clone();
        let mut updated_user_activities = Vec::new();
        for user_activity in existing_user_activities {
            let mut updated_user_activity = user_activity.clone();
            if user_activity.activity_name == STAKING_ACTIVITY.to_string() {
                if user_activity.activity_qualified == QUALIFIED_FOR_REWARD {
                    updated_user_activity.activity_reward_amount_accrued += activity_eligible_reward_amount;
                }
            }
            updated_user_activities.push(updated_user_activity);
        }
        USER_ACTIVITY_DETAILS.save(deps.storage, user_name, &updated_user_activities)?;
    }

    return Ok(Response::default());
}

fn create_activity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    activity_name: String,
    prereq_activity_name: String,
    eligible_activity_reward_amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    let mut activities = Vec::new();
    let all_activities = ACTIVITY_DETAILS.may_load(deps.storage, activity_name.clone())?;
    match all_activities {
        Some(some_activities) => {
            activities = some_activities;
        }
        None => {}
    }

    activities.push(ActivityDetails {
        activity_name: activity_name.clone(),
        prereq_activity_name: prereq_activity_name.clone(),
        eligible_activity_reward_amount: eligible_activity_reward_amount,
    });
    ACTIVITY_DETAILS.save(deps.storage, activity_name, &activities)?;

    return Ok(Response::default());
}

fn update_activity_eligibility_reward_amount(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    activity_name: String,
    eligible_activity_reward_amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    let mut activities = Vec::new();
    let all_activities = ACTIVITY_DETAILS.may_load(deps.storage, activity_name.clone())?;
    match all_activities {
        Some(some_activities) => {
            activities = some_activities;
        }
        None => {}
    }

    let existing_activities = activities.clone();
    let mut updated_activities = Vec::new();
    for activity in existing_activities {
        let mut updated_activity = activity.clone();
        if activity.activity_name == activity_name {
            updated_activity.eligible_activity_reward_amount = eligible_activity_reward_amount;
        }
        updated_activities.push(updated_activity);
    }
    ACTIVITY_DETAILS.save(deps.storage, activity_name, &updated_activities)?;

    return Ok(Response::default());
}

/*
call this function to accrue amount if the activity is qualified. Using an event log
Once every week reset the activity_qualified status to NOT_QUALIFIED_FOR_REWARD 
*/
fn update_user_activity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_name: String,
    activity_name: String,
    activity_qualified: bool,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    // Ensure that the contract is not locked for batch processing
    // it needs to get initialized
    let distribute_from = String::from(MAIN_WALLET);
    let address = deps.api.addr_validate(distribute_from.clone().as_str())?;
    let cls = CONTRACT_LOCK_STATUS.may_load(deps.storage, &address)?;
    match cls {
        Some(cls) => {
            if cls == Uint128::from(LOCKED) {    
                return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("Contract is locked"),
                }));
            }
        }
        None => {
                return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("Cant get Contract lock status"),
                }));
        }
    }


    // If you are setting the qualified flag for the activity, ensure 
    // first that the luna qualification is complete
    if activity_qualified == QUALIFIED_FOR_REWARD {
        let res = query_luna_user_details(deps.storage, user_name.clone());
        match res {
            Ok(user) => {
                if user.luna_airdrop_qualified != QUALIFIED_FOR_REWARD {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: String::from("Luna is not qualified"),
                    }));
                }
            }
            Err(e) => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("No Luna user found"),
                }));
            }
        }
    }


    let mut activities = Vec::new();
    let all_activities = ACTIVITY_DETAILS.may_load(deps.storage, activity_name.clone())?;
    match all_activities {
        Some(some_activities) => {
            activities = some_activities;
        }
        None => {}
    }
    let mut activity_eligible_reward_amount = Uint128::zero();
    for activity in activities {
        if activity.activity_name == activity_name {
            activity_eligible_reward_amount = activity.eligible_activity_reward_amount;
        }
    }

    let mut user_activities = Vec::new();
    let all_user_activities = USER_ACTIVITY_DETAILS.may_load(deps.storage, user_name.clone())?;
    match all_user_activities {
        Some(some_user_activities) => {
            user_activities = some_user_activities;
        }
        None => {}
    }

    let existing_user_activities = user_activities.clone();
    let mut updated_user_activities = Vec::new();
    for user_activity in existing_user_activities {
        let mut updated_user_activity = user_activity.clone();
        if user_activity.activity_name == activity_name {
            if activity_qualified == QUALIFIED_FOR_REWARD && user_activity.activity_qualified == NOT_QUALIFIED_FOR_REWARD {
                updated_user_activity.activity_reward_amount_accrued += activity_eligible_reward_amount;
            }
            updated_user_activity.activity_qualified = activity_qualified;
        }
        updated_user_activities.push(updated_user_activity);
    }
    USER_ACTIVITY_DETAILS.save(deps.storage, user_name, &updated_user_activities)?;

    return Ok(Response::default());
}


fn clear_qualified_flag_for_all_luna_users_and_non_exempt_activities(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // TODO: Add some authentication check here

    // TODO: Implement processing in batches 
    // Current implementation Worked up to 1 million users
    // Machine configuration Ubuntu 20, i7 processor, 32GB RAM, 1 TB SSD

    let all_users_in_activities: Vec<String> = USER_ACTIVITY_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for user_name in all_users_in_activities {
        let mut updated_user_activities = Vec::new();
        let existing_user_activity_details = USER_ACTIVITY_DETAILS.load(deps.storage, user_name.clone())?;
        for existing_user_activity in existing_user_activity_details {
            let mut updated_user_activity = existing_user_activity.clone();
            if existing_user_activity.activity_name != STAKING_ACTIVITY.to_string() {
                updated_user_activity.activity_qualified = NOT_QUALIFIED_FOR_REWARD;
            }
            updated_user_activities.push(updated_user_activity);
        }
        USER_ACTIVITY_DETAILS.save(deps.storage, user_name, &updated_user_activities)?;
    }


    // clear the qualified flag for all users
    let all_users: Vec<String> = LUNA_USER_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for user_name in all_users {
        let mut user = LUNA_USER_DETAILS.load(deps.storage, user_name.clone())?;
        user.luna_airdrop_qualified = NOT_QUALIFIED_FOR_REWARD;
        LUNA_USER_DETAILS.save(deps.storage, user_name, &user)?;
    }

    return Ok(Response::default());
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&query_allowance(deps, owner.clone(), owner.clone())?),
        QueryMsg::UserActivityDetails { user_name } => {
            to_binary(&query_airdrop_activity_details(deps.storage, user_name)?)
        }
    }
}


pub fn query_airdrop_activity_details(
    storage: &dyn Storage,
    user_name: String,
) -> StdResult<Vec<UserActivityDetails>> {
    let ad = USER_ACTIVITY_DETAILS.may_load(storage, user_name)?;
    match ad {
        Some(ad) => return Ok(ad),
        None => return Err(StdError::generic_err("No airdrop activity details found")),
    };
}

fn query_all_activities(storage: &dyn Storage) -> StdResult<Vec<ActivityDetails>> {
    let mut all_activities = Vec::new();
    let all_acts: Vec<String> = ACTIVITY_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for act_name in all_acts {
        let activity_details = ACTIVITY_DETAILS.load(storage, act_name)?;
        for activity in activity_details {
            all_activities.push(activity);
        }
    }
    return Ok(all_activities);
}

fn query_all_user_activities(storage: &dyn Storage) -> StdResult<Vec<UserActivityDetails>> {
    let mut all_activities = Vec::new();
    let all_users: Vec<String> = USER_ACTIVITY_DETAILS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for user_name in all_users {
        let activity_details = USER_ACTIVITY_DETAILS.load(storage, user_name)?;
        for activity in activity_details {
            all_activities.push(activity);
        }
    }
    return Ok(all_activities);
}

fn query_luna_user_details(
    storage: &dyn Storage,
    user_name: String,
) -> StdResult<LunaUserDetails> {
    let lud = LUNA_USER_DETAILS.may_load(storage, user_name)?;
    match lud {
        Some(lud) => return Ok(lud),
        None => return Err(StdError::generic_err("No luna user details found")),
    };
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, CosmosMsg, StdError, SubMsg, WasmMsg};

    use super::*;
    use crate::msg::InstantiateMarketingInfo;

    use cosmwasm_std::coin;

    #[test]
    fn test_create_luna_user() {
        let mut deps = mock_dependencies(&[]);

        let user1Info = mock_info("LunaUser001", &[coin(1000, "stake")]);
        create_luna_user_details(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(),
            QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        let queryRes = query_luna_user_details(&mut deps.storage, "LunaUser001".to_string());
        match queryRes {
            Ok(lud) => {
                assert_eq!(lud.user_name, "LunaUser001".to_string());
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let user2Info = mock_info("LunaUser002", &[coin(1000, "stake")]);
        create_luna_user_details(deps.as_mut(), mock_env(), user2Info.clone(), "LunaUser002".to_string(),
            QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        let queryRes = query_luna_user_details(&mut deps.storage, "LunaUser002".to_string());
        match queryRes {
            Ok(lud) => {
                assert_eq!(lud.user_name, "LunaUser002".to_string());
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

    }

    #[test]
    fn test_create_million_luna_user() {
        let mut deps = mock_dependencies(&[]);

        let instantiate_msg = InstantiateMsg {
            cw20_token_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);

        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            GAMING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(11u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LIQUIDITY_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(42u128));

        let mut user_name_list_for_final_processing = Vec::new();
        let mut qualified_list_for_final_processing = Vec::new();
        let mut rw_amount_list_for_final_processing = Vec::new();
        let total_count = 1000;
        // Worked up to 1 million. Reducing it to 100 
        for count in 1..total_count+1 {
            let count_str : String = count.to_string();
            let mut username = String::new();
            username += "LunaUser_";
            username += &count_str;
            user_name_list_for_final_processing.push(username.clone());
            qualified_list_for_final_processing.push(QUALIFIED_FOR_REWARD);
            rw_amount_list_for_final_processing.push(Uint128::from(100u128));
            let userInfo = mock_info(&username, &[coin(1000, "stake")]);
            create_luna_user_details(deps.as_mut(), mock_env(), userInfo.clone(), username.clone(),
                QUALIFIED_FOR_REWARD, Uint128::from(100u128));

            let queryRes = query_luna_user_details(&mut deps.storage, username.clone());
            match queryRes {
                Ok(lud) => {
                    assert_eq!(lud.user_name, username.clone());
                }
                Err(e) => {
                    println!("error parsing header: {:?}", e);
                    assert_eq!(1, 2);
                }
            }
        }

        instantiate(deps.as_mut(), mock_env(), rewardInfo.clone(), instantiate_msg).unwrap();
        
        
        clear_qualified_flag_for_all_luna_users_and_non_exempt_activities(deps.as_mut(), mock_env(), rewardInfo.clone());

        let all_luna_users: Vec<String> = LUNA_USER_DETAILS
            .keys(&deps.storage, None, None, Order::Ascending)
            .map(|k| String::from_utf8(k).unwrap())
            .collect();
        for user in all_luna_users {
            // check that these many can be loaded in memory
            // it maxes out at 2 million for my machine
            // i7 processor, 32GB RAM, 1 TB SSD

            let queryRes = query_luna_user_details (&deps.storage, user);
            match queryRes {
                Ok(lud) => {
                    assert_eq!(lud.luna_airdrop_qualified, NOT_QUALIFIED_FOR_REWARD);
                    assert_eq!(lud.luna_airdrop_reward_amount, Uint128::zero());
                }
                Err(e) => {
                    println!("error parsing header: {:?}", e);
                    assert_eq!(1, 2);
                }
            }
        }
        let queryAllUserActRes = query_all_user_activities(&mut deps.storage);
        match queryAllUserActRes {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), total_count*NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name != STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.activity_qualified, NOT_QUALIFIED_FOR_REWARD);
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_luna_user_list_details(deps.as_mut(), mock_env(), rewardInfo.clone(), total_count, 
                user_name_list_for_final_processing, qualified_list_for_final_processing, rw_amount_list_for_final_processing);

        let all_luna_users_final: Vec<String> = LUNA_USER_DETAILS
            .keys(&deps.storage, None, None, Order::Ascending)
            .map(|k| String::from_utf8(k).unwrap())
            .collect();
        for user in all_luna_users_final {
            // check that these many can be loaded in memory
            // it maxes out at 2 million for my machine
            // i7 processor, 32GB RAM, 1 TB SSD

            let queryRes = query_luna_user_details (&deps.storage, user);
            match queryRes {
                Ok(lud) => {
                    assert_eq!(lud.luna_airdrop_qualified, QUALIFIED_FOR_REWARD);
                    assert_eq!(lud.luna_airdrop_reward_amount, Uint128::from(100u128));
                }
                Err(e) => {
                    println!("error parsing header: {:?}", e);
                    assert_eq!(1, 2);
                }
            }
        }
    }

    #[test]
    fn test_userlist_update_activity() {
        let mut deps = mock_dependencies(&[]);

        let instantiate_msg = InstantiateMsg {
            cw20_token_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);

        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            GAMING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(11u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LIQUIDITY_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(42u128));

        let mut user_name_list_for_final_processing = Vec::new();
        let total_count = 1000;
        // Worked up to 1 million. Reducing it to 100 
        for count in 1..total_count+1 {
            let count_str : String = count.to_string();
            let mut username = String::new();
            username += "LunaUser_";
            username += &count_str;

			let mut user_reward = UserRewardInfo { 
				user_name: username.clone(),
				reward_amount: Uint128::from(100u128),
			};
			user_name_list_for_final_processing.push (user_reward);
        }
		
        instantiate(deps.as_mut(), mock_env(), rewardInfo.clone(), instantiate_msg).unwrap();

		update_activity_reward_for_users (deps.as_mut(), mock_env(), rewardInfo.clone(), 
			"STAKING_ACTIVITY".to_string(), user_name_list_for_final_processing.clone());
        
        let all_luna_users: Vec<String> = LUNA_USER_DETAILS
            .keys(&deps.storage, None, None, Order::Ascending)
            .map(|k| String::from_utf8(k).unwrap())
            .collect();
        for user in all_luna_users {
            // check that these many can be loaded in memory
            // it maxes out at 2 million for my machine
            // i7 processor, 32GB RAM, 1 TB SSD

            let queryRes = query_luna_user_details (&deps.storage, user);
            match queryRes {
                Ok(lud) => {
                    assert_eq!(lud.luna_airdrop_qualified, QUALIFIED_FOR_REWARD);
                    assert_eq!(lud.luna_airdrop_reward_amount, Uint128::zero());
                }
                Err(e) => {
                    println!("error parsing header: {:?}", e);
                    assert_eq!(1, 2);
                }
            }
        }
        let queryAllUserActRes = query_all_user_activities(&mut deps.storage);
        match queryAllUserActRes {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), total_count*NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name != STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                    } else {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::from(100u128));
					}
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

		update_activity_reward_for_users (deps.as_mut(), mock_env(), rewardInfo.clone(), 
			"LUNA_ACTIVITY".to_string(), user_name_list_for_final_processing.clone());

        let all_luna_users_2: Vec<String> = LUNA_USER_DETAILS
            .keys(&deps.storage, None, None, Order::Ascending)
            .map(|k| String::from_utf8(k).unwrap())
            .collect();
        for user in all_luna_users_2 {
            // check that these many can be loaded in memory
            // it maxes out at 2 million for my machine
            // i7 processor, 32GB RAM, 1 TB SSD

            let queryRes = query_luna_user_details (&deps.storage, user);
            match queryRes {
                Ok(lud) => {
                    assert_eq!(lud.luna_airdrop_qualified, QUALIFIED_FOR_REWARD);
                    assert_eq!(lud.luna_airdrop_reward_amount, Uint128::zero());
                }
                Err(e) => {
                    println!("error parsing header: {:?}", e);
                    assert_eq!(1, 2);
                }
            }
        }
        let queryAllUserActRes_2 = query_all_user_activities(&mut deps.storage);
        match queryAllUserActRes_2 {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), total_count*NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name == STAKING_ACTIVITY.to_string() 
                       || act.activity_name == LUNA_ACTIVITY.to_string() {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::from(100u128));
                    } else {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
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
    fn test_user_activities () {
        let mut deps = mock_dependencies(&[]);

        let instantiate_msg = InstantiateMsg {
            cw20_token_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);

        set_contract_lock_status (deps.as_mut(), Uint128::from(UNLOCKED));
        
        let user1Info = mock_info("LunaUser001", &[coin(1000, "stake")]);
        create_luna_user_details(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(),
            QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            GAMING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(11u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LIQUIDITY_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(42u128));

        let queryRes = query_all_user_activities(&mut deps.storage);
        match queryRes {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_user_activity(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(), 
            STAKING_ACTIVITY.to_string(), QUALIFIED_FOR_REWARD);

        let queryResAfter = query_all_user_activities(&mut deps.storage);
        match queryResAfter {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name == STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::from(33u128));
                    } else {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
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
    fn test_update_activity_eligibile_amount () {
        let mut deps = mock_dependencies(&[]);

        let instantiate_msg = InstantiateMsg {
            cw20_token_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);

        set_contract_lock_status (deps.as_mut(), Uint128::from(UNLOCKED));
        
        let user1Info = mock_info("LunaUser001", &[coin(1000, "stake")]);
        create_luna_user_details(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(),
            QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LUNA_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            GAMING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(11u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LIQUIDITY_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(42u128));

        let queryRes = query_all_user_activities(&mut deps.storage);
        match queryRes {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_activity_eligibility_reward_amount(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), Uint128::from(88u128));
        let queryResAfterUpdateEligible = query_all_activities(&mut deps.storage);
        match queryResAfterUpdateEligible {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name == STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(88u128));
                    } else if act.activity_name == GAMING_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(11u128));
                    } else if act.activity_name == LIQUIDITY_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(42u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_user_activity(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(), 
            STAKING_ACTIVITY.to_string(), QUALIFIED_FOR_REWARD);

        let queryResAfterUpdateUser = query_all_user_activities(&mut deps.storage);
        match queryResAfterUpdateUser {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name == STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::from(88u128));
                    } else {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
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
    fn test_claim_user_rewards () {
        let mut deps = mock_dependencies(&[]);

        let instantiate_msg = InstantiateMsg {
            cw20_token_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);

        set_contract_lock_status (deps.as_mut(), Uint128::from(UNLOCKED));
        
        let user1Info = mock_info("LunaUser001", &[coin(1000, "stake")]);
        create_luna_user_details(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(),
            QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LUNA_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            GAMING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(11u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LIQUIDITY_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(42u128));


        let queryRes = query_all_user_activities(&mut deps.storage);
        match queryRes {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_activity_eligibility_reward_amount(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), Uint128::from(88u128));
        let queryResAfterUpdateEligible = query_all_activities(&mut deps.storage);
        match queryResAfterUpdateEligible {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name == STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(88u128));
                    } else if act.activity_name == GAMING_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(11u128));
                    } else if act.activity_name == LIQUIDITY_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(42u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_user_activity(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(), 
            STAKING_ACTIVITY.to_string(), QUALIFIED_FOR_REWARD);

        let queryResAfterUpdateUser = query_all_user_activities(&mut deps.storage);
        match queryResAfterUpdateUser {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name == STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::from(88u128));
                    } else {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let rsp1 = claim_user_rewards(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string());
        match rsp1 {
            Ok(rsp1) => {
                assert_eq!(rsp1, Response::new().add_attribute("reward", Uint128::from(88u128)));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_luna_user_details(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(), 
            QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        let rsp = claim_user_rewards(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string());
        match rsp {
            Ok(rsp) => {
                /*
                    Nov 30, 2021: Reward does not get the luna user details reward info anymore
                    So changing it from 188u128 to 88u128
                */
                assert_eq!(rsp, Response::new().add_attribute("reward", Uint128::from(88u128)));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let queryResAfterClaimReward = query_all_user_activities(&mut deps.storage);
        match queryResAfterClaimReward {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_update_activity_if_luna_is_not_qualified () {
        let mut deps = mock_dependencies(&[]);

        let instantiate_msg = InstantiateMsg {
            cw20_token_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);

        set_contract_lock_status (deps.as_mut(), Uint128::from(UNLOCKED));
        
        let user1Info = mock_info("LunaUser001", &[coin(1000, "stake")]);
        create_luna_user_details(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(),
            NOT_QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LUNA_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(33u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            GAMING_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(11u128));
        create_activity(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            LIQUIDITY_ACTIVITY.to_string(), LUNA_ACTIVITY.to_string(), Uint128::from(42u128));

        let queryRes = query_all_user_activities(&mut deps.storage);
        match queryRes {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_activity_eligibility_reward_amount(deps.as_mut(), mock_env(), rewardInfo.clone(), 
            STAKING_ACTIVITY.to_string(), Uint128::from(88u128));
        let queryResAfterUpdateEligible = query_all_activities(&mut deps.storage);
        match queryResAfterUpdateEligible {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    if act.activity_name == STAKING_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(88u128));
                    } else if act.activity_name == GAMING_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(11u128));
                    } else if act.activity_name == LIQUIDITY_ACTIVITY.to_string() {
                        assert_eq!(act.eligible_activity_reward_amount, Uint128::from(42u128));
                    }
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_user_activity(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(), 
            STAKING_ACTIVITY.to_string(), QUALIFIED_FOR_REWARD);

        let queryResAfterUpdateUser = query_all_user_activities(&mut deps.storage);
        match queryResAfterUpdateUser {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                        assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        update_luna_user_details(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string(), 
            QUALIFIED_FOR_REWARD, Uint128::from(100u128));

        let rsp = claim_user_rewards(deps.as_mut(), mock_env(), user1Info.clone(), "LunaUser001".to_string());
        match rsp {
            Ok(rsp) => {
                /*
                    Nov 30, 2021: Reward does not get the luna user details reward info anymore
                    So changing it from 100u128 to 0u128
                */
                assert_eq!(rsp, Response::new().add_attribute("reward", Uint128::from(0u128)));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let queryResAfterClaimReward = query_all_user_activities(&mut deps.storage);
        match queryResAfterClaimReward {
            Ok(all_acts) => {
                assert_eq!(all_acts.len(), NUM_OF_USER_ACTIVITIES);
                for act in all_acts {
                    assert_eq!(act.activity_reward_amount_accrued, Uint128::zero());
                }
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_lock_unlock () {
        let mut deps = mock_dependencies(&[]);
        set_contract_lock_status (deps.as_mut(), Uint128::from(LOCKED));
        let queryRes = get_contract_lock_status(deps.as_mut());
        match queryRes {
            Ok(lud) => {
                assert_eq!(lud, Uint128::from(LOCKED));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        set_contract_lock_status (deps.as_mut(), Uint128::from(UNLOCKED));
        let queryRes = get_contract_lock_status(deps.as_mut());
        match queryRes {
            Ok(lud) => {
                assert_eq!(lud, Uint128::from(UNLOCKED));
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }
}
