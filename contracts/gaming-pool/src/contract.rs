use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{
    cancel_game, claim_refund, claim_reward, create_pool, game_pool_bid_submit,
    game_pool_reward_distribute, lock_game, received_message, save_team_details,
    set_platform_fee_wallets, set_pool_type_params,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{
    get_team_count_for_user_in_pool_type, query_all_pool_type_details, query_all_pools_in_game,
    query_all_teams, query_game_details, query_game_result, query_pool_collection,
    query_pool_details, query_pool_team_details, query_pool_type_details, query_refund,
    query_reward, query_team_details,
};
use crate::state::{Config, GameDetails, GameResult, CONFIG, GAME_DETAILS, GAME_RESULT_DUMMY};

// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:gaming-pool";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DUMMY_WALLET: &str = "terra1t3czdl5h4w4qwgkzs80fdstj0z7rfv9v2j6uh3";

// Initial reward amount to gamer for joining a pool
pub const INITIAL_REWARD_AMOUNT: u128 = 0u128;
// Initial refund amount to gamer for joining a pool
pub const INITIAL_REFUND_AMOUNT: u128 = 0u128;

// Initial value of team points
pub const INITIAL_TEAM_POINTS: u64 = 0u64;

// Initial rank of team - set to a low rank more than max pool size
pub const INITIAL_TEAM_RANK: u64 = 100000u64;

pub const UNCLAIMED_REWARD: bool = false;
pub const CLAIMED_REWARD: bool = true;
pub const UNCLAIMED_REFUND: bool = false;
pub const CLAIMED_REFUND: bool = true;
pub const REWARDS_DISTRIBUTED: bool = true;
pub const REWARDS_NOT_DISTRIBUTED: bool = false;

pub const GAME_POOL_OPEN: u64 = 1u64;
pub const GAME_POOL_CLOSED: u64 = 2u64;
pub const GAME_CANCELLED: u64 = 3u64;
pub const GAME_COMPLETED: u64 = 4u64;
pub const HUNDRED_PERCENT: u128 = 10000u128;
pub const NINETY_NINE_NINE_PERCENT: u128 = 9990u128;

pub const DUMMY_TEAM_ID: &str = "DUMMY_TEAM_ID";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin_address: deps.api.addr_validate(&msg.admin_address)?,
        minting_contract_address: deps.api.addr_validate(&msg.minting_contract_address)?,
        platform_fees_collector_wallet: deps
            .api
            .addr_validate(&msg.platform_fees_collector_wallet)?,
        astro_proxy_address: deps.api.addr_validate(&msg.astro_proxy_address)?,
        platform_fee: msg.platform_fee,
        transaction_fee: msg.transaction_fee,
        game_id: msg.game_id.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    let dummy_wallet = String::from(DUMMY_WALLET);
    let main_address = deps.api.addr_validate(dummy_wallet.clone().as_str())?;
    GAME_RESULT_DUMMY.save(
        deps.storage,
        &main_address,
        &GameResult {
            gamer_address: DUMMY_WALLET.to_string(),
            game_id: msg.game_id.clone(),
            team_id: DUMMY_TEAM_ID.to_string(),
            team_rank: INITIAL_TEAM_RANK,
            team_points: INITIAL_TEAM_POINTS,
            reward_amount: Uint128::from(INITIAL_REWARD_AMOUNT),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        },
    )?;

    GAME_DETAILS.save(
        deps.storage,
        msg.game_id.clone(),
        &GameDetails {
            game_id: msg.game_id.clone(),
            game_status: GAME_POOL_OPEN,
        },
    )?;
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
        ExecuteMsg::SetPlatformFeeWallets { wallet_percentages } => {
            set_platform_fee_wallets(deps, info, wallet_percentages)
        }
        ExecuteMsg::SetPoolTypeParams {
            pool_type,
            pool_fee,
            min_teams_for_pool,
            max_teams_for_pool,
            max_teams_for_gamer,
            wallet_percentages,
        } => set_pool_type_params(
            deps,
            env,
            info,
            pool_type,
            pool_fee,
            min_teams_for_pool,
            max_teams_for_pool,
            max_teams_for_gamer,
            wallet_percentages,
        ),
        ExecuteMsg::CancelGame {} => cancel_game(deps, env, info),
        ExecuteMsg::LockGame {} => lock_game(deps, env, info),
        ExecuteMsg::CreatePool { pool_type } => create_pool(deps, env, info, pool_type),
        ExecuteMsg::ClaimReward { gamer } => claim_reward(deps, info, gamer, env),
        ExecuteMsg::ClaimRefund { gamer } => claim_refund(deps, info, gamer, env),
        ExecuteMsg::GamePoolRewardDistribute {
            pool_id,
            game_winners,
        } => game_pool_reward_distribute(deps, env, info, pool_id, game_winners),
        ExecuteMsg::SaveTeamDetails {
            gamer,
            pool_id,
            team_id,
            game_id,
            pool_type,
            reward_amount,
            claimed_reward,
            refund_amount,
            claimed_refund,
            team_points,
            team_rank,
        } => save_team_details(
            deps.storage,
            env,
            gamer,
            pool_id,
            team_id,
            game_id,
            pool_type,
            reward_amount,
            claimed_reward,
            refund_amount,
            claimed_refund,
            team_points,
            team_rank,
        ),
        ExecuteMsg::GamePoolBidSubmitCommand {
            gamer,
            pool_type,
            pool_id,
            team_id,
            amount,
        } => game_pool_bid_submit(
            deps, env, info, gamer, pool_type, pool_id, team_id, amount, false,
        ),
    }
}

