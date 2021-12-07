use cosmwasm_std::{to_binary, Deps, Env};
use pylon_gateway::swap_resp::StateResponse;

use crate::states::state::State;

pub fn query_state(deps: Deps, _env: Env) -> super::QueryResult {
    let state = State::load(deps.storage)?;

    Ok(to_binary(&StateResponse {
        total_swapped: state.total_swapped,
        total_claimed: state.total_claimed,
    })?)
}
