#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw20::{
    AllAccountsResponse, BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, DownloadLogoResponse,
    MarketingInfoResponse, MinterResponse, TokenInfoResponse,
};
use cw20_base::allowances::{
    execute_decrease_allowance, execute_increase_allowance, query_allowance,
};
use cw20_base::enumerable::query_all_allowances;
use cw20_base::state::{TokenInfo, TOKEN_INFO};
use cw20_base::ContractError;
use pylon_gateway::pool_token_msg::{InstantiateMsg, MigrateMsg};

use crate::executions;
use crate::querier::Querier;
use crate::states::Config;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let querier = Querier::new(&deps.querier);

    let pool_addr = deps.api.addr_validate(msg.pool.as_str())?;
    let pool_config = querier.load_pool_config(&pool_addr)?;

    let reward_token_info =
        querier.load_token_info(&deps.api.addr_validate(pool_config.reward_token.as_str())?)?;

    let months = (pool_config.finish_time - pool_config.start_time) / (30 * 86400);

    TOKEN_INFO.save(
        deps.storage,
        &TokenInfo {
            name: format!(
                "Pylon bDP Token for Gateway {} {}m Pool",
                reward_token_info.symbol, months
            ),
            symbol: format!("b{}DP-{}m", reward_token_info.symbol, months),
            decimals: 6,
            total_supply: Uint128::zero(),
            mint: None,
        },
    )?;

    Config::save(deps.storage, &Config { pool: pool_addr })?;

    Ok(Response::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        Cw20ExecuteMsg::Transfer { recipient, amount } => {
            executions::execute_transfer(deps, env, info, recipient, amount)
        }
        Cw20ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => executions::execute_transfer_from(deps, env, info, owner, recipient, amount),
        Cw20ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => executions::execute_send(deps, env, info, contract, amount, msg),
        Cw20ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => executions::execute_send_from(deps, env, info, owner, contract, amount, msg),
        Cw20ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        Cw20ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
        // not used
        Cw20ExecuteMsg::Mint { .. } => Err(ContractError::Unauthorized {}),
        Cw20ExecuteMsg::Burn { .. } => Err(ContractError::Unauthorized {}),
        Cw20ExecuteMsg::BurnFrom { .. } => Err(ContractError::Unauthorized {}),
        Cw20ExecuteMsg::UpdateMarketing { .. } => Err(ContractError::Unauthorized {}),
        Cw20ExecuteMsg::UploadLogo(_) => Err(ContractError::Unauthorized {}),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: Cw20QueryMsg) -> StdResult<Binary> {
    let querier = Querier::new(&deps.querier);

    match msg {
        Cw20QueryMsg::Balance { address } => {
            let config = Config::load(deps.storage)?;
            let balance = querier
                .load_pool_balance(&config.pool, &deps.api.addr_validate(address.as_str())?)?;

            to_binary(&BalanceResponse { balance })
        }
        Cw20QueryMsg::TokenInfo {} => {
            let config = Config::load(deps.storage)?;
            let pool_reward = querier.load_pool_reward(&config.pool)?;

            let mut token_info = TOKEN_INFO.load(deps.storage)?;
            token_info.total_supply = Uint128::from(pool_reward.total_deposit);

            to_binary(&TokenInfoResponse {
                name: token_info.name,
                symbol: token_info.symbol,
                decimals: token_info.decimals,
                total_supply: token_info.total_supply,
            })
        }
        Cw20QueryMsg::Allowance { owner, spender } => {
            Ok(to_binary(&query_allowance(deps, owner, spender)?)?)
        }
        Cw20QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => Ok(to_binary(&query_all_allowances(
            deps,
            owner,
            start_after,
            limit,
        )?)?),
        Cw20QueryMsg::AllAccounts { start_after, limit } => {
            let config = Config::load(deps.storage)?;
            let pool_stakers = querier.load_pool_stakers(&config.pool, start_after, limit)?;

            to_binary(&AllAccountsResponse {
                accounts: pool_stakers
                    .stakers
                    .iter()
                    .map(|staker| staker.address.to_string())
                    .collect(),
            })
        }
        // not used
        Cw20QueryMsg::Minter {} => to_binary(&Option::<MinterResponse>::None),
        Cw20QueryMsg::MarketingInfo { .. } => to_binary(&Option::<MarketingInfoResponse>::None),
        Cw20QueryMsg::DownloadLogo { .. } => to_binary(&Option::<DownloadLogoResponse>::None),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