// This is the safe way of contract migration
// We can add expose specific state properties to
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = cw2::get_contract_version(deps.storage)?;
    println!("Contract: {}, Version: {}", ver.contract, ver.version);
    // ensure we are migrating from an allowed contract
    // if ver.contract != CONTRACT_NAME {
    //     return Err(StdError::generic_err("Can only upgrade from same type").into());
    // }
    // // note: better to do proper semver compare, but string compare *usually* works
    // if ver.version >= CONTRACT_VERSION.to_string() {
    //     return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    // }
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PoolTeamDetails { pool_id } => {
            to_binary(&query_pool_team_details(deps.storage, pool_id)?)
        }
        QueryMsg::PoolDetails { pool_id } => to_binary(&query_pool_details(deps.storage, pool_id)?),
        QueryMsg::PoolTypeDetails { pool_type } => {
            to_binary(&query_pool_type_details(deps.storage, pool_type)?)
        }
        QueryMsg::AllPoolTypeDetails {} => to_binary(&query_all_pool_type_details(deps.storage)?),
        QueryMsg::AllTeams {} => to_binary(&query_all_teams(deps.storage)?),
        QueryMsg::QueryReward { gamer } => to_binary(&query_reward(deps.storage, gamer)?),
        QueryMsg::QueryRefund { gamer } => to_binary(&query_refund(deps.storage, gamer)?),
        QueryMsg::QueryGameResult {
            gamer,
            pool_id,
            team_id,
        } => to_binary(&query_game_result(deps, gamer, pool_id, team_id)?),
        QueryMsg::GameDetails {} => to_binary(&query_game_details(deps.storage)?),
        QueryMsg::PoolTeamDetailsWithTeamId { pool_id, team_id } => {
            to_binary(&query_team_details(deps.storage, pool_id, team_id)?)
        }
        QueryMsg::AllPoolsInGame {} => to_binary(&query_all_pools_in_game(deps.storage)?),
        QueryMsg::PoolCollection { pool_id } => {
            to_binary(&query_pool_collection(deps.storage, pool_id)?)
        }
        QueryMsg::GetTeamCountForUserInPoolType {
            game_id,
            gamer,
            pool_type,
        } => to_binary(&get_team_count_for_user_in_pool_type(
            deps.storage,
            gamer,
            game_id,
            pool_type,
        )?),
    }
}
