use cosmwasm_std::{to_binary, Deps, Env};
use pylon_gateway::pool_resp::{StakerResponse, StakersResponse};
use pylon_utils::common::OrderBy;
use std::cmp::{max, min};

use crate::executions::staking::calculate_rewards;
use crate::states::config::Config;
use crate::states::reward::Reward;
use crate::states::user::User;

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

    let staker = StakerResponse {
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
        .map(|(address, user)| -> StakerResponse {
            StakerResponse {
                address: deps.api.addr_humanize(address).unwrap().to_string(),
                staked: user.amount,
                reward: calculate_rewards(&config, &reward, user, &applicable_reward_time).unwrap(),
            }
        })
        .collect();

    Ok(to_binary(&StakersResponse { stakers })?)
}
