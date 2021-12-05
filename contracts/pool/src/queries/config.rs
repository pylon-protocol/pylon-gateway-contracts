use cosmwasm_std::{to_binary, Deps, Env, Uint128};
use pylon_gateway::pool_resp;
use pylon_gateway::pool_resp_v2;

use crate::states::config::Config;

pub fn query_config(deps: Deps, _env: Env) -> super::QueryResult {
    let config = Config::load(deps.storage)?;

    Ok(to_binary(&pool_resp::ConfigResponse {
        // main
        owner: config.owner.to_string(),
        // staking
        share_token: config.share_token.to_string(),
        deposit_config: pool_resp::ConfigResponseDepositConfig {
            time: config.deposit_time.first().unwrap().clone(),
            user_cap: Uint128::zero().to_string(),
            total_cap: Uint128::zero().to_string(),
        },
        withdraw_time: config.withdraw_time,
        // reward
        reward_token: config.reward_token.to_string(),
        claim_time: config.reward_claim_time.first().unwrap().clone(),
        distribution_config: pool_resp::ConfigResponseDistributionConfig {
            time: config.reward_distribution_time.clone(),
            reward_rate: config.reward_rate,
            total_reward_amount: config.reward_rate
                * Uint128::from(config.reward_distribution_time.period()),
        },
    })?)
}

pub fn query_config_v2(deps: Deps, _env: Env) -> super::QueryResult {
    let config = Config::load(deps.storage)?;

    Ok(to_binary(&pool_resp_v2::ConfigResponse {
        // main
        owner: config.owner.to_string(),
        token: config.token.to_string(),
        // staking
        share_token: config.share_token.to_string(),
        deposit_time: config.deposit_time,
        withdraw_time: config.withdraw_time,
        deposit_cap_strategy: config.deposit_cap_strategy.map(|x| x.to_string()),
        // reward
        reward_token: config.reward_token.to_string(),
        reward_rate: config.reward_rate,
        reward_claim_time: config.reward_claim_time,
        reward_distribution_time: config.reward_distribution_time,
    })?)
}
