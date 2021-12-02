use cosmwasm_std::{to_binary, Coin, Deps, Uint128};
use cw20::Denom;
use pylon_gateway::swap_resp::{CurrentPriceResponse, SimulateWithdrawResponse};
use pylon_utils::tax::deduct_tax;

use crate::executions::swap::{calculate_current_price, calculate_withdraw_amount};
use crate::states::config::Config;
use crate::states::state::State;
use crate::states::user::User;

pub fn query_current_price(deps: Deps) -> super::QueryResult {
    let state = State::load(deps.storage)?;

    Ok(to_binary(&CurrentPriceResponse {
        price: calculate_current_price(&state),
    })?)
}

pub fn query_simulate_withdraw(
    deps: Deps,
    address: Option<String>,
    amount: Uint128,
) -> super::QueryResult {
    let config = Config::load(deps.storage)?;
    let state = State::load(deps.storage)?;

    let withdraw_amount = calculate_withdraw_amount(&state, &amount);
    let penalty = (amount * config.price) - withdraw_amount;

    let mut withdrawable = true;
    if let Some(address) = address {
        let user = User::load(deps.storage, &deps.api.addr_canonicalize(address.as_str())?);
        withdrawable = user.swapped_out_claimed.is_zero();
    }

    let input_denom = match config.input_token {
        Denom::Native(token) => token.to_string(),
        Denom::Cw20(_) => panic!("input token should be native"),
    };

    Ok(to_binary(&SimulateWithdrawResponse {
        amount: deduct_tax(
            deps,
            Coin {
                denom: input_denom,
                amount: withdraw_amount,
            },
        )?
        .amount,
        penalty,
        withdrawable,
    })?)
}
