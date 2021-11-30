use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Response, Uint128};
use cosmwasm_storage::ReadonlySingleton;
use pylon_gateway::pool_msg::MigrateMsg;
use pylon_gateway::time_range::TimeRange;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::states::config::Config;
use crate::states::reward::Reward;
use crate::states::{KEY_CONFIG, KEY_REWARD};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositConfig {
    pub time: TimeRange,
    pub user_cap: Uint256,
    pub total_cap: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionConfig {
    pub time: TimeRange,
    pub reward_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub owner: String,
    // share
    pub share_token: String,
    pub deposit_config: DepositConfig,
    pub withdraw_time: Vec<TimeRange>,
    // reward
    pub reward_token: String,
    pub claim_time: TimeRange,
    pub distribution_config: DistributionConfig,
    // strategy
    pub cap_strategy: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyReward {
    pub total_deposit: Uint256,
    pub last_update_time: u64,
    pub reward_per_token_stored: Decimal256,
}

pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> super::MigrateResult {
    let api = deps.api;

    let legacy_config = ReadonlySingleton::<LegacyConfig>::new(deps.storage, KEY_CONFIG).load()?;
    Config::save(
        deps.storage,
        &Config {
            owner: api.addr_validate(legacy_config.owner.as_str())?,
            token: Addr::unchecked("".to_string()),
            share_token: api.addr_validate(legacy_config.share_token.as_str())?,
            deposit_time: vec![legacy_config.deposit_config.time],
            withdraw_time: legacy_config.withdraw_time,
            deposit_cap_strategy: legacy_config
                .cap_strategy
                .map(|x| api.addr_validate(x.as_str()).unwrap()),
            reward_token: api.addr_validate(legacy_config.reward_token.as_str())?,
            reward_rate: Decimal::from(legacy_config.distribution_config.reward_rate),
            reward_claim_time: vec![legacy_config.claim_time],
            reward_distribution_time: legacy_config.distribution_config.time,
        },
    )?;

    let legacy_reward = ReadonlySingleton::<LegacyReward>::new(deps.storage, KEY_REWARD).load()?;
    Reward::save(
        deps.storage,
        &Reward {
            total_deposit: Uint128::from(legacy_reward.total_deposit),
            last_update_time: legacy_reward.last_update_time,
            reward_per_token_stored: Decimal::from(legacy_reward.reward_per_token_stored),
        },
    )?;

    Ok(Response::new())
}
