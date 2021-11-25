#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdError, StdResult, SubMsg, WasmMsg,
};
use protobuf::Message;
use pylon_gateway::pool_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use pylon_gateway::pool_token_msg::InstantiateMsg as PoolTokenInitMsg;
use pylon_gateway::time_range::TimeRange;
use std::ops::Add;

use crate::error::ContractError;
use crate::handler::configure as Config;
use crate::handler::core as Core;
use crate::handler::query as Query;
use crate::handler::receive;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{config, reward};

const INSTANTIATE_REPLY_ID: u64 = 1;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    config::store(
        deps.storage,
        &config::Config {
            owner: info.sender.to_string(),
            token: "".to_string(),
            // share
            share_token: api.addr_validate(msg.share_token.as_str())?.to_string(),
            deposit_config: config::DepositConfig {
                time: TimeRange {
                    start: msg.start,
                    finish: 0,
                    inverse: false,
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero(),
            },
            withdraw_time: vec![TimeRange {
                start: msg.start,
                finish: msg.start.add(msg.period),
                inverse: true,
            }],
            // reward
            reward_token: api.addr_validate(msg.reward_token.as_str())?.to_string(),
            claim_time: TimeRange {
                start: msg.start.add(msg.cliff),
                finish: 0,
                inverse: false,
            },
            distribution_config: config::DistributionConfig {
                time: TimeRange {
                    start: msg.start,
                    finish: msg.start.add(msg.period),
                    inverse: false,
                },
                reward_rate: Decimal256::from_ratio(msg.reward_amount, Uint256::from(msg.period)),
            },
            cap_strategy: msg
                .cap_strategy
                .map(|x| api.addr_validate(x.as_str()).unwrap().to_string()),
        },
    )?;

    reward::store(
        deps.storage,
        &reward::Reward {
            total_deposit: Uint256::zero(),
            last_update_time: msg.start,
            reward_per_token_stored: Decimal256::zero(),
        },
    )?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create DP token
        msg: WasmMsg::Instantiate {
            admin: None,
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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Update { target } => Core::update(deps, env, info, target),
        ExecuteMsg::Receive(msg) => receive(deps, env, info, msg),
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
            Core::deposit_internal(deps, env, info, sender, amount)
        }
        ExecuteMsg::WithdrawInternal { sender, amount } => {
            Core::withdraw_internal(deps, env, info, sender, amount)
        }
        ExecuteMsg::ClaimInternal { sender } => Core::claim_internal(deps, env, info, sender),
        ExecuteMsg::TransferInternal {
            owner,
            recipient,
            amount,
        } => Core::transfer_internal(deps, env, info, owner, recipient, Uint256::from(amount)),
        // owner
        ExecuteMsg::Configure(msg) => Config::configure(deps, env, info, msg),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
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

            let mut config = config::read(deps.storage)?;
            config.token = token_addr.to_string();
            config::store(deps.storage, &config)?;

            Ok(Response::new()
                .add_attribute("action", "register_token")
                .add_attribute("token", token_addr.to_string()))
        }
        _ => Err(ContractError::InvalidReplyId { id: msg.id }),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Query::config(deps, env),
        QueryMsg::Stakers { start_after, limit } => Query::stakers(deps, env, start_after, limit),
        QueryMsg::Reward {} => Query::reward(deps, env),
        QueryMsg::BalanceOf { owner } => Query::balance_of(deps, env, owner),
        QueryMsg::ClaimableReward { owner } => Query::claimable_reward(deps, env, owner),
        QueryMsg::AvailableCapOf { address } => Query::available_cap_of(deps, env, address),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
