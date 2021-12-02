use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

use crate::states::config::Config;

pub fn update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: Option<String>,
    beneficiary: Option<String>,
    cap_strategy: Option<String>,
    whitelist_enabled: Option<bool>,
) -> super::ExecuteResult {
    let mut config = Config::load(deps.storage)?;

    if let Some(v) = owner {
        config.owner = deps.api.addr_validate(v.as_str())?;
    }

    if let Some(v) = beneficiary {
        config.beneficiary = deps.api.addr_validate(v.as_str())?;
    }

    config.cap_strategy = cap_strategy;

    if let Some(v) = whitelist_enabled {
        config.whitelist_enabled = v;
    }

    Config::save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_config")]))
}
