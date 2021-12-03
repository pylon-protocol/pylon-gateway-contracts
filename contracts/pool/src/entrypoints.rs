#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdError, SubMsg, Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use protobuf::Message;
use pylon_gateway::pool_msg::{ConfigureMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use pylon_gateway::pool_token_msg::InstantiateMsg as PoolTokenInitMsg;

use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION, INSTANTIATE_REPLY_ID};
use crate::error::ContractError;
use crate::querier::Querier;
use crate::response::MsgInstantiateContractResponse;
use crate::states::config::Config;
use crate::states::reward::Reward;
use crate::{executions, migrations, queries};

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> executions::ExecuteResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let api = deps.api;
    let querier = Querier::new(&deps.querier);
    let reward_token_addr = deps.api.addr_validate(msg.reward_token.as_str())?;
    let reward_token_info = querier.load_token_info(&reward_token_addr)?;
    let reward_rate = Decimal::from_ratio(
        msg.reward_amount * Uint128::from(10u128.pow(u32::from(reward_token_info.decimals))),
        msg.reward_distribution_time.period(),
    );

    Config::save(
        deps.storage,
        &Config {
            owner: info.sender.clone(),
            token: Addr::unchecked("".to_string()),
            share_token: api.addr_validate(msg.share_token.as_str())?,
            deposit_time: msg.deposit_time,
            withdraw_time: msg.withdraw_time,
            deposit_cap_strategy: msg
                .deposit_cap_strategy
                .map(|x| api.addr_validate(x.as_str()).unwrap()),
            reward_token: api.addr_validate(msg.reward_token.as_str())?,
            reward_rate,
            reward_claim_time: msg.reward_claim_time,
            reward_distribution_time: msg.reward_distribution_time.clone(),
        },
    )?;

    Reward::save(
        deps.storage,
        &Reward {
            total_deposit: Uint128::zero(),
            last_update_time: msg.reward_distribution_time.start,
            reward_per_token_stored: Decimal::zero(),
        },
    )?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create DP token
        msg: WasmMsg::Instantiate {
            admin: Some(info.sender.to_string()),
            code_id: msg.pool_token_code_id,
            funds: vec![],
            label: "".to_string(),
            msg: to_binary(&PoolTokenInitMsg {
                pool: env.contract.address.to_string(),
            })?,
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> executions::ExecuteResult {
    match msg {
        ExecuteMsg::Update { target } => executions::staking::update(deps, env, info, target),
        ExecuteMsg::Receive(msg) => executions::receive(deps, env, info, msg),
        ExecuteMsg::Withdraw { amount } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Update {
                    target: Option::Some(info.sender.to_string()),
                })?,
                funds: vec![],
            }))
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::WithdrawInternal {
                    sender: info.sender.to_string(),
                    amount,
                })?,
                funds: vec![],
            }))),
        ExecuteMsg::Claim { target } => Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Update {
                    target: Option::Some(target.clone().unwrap_or_else(|| info.sender.to_string())),
                })?,
                funds: vec![],
            }))
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::ClaimInternal {
                    sender: target.unwrap_or_else(|| info.sender.to_string()),
                })?,
                funds: vec![],
            }))),
        // internal
        ExecuteMsg::DepositInternal { sender, amount } => {
            executions::staking::deposit(deps, env, info, sender, amount)
        }
        ExecuteMsg::WithdrawInternal { sender, amount } => {
            executions::staking::withdraw(deps, env, info, sender, amount)
        }
        ExecuteMsg::ClaimInternal { sender } => executions::staking::claim(deps, env, info, sender),
        ExecuteMsg::TransferInternal {
            owner,
            recipient,
            amount,
        } => executions::staking::transfer(deps, env, info, owner, recipient, amount),
        // owner
        ExecuteMsg::Configure(msg) => {
            let config = Config::load(deps.storage)?;
            if config.owner != info.sender {
                return Err(ContractError::Unauthorized {
                    action: "configure".to_string(),
                    expected: config.owner.to_string(),
                    actual: info.sender.to_string(),
                });
            }

            match msg {
                ConfigureMsg::Config {
                    owner,
                    share_token,
                    reward_token,
                    claim_time,
                    deposit_time,
                    withdraw_time,
                    deposit_cap_strategy,
                } => executions::config::update(
                    deps,
                    env,
                    info,
                    owner,
                    share_token,
                    reward_token,
                    claim_time,
                    deposit_time,
                    withdraw_time,
                    deposit_cap_strategy,
                ),
                ConfigureMsg::SubReward { amount } => {
                    executions::config::adjust_reward(deps, env, amount, true)
                }
                ConfigureMsg::AddReward { amount } => {
                    executions::config::adjust_reward(deps, env, amount, false)
                }
            }
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> executions::ExecuteResult {
    match msg.id {
        INSTANTIATE_REPLY_ID => {
            // get new token's contract address
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                ContractError::Std(StdError::parse_err(
                    "MsgInstantiateContractResponse",
                    "failed to parse data",
                ))
            })?;
            let token_addr = Addr::unchecked(res.get_contract_address());

            let mut config = Config::load(deps.storage)?;
            if config.token != Addr::unchecked("".to_string()) {
                return Err(ContractError::Unauthorized {
                    action: "register_token".to_string(),
                    expected: "".to_string(),
                    actual: config.token.to_string(),
                });
            }
            config.token = token_addr;
            Config::save(deps.storage, &config)?;

            Ok(Response::new().add_attribute("action", "register_token"))
        }
        _ => Err(ContractError::InvalidReplyId { id: msg.id }),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> queries::QueryResult {
    match msg {
        // v1
        QueryMsg::Config {} => queries::config::query_config(deps, env),
        QueryMsg::BalanceOf { owner } => queries::user::query_balance(deps, env, owner),
        QueryMsg::ClaimableReward { owner, timestamp } => {
            queries::user::query_claimable_reward(deps, env, owner, timestamp)
        }
        QueryMsg::AvailableCapOf { address } => {
            queries::user::query_available_cap(deps, env, address)
        }

        // v2
        QueryMsg::ConfigV2 {} => queries::config::query_config_v2(deps, env),

        // common
        QueryMsg::Reward {} => queries::reward::query_reward(deps, env),
        QueryMsg::Staker { address } => queries::user::query_staker(deps, env, address),
        QueryMsg::Stakers {
            start_after,
            limit,
            order,
        } => queries::user::query_stakers(deps, env, start_after, limit, order),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, _msg: MigrateMsg) -> migrations::MigrateResult {
    match get_contract_version(deps.storage) {
        Ok(ContractVersion { contract, version }) => {
            if contract != CONTRACT_NAME {
                return Err(ContractError::Unauthorized {
                    action: "migrate".to_string(),
                    expected: CONTRACT_NAME.to_string(),
                    actual: contract,
                });
            }

            match version.as_str() {
                "0.1.1" => Ok(Response::new()), // TODO: do it next time
                _ => Err(ContractError::InvalidContractVersionForMigration {}),
            }
        }
        Err(_) => migrations::legacy::migrate(deps, env),
    }
}
