use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::states::config::Config;
use crate::states::state::State;

pub fn update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    x_denom: Option<String>,
    y_addr: Option<String>,
    liq_x: Option<Uint128>,
    liq_y: Option<Uint128>,
) -> super::ExecuteResult {
    let mut state = State::load(deps.storage)?;

    if let Some(v) = x_denom {
        state.x_denom = v;
    }
    if let Some(v) = y_addr {
        state.y_addr = deps.api.addr_validate(v.as_str())?;
    }
    if let Some(v) = liq_x {
        state.x_liquidity = v;
    }
    if let Some(v) = liq_y {
        state.y_liquidity = v;
    }

    State::save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_state")]))
}
