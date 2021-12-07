#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, Uint128};
use cw2::set_contract_version;
use cw20::Denom;
use pylon_gateway::swap_msg::{ConfigureMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION};
use crate::error::ContractError;
use crate::states::config::Config;
use crate::states::state::State;
use crate::types::cap_strategy::CapStrategy;
use crate::types::distribution_strategy::DistributionStrategy;
use crate::{executions, migrations, queries};

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
            amount: msg.amount,
            input_token: Denom::Native(msg.input_token),
            output_token: Denom::Cw20(api.addr_validate(msg.output_token.as_str())?),
            deposit_cap_strategy: msg.deposit_cap_strategy.map(CapStrategy::from),
            distribution_strategies: msg
                .distribution_strategies
                .iter()
                .map(|x| DistributionStrategy::from(x.clone()))
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
            let config = Config::load(deps.storage)?;
            if config.owner != info.sender {
                return Err(ContractError::Unauthorized {
                    action: match *cfg_msg {
                        ConfigureMsg::Config { .. } => "update_config",
                        ConfigureMsg::State { .. } => "update_state",
                        ConfigureMsg::Whitelist { .. } => "whitelist",
                    }
                    .to_string(),
                    expected: config.owner.to_string(),
                    actual: info.sender.to_string(),
                });
            }

            match *cfg_msg {
                ConfigureMsg::Config {
                    owner,
                    beneficiary,
                    input_token,
                    output_token,
                    deposit_cap_strategy,
                    distribution_strategies,
                    whitelist_enabled,
                } => executions::config::update(
                    deps,
                    env,
                    info,
                    owner,
                    beneficiary,
                    input_token,
                    output_token,
                    deposit_cap_strategy,
                    distribution_strategies,
                    whitelist_enabled,
                ),
                ConfigureMsg::State {
                    x_liquidity,
                    y_liquidity,
                } => executions::state::update(deps, env, info, x_liquidity, y_liquidity),
                ConfigureMsg::Whitelist {
                    whitelist,
                    candidates,
                } => executions::user::whitelist(deps, env, info, whitelist, candidates),
            }
        }
        ExecuteMsg::Deposit {} => executions::swap::deposit(deps, env, info),
        ExecuteMsg::Withdraw { amount } => executions::swap::withdraw(deps, env, info, amount),
        ExecuteMsg::Claim {} => executions::swap::claim(deps, env, info),
        ExecuteMsg::Earn {} => executions::swap::earn(deps, env, info),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => queries::config::query_config(deps, env),
        QueryMsg::ConfigV2 {} => queries::config::query_config_v2(deps, env),
        QueryMsg::State {} => queries::state::query_state(deps, env),
        QueryMsg::User { address } => queries::user::query_user(deps, env, address),
        QueryMsg::Users {
            start_after,
            limit,
            order,
        } => queries::user::query_users(deps, env, start_after, limit, order),
        QueryMsg::CurrentPrice {} => queries::swap::query_current_price(deps),
        QueryMsg::SimulateWithdraw { amount, address } => {
            queries::swap::query_simulate_withdraw(deps, address, amount)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::Pylon {} => migrations::pylon::migrate(deps, env),
        MigrateMsg::Nexus {
            deposit_cap_strategy,
        } => migrations::nexus::migrate(deps, env, deposit_cap_strategy),
        MigrateMsg::Valkyrie {
            deposit_cap_strategy,
        } => migrations::valkyrie::migrate(deps, env, deposit_cap_strategy),
        MigrateMsg::General {} => Ok(Response::default()),
    }
}
