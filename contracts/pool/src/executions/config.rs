use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response, Uint128};
use pylon_gateway::time_range::TimeRange;
use std::cmp::max;

use crate::states::config::Config;

#[allow(clippy::too_many_arguments)]
pub fn update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: Option<String>,
    share_token: Option<String>,
    reward_token: Option<String>,
    claim_time: Option<Vec<(u64, u64, bool)>>,
    deposit_time: Option<Vec<(u64, u64, bool)>>,
    withdraw_time: Option<Vec<(u64, u64, bool)>>,
    deposit_cap_strategy: Option<String>,
) -> super::ExecuteResult {
    let mut config = Config::load(deps.storage)?;

    // authorization
    if let Some(v) = owner {
        config.owner = deps.api.addr_validate(v.as_str())?;
    }
    // tokens
    if let Some(v) = share_token {
        config.share_token = deps.api.addr_validate(v.as_str())?;
    }
    if let Some(v) = reward_token {
        config.reward_token = deps.api.addr_validate(v.as_str())?;
    }
    // times
    if let Some(v) = claim_time {
        config.reward_claim_time = v
            .iter()
            .map(|(start, finish, inverse)| TimeRange {
                start: *start,
                finish: *finish,
                inverse: *inverse,
            })
            .collect();
    }
    if let Some(v) = deposit_time {
        config.deposit_time = v
            .iter()
            .map(|(start, finish, inverse)| TimeRange {
                start: *start,
                finish: *finish,
                inverse: *inverse,
            })
            .collect();
    }
    if let Some(v) = withdraw_time {
        config.withdraw_time = v
            .iter()
            .map(|(start, finish, inverse)| TimeRange {
                start: *start,
                finish: *finish,
                inverse: *inverse,
            })
            .collect();
    }
    // strategy
    if let Some(v) = deposit_cap_strategy {
        config.deposit_cap_strategy = Some(deps.api.addr_validate(v.as_str())?);
    }

    Config::save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn adjust_reward(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
    remove: bool,
) -> super::ExecuteResult {
    let mut response = Response::new().add_attribute(
        "action",
        if remove { "sub_reward" } else { "add_reward" }.to_string(),
    );
    let mut config = Config::load(deps.storage)?;

    response = response.add_attribute("reward_rate_before", config.reward_rate.to_string());

    let remaining = Uint128::from(
        config.reward_distribution_time.finish
            - max(
                config.reward_distribution_time.start,
                env.block.time.seconds(),
            ),
    );

    config.reward_rate = if remove {
        config.reward_rate - Decimal::from_ratio(amount, remaining)
    } else {
        config.reward_rate + Decimal::from_ratio(amount, remaining)
    };

    Config::save(deps.storage, &config)?;

    response = response.add_attribute("reward_rate_after", config.reward_rate.to_string());

    Ok(response)
}
