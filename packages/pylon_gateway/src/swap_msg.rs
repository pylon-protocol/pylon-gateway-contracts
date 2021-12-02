use cosmwasm_std::{Decimal, Uint128};
use pylon_utils::common::OrderBy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::swap_types::{CapStrategy, DistributionStrategy};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Lockup {
        release_time: u64,
        release_amount: Decimal,
    },
    Vesting {
        release_start_time: u64,
        release_finish_time: u64,
        release_amount: Decimal,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub beneficiary: String,
    pub start: u64,
    pub period: u64,
    pub price: Decimal,
    pub amount: Uint128,
    pub input_token: String,
    pub output_token: String,
    pub x_liquidity: Uint128,
    pub y_liquidity: Uint128, // is also a maximum cap of this pool
    pub deposit_cap_strategy: Option<CapStrategy>,
    pub distribution_strategies: Vec<DistributionStrategy>,
    pub whitelist_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigureMsg {
    Swap {
        owner: Option<String>,
        beneficiary: Option<String>,
        cap_strategy: Option<String>,
        whitelist_enabled: Option<bool>,
    },
    Pool {
        x_denom: Option<String>,
        y_addr: Option<String>,
        liq_x: Option<Uint128>,
        liq_y: Option<Uint128>,
    },
    Whitelist {
        whitelist: bool,
        candidates: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Configure(ConfigureMsg),
    Deposit {},
    Withdraw { amount: Uint128 },
    Claim {},
    Earn {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    User {
        address: String,
    },
    Users {
        start_after: Option<String>,
        limit: Option<u32>,
        order: Option<OrderBy>,
    },
    CurrentPrice {},
    SimulateWithdraw {
        amount: Uint128,
        address: Option<String>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
