use cosmwasm_std::{to_binary, Deps, Env};
use pylon_gateway::pool_resp::RewardResponse;

use crate::states::reward::Reward;

pub fn query_reward(deps: Deps, _env: Env) -> super::QueryResult {
    let reward = Reward::load(deps.storage)?;

    Ok(to_binary(&RewardResponse {
        total_deposit: reward.total_deposit,
        last_update_time: reward.last_update_time,
    })?)
}
