use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct VestingSchedule {
    /// Wallet address of the account.
    pub address: String,
    /// Amount of tokens allocated at the start of vesting.
    pub initial_vesting_count: Uint128,
    /// How often the vesting should occur. This is expressed in seconds
    pub vesting_periodicity: u64,
    /// Amount of tokens to be awarded in every vesting cycle
    pub vesting_count_per_period: Uint128,
    /// Total tokens to be awarded to the address
    pub total_vesting_token_count: Uint128,
    /// Cliff period in weeks
    pub cliff_period: u64,
    /// Address of the parent category to which this account is investing into
    pub parent_category_address: Option<String>,
    /// Flag to let system know if the vested amount has to be transferred immediately
    /// Or should be kept in allowances for the vester to claim
    pub should_transfer: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateVestingSchedulesInfo {
    pub vesting_schedules: Vec<VestingSchedule>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateMsg {
    pub admin_wallet: Addr,
    pub fury_token_contract: Addr,
    pub vesting: InstantiateVestingSchedulesInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    PeriodicallyTransferToCategories {},
    PeriodicallyCalculateVesting {},
    ClaimVestedTokens {
        amount: Uint128,
    },
    AddVestingSchedules {
        schedules: InstantiateVestingSchedulesInfo,
    },
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    VestingDetails { address: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct MigrateMsg {}
