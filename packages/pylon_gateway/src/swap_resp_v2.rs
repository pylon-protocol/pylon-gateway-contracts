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
