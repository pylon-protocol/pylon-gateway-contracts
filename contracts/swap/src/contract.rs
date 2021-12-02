#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;
use cw20::Denom;
use pylon_gateway::swap_msg::{ConfigureMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION};
use crate::error::ContractError;
use crate::states::config::Config;
use crate::states::state::State;
use crate::types::cap_strategy::CapStrategy;
use crate::types::distribution_strategy::DistributionStrategy;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let api = deps.api;
    Config::save(
        deps.storage,
        &Config {
            owner: info.sender,
            beneficiary: api.addr_validate(msg.beneficiary.as_str())?,
            start: msg.start,
            finish: msg.start + msg.period,
            price: msg.price,
            amount: msg.swap_pool_size,
            input_token: Denom::Native(msg.pool_x_denom),
            output_token: Denom::Cw20(api.addr_validate(msg.pool_y_addr.as_str())?),
            deposit_cap_strategy: msg.deposit_cap_strategy.map(CapStrategy::from),
            distribution_strategies: msg
                .distribution_strategies
                .iter()
                .map(DistributionStrategy::from)
                .collect(),
            whitelist_enabled: msg.whitelist_enabled,
        },
    )?;

    State::save(
        deps.storage,
        &State {
            total_swapped: Uint128::zero(),
            total_claimed: Uint128::zero(),
            x_liquidity: msg.x_liquidity,
            y_liquidity: msg.y_liquidity,
        },
    )?;

    Ok(Response::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Configure(cfg_msg) => {
            let config = config::read(deps.storage).load().unwrap();
            if config.owner != info.sender {
                return Err(ContractError::Unauthorized {
                    action: stringify!(cfg_msg).to_string(),
                    expected: config.owner,
                    actual: info.sender.to_string(),
                });
            }

            match cfg_msg {
                ConfigureMsg::Swap {
                    owner,
                    beneficiary,
                    cap_strategy,
                    whitelist_enabled,
                } => ConfigHandler::swap(
                    deps,
                    env,
                    info,
                    owner,
                    beneficiary,
                    cap_strategy,
                    whitelist_enabled,
                ),
                ConfigureMsg::Pool {
                    x_denom,
                    y_addr,
                    liq_x,
                    liq_y,
                } => ConfigHandler::pool(deps, env, info, x_denom, y_addr, liq_x, liq_y),
                ConfigureMsg::Whitelist {
                    whitelist,
                    candidates,
                } => ConfigHandler::whitelist(deps, env, info, whitelist, candidates),
            }
        }
        ExecuteMsg::Deposit {} => ExecHandler::deposit(deps, env, info),
        ExecuteMsg::Withdraw { amount } => ExecHandler::withdraw(deps, env, info, amount),
        ExecuteMsg::Claim {} => ExecHandler::claim(deps, env, info),
        ExecuteMsg::Earn {} => ExecHandler::earn(deps, env, info),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::BalanceOf { owner } => QueryHandler::balance_of(deps, owner),
        QueryMsg::IsWhitelisted { address } => QueryHandler::is_whitelisted(deps, address),
        QueryMsg::AvailableCapOf { address } => QueryHandler::available_cap_of(deps, address),
        QueryMsg::ClaimableTokenOf { address } => {
            QueryHandler::claimable_token_of(deps, env, address)
        }
        QueryMsg::TotalSupply {} => QueryHandler::total_supply(deps),
        QueryMsg::CurrentPrice {} => QueryHandler::current_price(deps),
        QueryMsg::SimulateWithdraw { amount, address } => {
            QueryHandler::simulate_withdraw(deps, address, amount)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
