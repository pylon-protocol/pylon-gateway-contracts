use cosmwasm_std::{to_binary, Deps, Env};
use pylon_gateway::{swap_resp, swap_resp_v2};

use crate::states::config::Config;

pub fn query_config(deps: Deps, _env: Env) -> super::QueryResult {
    let config = Config::load(deps.storage)?;

    Ok(to_binary(&swap_resp::ConfigResponse {
        owner: config.owner.to_string(),
        beneficiary: config.beneficiary.to_string(),
        start: config.start,
        finish: config.finish,
        price: config.price,
        total_sale_amount: config.amount,
    })?)
}

pub fn query_config_v2(deps: Deps, _env: Env) -> super::QueryResult {
    let config = Config::load(deps.storage)?;

    Ok(to_binary(&swap_resp_v2::ConfigResponse {
        owner: config.owner.to_string(),
        beneficiary: config.beneficiary.to_string(),
        start: config.start,
        finish: config.finish,
        price: config.price,
        amount: config.amount,
        input_token: config.input_token,
        output_token: config.output_token,
        deposit_cap_strategy: config.deposit_cap_strategy.map(|x| x.into()),
        distribution_strategies: config
            .distribution_strategies
            .iter()
            .map(|x| x.clone().into())
            .collect(),
        whitelist_enabled: config.whitelist_enabled,
    })?)
}
