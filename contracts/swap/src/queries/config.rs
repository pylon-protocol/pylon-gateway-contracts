use cosmwasm_std::{to_binary, Deps, Env};
use pylon_gateway::swap_resp::ConfigResponse;

use crate::states::config::Config;

pub fn query_config(deps: Deps, _env: Env) -> super::QueryResult {
    let config = Config::load(deps.storage)?;

    Ok(to_binary(&ConfigResponse {
        owner: config.owner.to_string(),
        beneficiary: config.beneficiary.to_string(),
        start: config.start,
        finish: config.finish,
        price: config.price,
        total_sale_amount: config.swap_pool_size,
    })?)
}
