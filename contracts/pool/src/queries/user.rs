use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, Deps, Env};
use pylon_gateway::cap_strategy_msg::QueryMsg;
use pylon_gateway::pool_resp;
use pylon_utils::common::OrderBy;
use std::cmp::{max, min};

use crate::executions::staking::calculate_rewards;
use crate::states::config::Config;
use crate::states::reward::Reward;
use crate::states::user::User;

pub fn query_balance(deps: Deps, _env: Env, owner: String) -> super::QueryResult {
    let user_addr = deps.api.addr_canonicalize(owner.as_str())?;
    let user = User::load(deps.storage, &user_addr);

    Ok(to_binary(&pool_resp::BalanceOfResponse {
        amount: user.amount,
    })?)
}

pub fn query_claimable_reward(
    deps: Deps,
    env: Env,
    owner: String,
    timestamp: Option<u64>,
) -> super::QueryResult {
    let config = Config::load(deps.storage)?;
    let reward = Reward::load(deps.storage)?;
    let user_addr = deps.api.addr_canonicalize(owner.as_str())?;
    let user = User::load(deps.storage, &user_addr);

    Ok(to_binary(&pool_resp::ClaimableRewardResponse {
        amount: calculate_rewards(
            &config,
            &reward,
            &user,
            &timestamp.unwrap_or_else(|| env.block.time.seconds()),
        )?,
    })?)
}

pub fn query_available_cap(deps: Deps, _env: Env, address: String) -> super::QueryResult {
    let config = Config::load(deps.storage)?;
    let user_addr = deps.api.addr_canonicalize(address.as_str())?;
    let user = User::load(deps.storage, &user_addr);

    if let Some(strategy) = config.deposit_cap_strategy {
        let resp: pool_resp::AvailableCapOfResponse = deps.querier.query_wasm_smart(
            strategy.to_string(),
            &QueryMsg::AvailableCapOf {
                address,
                amount: Uint256::from(user.amount),
            },
        )?;
        Ok(to_binary(&resp)?)
    } else {
        Ok(to_binary(&pool_resp::AvailableCapOfResponse {
            amount: None,
            unlimited: true,
        })?)
    }
}

pub fn query_staker(deps: Deps, env: Env, address: String) -> super::QueryResult {
    let config = Config::load(deps.storage)?;
    let reward = Reward::load(deps.storage)?;
    let user = User::load(deps.storage, &deps.api.addr_canonicalize(address.as_str())?);
    let applicable_reward_time = min(
        max(
            config.reward_distribution_time.start,
            env.block.time.seconds(),
        ),
        config.reward_distribution_time.finish,
    );

    let staker = pool_resp::StakerResponse {
        address,
        staked: user.amount,
        reward: calculate_rewards(&config, &reward, &user, &applicable_reward_time)?,
    };

    Ok(to_binary(&staker)?)
}

pub fn query_stakers(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order: Option<OrderBy>,
) -> super::QueryResult {
    let config = Config::load(deps.storage)?;
    let reward = Reward::load(deps.storage)?;
    let users = User::load_range(
        deps.storage,
        start_after.map(|x| deps.api.addr_canonicalize(x.as_str()).unwrap()),
        limit,
        order,
    );
    let applicable_reward_time = min(
        max(
            config.reward_distribution_time.start,
            env.block.time.seconds(),
        ),
        config.reward_distribution_time.finish,
    );

    let stakers = users
        .iter()
        .map(|(address, user)| -> pool_resp::StakerResponse {
            pool_resp::StakerResponse {
                address: deps.api.addr_humanize(address).unwrap().to_string(),
                staked: user.amount,
                reward: calculate_rewards(&config, &reward, user, &applicable_reward_time).unwrap(),
            }
        })
        .collect();

    Ok(to_binary(&pool_resp::StakersResponse { stakers })?)
}
