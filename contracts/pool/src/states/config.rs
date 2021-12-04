use cosmwasm_std::{Addr, Decimal, Env, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use pylon_gateway::time_range::TimeRange;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub token: Addr,
    // pool
    pub share_token: Addr,
    pub deposit_time: Vec<TimeRange>,
    pub withdraw_time: Vec<TimeRange>,
    pub deposit_cap_strategy: Option<Addr>,
    // reward
    pub reward_token: Addr,
    pub reward_rate: Decimal,
    pub reward_claim_time: Vec<TimeRange>,
    pub reward_distribution_time: TimeRange,
}

impl Config {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Self>::new(storage, super::KEY_CONFIG).load()
    }

    pub fn save(storage: &mut dyn Storage, config: &Self) -> StdResult<()> {
        Singleton::<Self>::new(storage, super::KEY_CONFIG).save(config)
    }

    // validator
    pub fn check_deposit_time(&self, env: &Env) -> Result<(), ContractError> {
        for (_, is_in_range) in self
            .deposit_time
            .iter()
            .map(|time| (time, time.is_in_range(env)))
        {
            if is_in_range {
                return Ok(());
            }
        }

        Err(ContractError::InvalidDepositTime {})
    }

    pub fn check_withdraw_time(&self, env: &Env) -> Result<(), ContractError> {
        for (_, is_in_range) in self
            .withdraw_time
            .iter()
            .map(|time| (time, time.is_in_range(env)))
        {
            if is_in_range {
                return Ok(());
            }
        }

        Err(ContractError::InvalidWithdrawTime {})
    }

    pub fn check_claim_time(&self, env: &Env) -> Result<(), ContractError> {
        for (_, is_in_range) in self
            .reward_claim_time
            .iter()
            .map(|time| (time, time.is_in_range(env)))
        {
            if is_in_range {
                return Ok(());
            }
        }

        Err(ContractError::InvalidClaimTime {})
    }
}
