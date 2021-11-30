use cosmwasm_std::{Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reward {
    pub total_deposit: Uint128,
    pub last_update_time: u64,
    pub reward_per_token_stored: Decimal,
}

impl Reward {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Reward>::new(storage, super::KEY_REWARD).load()
    }

    pub fn save(storage: &mut dyn Storage, reward: &Self) -> StdResult<()> {
        Singleton::<Reward>::new(storage, super::KEY_REWARD).save(reward)
    }
}
