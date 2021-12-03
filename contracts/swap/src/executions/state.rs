use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::states::config::Config;
use crate::states::state::State;

pub fn update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    x_liquidity: Option<Uint128>,
    y_liquidity: Option<Uint128>,
) -> super::ExecuteResult {
    let mut state = State::load(deps.storage)?;

    if let Some(v) = x_liquidity {
        state.x_liquidity = v;
    }
    if let Some(v) = y_liquidity {
        state.y_liquidity = v;
    }

    State::save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_state")]))
}
