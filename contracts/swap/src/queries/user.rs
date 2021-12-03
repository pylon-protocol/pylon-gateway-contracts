use cosmwasm_std::{to_binary, Deps, Env, QuerierWrapper};
use pylon_gateway::swap_resp::{UserResponse, UsersResponse};
use pylon_utils::common::OrderBy;

use crate::executions::swap::calculate_claimable_tokens;
use crate::states::config::Config;
use crate::states::user::User;

fn to_response(
    querier: QuerierWrapper,
    config: &Config,
    user: &User,
    address: String,
    time: u64,
) -> UserResponse {
    let claimable_token = calculate_claimable_tokens(&config, &user, time);

    UserResponse {
        whitelisted: user.whitelisted,
        swapped_in: user.swapped_in,
        available_cap: match config.deposit_cap_strategy.clone() {
            Some(strategy) => {
                let (cap, unlimited) = strategy.available_cap_of(querier, address, user.swapped_in);
                if unlimited {
                    None
                } else {
                    cap
                }
            }
            None => None,
        },
        reward_total: claimable_token,
        reward_remaining: user.swapped_out - (user.swapped_out_claimed + claimable_token),
    }
}

pub fn query_user(deps: Deps, env: Env, address: String) -> super::QueryResult {
    let user_addr = deps.api.addr_canonicalize(address.as_str())?;
    let user = User::load(deps.storage, &user_addr);
    let config = Config::load(deps.storage)?;

    Ok(to_binary(&to_response(
        deps.querier,
        &config,
        &user,
        address,
        env.block.time.seconds(),
    ))?)
}

pub fn query_users(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order: Option<OrderBy>,
) -> super::QueryResult {
    let config = Config::load(deps.storage)?;
    let api = deps.api;
    let users = User::load_range(
        deps.storage,
        start_after.map(|x| api.addr_canonicalize(x.as_str()).unwrap()),
        limit,
        order,
    )
    .iter()
    .map(|(user_addr, user)| -> (String, UserResponse) {
        let user_addr = api.addr_humanize(user_addr).unwrap();
        (
            user_addr.to_string(),
            to_response(
                deps.querier,
                &config,
                user,
                user_addr.to_string(),
                env.block.time.seconds(),
            ),
        )
    })
    .collect();

    Ok(to_binary(&UsersResponse { users })?)
}
