use cosmwasm_std::{to_binary, Deps, Env};
use pylon_gateway::pool_resp::ConfigResponse;

use crate::states::config::Config;

pub fn query_config(deps: Deps, _env: Env) -> super::QueryResult {
    let config = Config::load(deps.storage)?;

    Ok(to_binary(&ConfigResponse {
        owner: config.owner.to_string(),
        token: config.token.to_string(),
        share_token: config.share_token.to_string(),
        deposit_time: config
            .deposit_time
            .iter()
            .map(|x| (x.start, x.finish, x.inverse))
            .collect(),
        withdraw_time: config
            .withdraw_time
            .iter()
            .map(|x| (x.start, x.finish, x.inverse))
            .collect(),
        deposit_cap_strategy: config.deposit_cap_strategy.map(|x| x.to_string()),
        reward_token: config.reward_token.to_string(),
        reward_rate: config.reward_rate,
        reward_claim_time: config
            .reward_claim_time
            .iter()
            .map(|x| (x.start, x.finish, x.inverse))
            .collect(),
        reward_distribution_time: (
            config.reward_distribution_time.start,
            config.reward_distribution_time.finish,
        ),
    })?)
}
