use cosmwasm_std::{Decimal, Uint128};
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::swap_types::{CapStrategy, DistributionStrategy};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    // accounts
    pub owner: String,
    pub beneficiary: String,
    // details
    pub start: u64,
    pub finish: u64,
    pub price: Decimal,
    pub amount: Uint128,
    pub input_token: Denom,
    pub output_token: Denom,
    // strategies
    pub deposit_cap_strategy: Option<CapStrategy>,
    pub distribution_strategies: Vec<DistributionStrategy>,
    pub whitelist_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_swapped: Uint128, // total supply
    pub total_claimed: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserResponse {
    pub whitelisted: bool,
    pub swapped_in: Uint128,
    pub available_cap: Option<Uint128>, // None = unlimited
    pub reward_total: Uint128,
    pub reward_remaining: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UsersResponse {
    pub users: Vec<(String, UserResponse)>,
}
