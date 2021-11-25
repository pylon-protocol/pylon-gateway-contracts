use cosmwasm_std::{Addr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub pool: Addr,
}

impl Config {
    pub fn load(storage: &dyn Storage) -> StdResult<Config> {
        ReadonlySingleton::new(storage, b"config").load()
    }

    pub fn save(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
        Singleton::new(storage, b"config").save(config)
    }
}
