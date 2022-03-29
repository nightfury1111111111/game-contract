use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin_address: Addr,
    pub minting_contract_address: Addr,
    pub astro_proxy_address: Addr,
    pub club_fee_collector_wallet: Addr,
    pub club_reward_next_timestamp: Timestamp,
    pub reward_periodicity: u64,
    pub club_price: Uint128,
    pub bonding_duration: u64,
    pub owner_release_locking_duration: u64,
    pub platform_fees_collector_wallet: Addr,
    ///Specified in percentage multiplied by 100, i.e. 100% = 10000 and 0.01% = 1
    pub platform_fees: Uint128,
    ///Specified in percentage multiplied by 100, i.e. 100% = 10000 and 0.01% = 1
    pub transaction_fees: Uint128,
    ///Specified in percentage multiplied by 100, i.e. 100% = 10000 and 0.01% = 1
    pub control_fees: Uint128,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct ClubOwnershipDetails {
    /// The club name
    pub club_name: String,
    /// The system timestamp to be used as starting point when ownership
    /// of a club was released by the owner to sell it to another buyer
    pub start_timestamp: Timestamp,

    /// The locking period (days) expressed in seconds from start_timestamp
    /// after which the owner_released flag is no longer applicable
    pub locking_period: u64,

    pub owner_address: String,

    pub price_paid: Uint128,

    /// reward amount in quantity of tokens
    pub reward_amount: Uint128,

    /// has owner released the club to let another buyer purchase it
    pub owner_released: bool,
}

/// Used to shift previous owner from ClubOwnerShipDetails to a new state variable -
/// used by previous owner using new verb PreviousOwnerRewardOut()
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct ClubPreviousOwnerDetails {
    /// The previous owner name
    pub previous_owner_address: String,

    /// previous owner reward amount
    pub reward_amount: Uint128,
}

/// This is used for saving various vesting details
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct ClubStakingDetails {
    pub club_name: String,

    pub staker_address: String,

    /// The system timestamp to be used as starting point of staking
    pub staking_start_timestamp: Timestamp,

    /// staked amount in quantity of tokens
    pub staked_amount: Uint128,

    /// Duration of staking expressed in seconds
    pub staking_duration: u64,

    /// reward amount in quantity of tokens
    pub reward_amount: Uint128,

    /// whether rewards are auto-staked or do they need to be claimed
    pub auto_stake: bool,
}

/// This is used for saving various bonding details for an unstaked club
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct ClubBondingDetails {
    pub club_name: String,

    pub bonder_address: String,

    /// The system timestamp to be used as starting point of bonding
    pub bonding_start_timestamp: Timestamp,

    /// bonded amount in quantity of tokens
    pub bonded_amount: Uint128,

    /// Duration of bonding expressed in seconds
    pub bonding_duration: u64,
}

// pub const ALLOWANCES: Map<(&Addr, &Addr), AllowanceResponse> = Map::new("allowance");

/// Map of clubs and its owners. the key is club name and the
/// ClubOwnershipDetails will contain information about the owner
pub const CLUB_OWNERSHIP_DETAILS: Map<String, ClubOwnershipDetails> =
    Map::new("club_ownership_details");

/// Map of clubs and its stakers. the key is club name and the
/// ClubStakingDetails will contain information about the stakers and amount staked
pub const CLUB_STAKING_DETAILS: Map<String, Vec<ClubStakingDetails>> =
    Map::new("club_staking_details");

/// Map of clubs and its bonders. the key is club name and the
/// ClubBondingDetails will contain information about the bonders and amount bonded
pub const CLUB_BONDING_DETAILS: Map<String, Vec<ClubBondingDetails>> =
    Map::new("club_bonding_details");

/// Map of previous owners and their reward points. the key is owner address and the
/// ClubPreviousOwnerDetails will contain information about the
/// previous owner of the club and his reward points
pub const CLUB_PREVIOUS_OWNER_DETAILS: Map<String, ClubPreviousOwnerDetails> =
    Map::new("club_previous_owner_details");

pub const REWARD: Item<Uint128> = Item::new("staking_reward");

pub const CLUB_REWARD_NEXT_TIMESTAMP: Item<Timestamp> = Item::new("club_reward_next_timestamp");

/// Snapshot of ranking by stakes
pub const CLUB_STAKING_SNAPSHOT: Map<String, Uint128> =
    Map::new("club_staking_snapshot");

