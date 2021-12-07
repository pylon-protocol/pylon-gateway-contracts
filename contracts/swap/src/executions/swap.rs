use cosmwasm_std::{
    attr, to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, Fraction, MessageInfo,
    Response, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Denom};
use pylon_utils::tax::deduct_tax;
use std::convert::TryFrom;

use crate::error::ContractError;
use crate::states::config::Config;
use crate::states::state::State;
use crate::states::user::User;

pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> super::ExecuteResult {
    let config = Config::load(deps.storage)?;
    let now = env.block.time.seconds();

    if now < config.start {
        return Err(ContractError::SwapNotStarted {
            start: config.start,
        });
    }
    if config.finish < now {
        return Err(ContractError::SwapFinished {
            finish: config.finish,
        });
    }

    let input_token_denom = match config.input_token {
        Denom::Native(denom) => denom,
        Denom::Cw20(_) => panic!("input token as cw20 token not supported"),
    };

    // 1:1
    let swapped_in = info
        .funds
        .iter()
        .find(|c| c.denom == input_token_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);
    if swapped_in.is_zero() {
        return Err(ContractError::NotAllowZeroAmount {});
    }
    if info.funds.len() > 1 {
        return Err(ContractError::NotAllowOtherDenoms {
            denom: input_token_denom,
        });
    }

    let sender = &deps.api.addr_canonicalize(info.sender.as_str())?;
    let mut user = User::load(deps.storage, sender);
    let mut state = State::load(deps.storage)?;

    // check whitelisted, or free to participate everyone
    if config.whitelist_enabled && !User::is_whitelisted(deps.storage, sender) {
        return Err(ContractError::NotAllowNonWhitelisted {
            address: info.sender.to_string(),
        });
    }

    if let Some(strategy) = config.deposit_cap_strategy {
        let (amount, unlimited) =
            strategy.available_cap_of(deps.querier, info.sender.to_string(), user.swapped_in);
        if !unlimited && swapped_in > amount {
            return Err(ContractError::AvailableCapExceeded { available: amount });
        }
    }

    let swapped_out = swapped_in * Uint128::from(config.price.denominator())
        / Uint128::from(config.price.numerator());
    if state.total_swapped + swapped_out > config.amount {
        return Err(ContractError::PoolSizeExceeded {
            available: config.amount - state.total_swapped,
        });
    }

    user.swapped_in += swapped_in;
    user.swapped_out += swapped_out;

    state.total_swapped += swapped_out;

    User::save(deps.storage, sender, &user)?;
    State::save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "deposit"),
        attr("sender", info.sender.to_string()),
        attr("swapped_in", swapped_in.to_string()),
        attr("swapped_out", swapped_out.to_string()),
    ]))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> super::ExecuteResult {
    // xyk
    let config = Config::load(deps.storage)?;
    let now = env.block.time.seconds();
    if config
        .distribution_strategies
        .iter()
        .all(|strategy| !strategy.check_release_time(&now))
    {
        return Err(ContractError::NotAllowWithdrawAfterRelease {});
    }

    let sender = &deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    let mut user = User::load(deps.storage, sender);
    let mut state = State::load(deps.storage)?;

    if !user.swapped_out_claimed.is_zero() {
        return Err(ContractError::NotAllowWithdrawAfterClaim {});
    }

    if user.swapped_in * Uint128::from(config.price.denominator())
        / Uint128::from(config.price.numerator())
        < amount
    {
        return Err(ContractError::WithdrawAmountExceeded {
            available: user.swapped_in,
        });
    }

    let withdraw_amount = calculate_withdraw_amount(&state, &amount);
    let penalty = (amount * config.price) - withdraw_amount;

    user.swapped_out -= amount;
    user.swapped_in -= amount * config.price;

    state.total_swapped -= amount;
    state.x_liquidity -= withdraw_amount;
    state.y_liquidity += amount;

    User::save(deps.storage, sender, &user)?;
    State::save(deps.storage, &state)?;

    let input_token = match config.input_token {
        Denom::Native(input_token) => input_token,
        Denom::Cw20(_) => unreachable!("cw20 as input_token is not supported"),
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: input_token.clone(),
                    amount: withdraw_amount,
                },
            )?],
        }))
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary.to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: input_token,
                    amount: penalty,
                },
            )?],
        }))
        .add_attributes(vec![
            attr("action", "withdraw"),
            attr("sender", info.sender.to_string()),
            attr("amount", withdraw_amount.to_string()),
            attr("penalty", penalty.to_string()),
        ]))
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> super::ExecuteResult {
    let config = Config::load(deps.storage)?;

    let sender = &deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    let mut state = State::load(deps.storage)?;
    let mut user = User::load(deps.storage, sender);

    let claimable_token = calculate_claimable_tokens(&config, &user, env.block.time.seconds());

    user.swapped_out_claimed += claimable_token;

    state.total_claimed += claimable_token;

    User::save(deps.storage, sender, &user)?;
    State::save(deps.storage, &state)?;

    let output_token = match config.output_token {
        Denom::Native(_) => unreachable!("native as output_token is not supported"),
        Denom::Cw20(output_token) => output_token,
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: output_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: claimable_token,
            })?,
            funds: vec![],
        }))
        .add_attributes(vec![
            attr("action", "claim"),
            attr("sender", info.sender.to_string()),
            attr("amount", claimable_token.to_string()),
        ]))
}

const EARN_LOCK_PERIOD: u64 = 86400 * 7;

pub fn earn(deps: DepsMut, env: Env, info: MessageInfo) -> super::ExecuteResult {
    let config = Config::load(deps.storage)?;
    if config.beneficiary != info.sender {
        return Err(ContractError::Unauthorized {
            action: "earn".to_string(),
            expected: config.beneficiary.to_string(),
            actual: info.sender.to_string(),
        });
    }

    if env.block.time.seconds() < config.finish + EARN_LOCK_PERIOD {
        return Err(ContractError::NotAllowEarnBeforeLockPeriod {});
    }

    let input_token = match config.input_token {
        Denom::Native(input_token) => input_token,
        Denom::Cw20(_) => unreachable!("cw20 as input_token is not supported"),
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary.to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                deps.querier
                    .query_balance(env.contract.address, input_token)
                    .unwrap(),
            )?],
        }))
        .add_attribute("action", "earn")
        .add_attribute("sender", info.sender.to_string()))
}

pub fn calculate_withdraw_amount(state: &State, dy: &Uint128) -> Uint128 {
    let k = state.x_liquidity * state.y_liquidity;
    state.x_liquidity - (k / (state.y_liquidity + *dy))
}

pub fn calculate_current_price(state: &State) -> Decimal {
    Decimal::from_ratio(state.x_liquidity, state.y_liquidity)
}

pub fn calculate_claimable_tokens(config: &Config, user: &User, time: u64) -> Uint128 {
    let (count, mut ratio) = config.distribution_strategies.iter().fold(
        (0u64, Decimal::zero()),
        |(count, ratio), strategy| {
            let (release_amount, fulfilled) = strategy.release_amount_at(&time);
            (
                count + if fulfilled { 1 } else { 0 },
                ratio + release_amount,
            )
        },
    );
    if u64::try_from(config.distribution_strategies.len()).unwrap() == count {
        ratio = Decimal::one();
    }

    (user.swapped_out * ratio) - user.swapped_out_claimed
}
