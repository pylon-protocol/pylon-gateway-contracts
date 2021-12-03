use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};
use cw20::Denom;
use pylon_gateway::swap_types::{
    CapStrategy as SwapCapStrategy, DistributionStrategy as SwapDistributionStrategy,
};

use crate::states::config::Config;
use crate::types::cap_strategy::CapStrategy;
use crate::types::distribution_strategy::DistributionStrategy;

pub fn update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: Option<String>,
    beneficiary: Option<String>,
    input_token: Option<String>,
    output_token: Option<String>,
    deposit_cap_strategy: Option<SwapCapStrategy>,
    distribution_strategies: Option<Vec<SwapDistributionStrategy>>,
    whitelist_enabled: Option<bool>,
) -> super::ExecuteResult {
    let mut config = Config::load(deps.storage)?;

    if let Some(v) = owner {
        config.owner = deps.api.addr_validate(v.as_str())?;
    }

    if let Some(v) = beneficiary {
        config.beneficiary = deps.api.addr_validate(v.as_str())?;
    }

    if let Some(v) = input_token {
        config.input_token = Denom::Native(v);
    }

    if let Some(v) = output_token {
        config.output_token = Denom::Cw20(deps.api.addr_validate(v.as_str())?);
    }

    config.deposit_cap_strategy = deposit_cap_strategy.map(CapStrategy::from);

    if let Some(v) = distribution_strategies {
        config.distribution_strategies = v.iter().map(DistributionStrategy::from).collect();
    }

    if let Some(v) = whitelist_enabled {
        config.whitelist_enabled = v;
    }

    Config::save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}
