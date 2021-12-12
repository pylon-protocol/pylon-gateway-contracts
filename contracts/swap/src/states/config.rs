use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::cap_strategy::CapStrategy;
use crate::types::distribution_strategy::DistributionStrategy;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // accounts
    pub owner: Addr,
    pub beneficiary: Addr,
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

impl Config {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Self>::new(storage, super::KEY_CONFIG).load()
    }

    pub fn save(storage: &mut dyn Storage, data: &Self) -> StdResult<()> {
        Singleton::<Self>::new(storage, super::KEY_CONFIG).save(data)
    }
}
