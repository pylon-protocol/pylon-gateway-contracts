use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_swapped: Uint128,
    pub total_claimed: Uint128,

    pub x_liquidity: Uint128,
    pub y_liquidity: Uint128,
}

impl State {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Self>::new(storage, super::KEY_STATE).load()
    }

    pub fn save(storage: &mut dyn Storage, data: &Self) -> StdResult<()> {
        Singleton::<Self>::new(storage, super::KEY_STATE).save(data)
    }
}
