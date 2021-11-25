use cosmwasm_std::{
    attr, to_binary, Addr, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use cw20_base::allowances::deduct_allowance;
use cw20_base::ContractError;
use pylon_gateway::pool_msg::ExecuteMsg as PoolExecuteMsg;

use crate::states::Config;

pub type ExecuteResult = Result<Response, ContractError>;

fn to_transfer_messages(
    pool: &Addr,
    owner: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pool.to_string(),
            msg: to_binary(&PoolExecuteMsg::Update {
                target: Some(owner.to_string()),
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pool.to_string(),
            msg: to_binary(&PoolExecuteMsg::Update {
                target: Some(recipient.to_string()),
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pool.to_string(),
            msg: to_binary(&PoolExecuteMsg::TransferInternal {
                owner: owner.to_string(),
                recipient: recipient.to_string(),
                amount,
            })?,
            funds: vec![],
        }),
    ])
}

pub fn execute_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> ExecuteResult {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let pool_addr = Config::load(deps.storage).unwrap().pool;
    let recipient_addr = deps.api.addr_validate(&recipient)?;

    let mut res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender.to_string())
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);

    // execute transfer
    res = res.add_messages(to_transfer_messages(
        &pool_addr,
        &info.sender,
        &recipient_addr,
        amount,
    )?);

    Ok(res)
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> ExecuteResult {
    let pool_addr = Config::load(deps.storage).unwrap().pool;
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    let mut res = Response::new().add_attributes(vec![
        attr("action", "transfer_from"),
        attr("from", owner),
        attr("to", recipient),
        attr("by", info.sender),
        attr("amount", amount),
    ]);

    // execute transfer
    res = res.add_messages(to_transfer_messages(
        &pool_addr,
        &owner_addr,
        &recipient_addr,
        amount,
    )?);

    Ok(res)
}

pub fn execute_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> ExecuteResult {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let pool_addr = Config::load(deps.storage).unwrap().pool;
    let recipient_addr = deps.api.addr_validate(&contract)?;

    let mut res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &contract)
        .add_attribute("amount", amount);

    // execute transfer
    res = res.add_messages(to_transfer_messages(
        &pool_addr,
        &info.sender,
        &recipient_addr,
        amount,
    )?);

    res = res.add_message(
        Cw20ReceiveMsg {
            sender: info.sender.into(),
            amount,
            msg,
        }
        .into_cosmos_msg(contract)?,
    );

    Ok(res)
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> ExecuteResult {
    let pool_addr = Config::load(deps.storage).unwrap().pool;
    let owner_addr = deps.api.addr_validate(&owner)?;
    let recipient_addr = deps.api.addr_validate(&contract)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    let mut res = Response::new().add_attributes(vec![
        attr("action", "send_from"),
        attr("from", &owner),
        attr("to", &contract),
        attr("by", &info.sender),
        attr("amount", amount),
    ]);

    res = res.add_messages(to_transfer_messages(
        &pool_addr,
        &owner_addr,
        &recipient_addr,
        amount,
    )?);

    // create a send message
    res = res.add_message(
        Cw20ReceiveMsg {
            sender: info.sender.into(),
            amount,
            msg,
        }
        .into_cosmos_msg(contract)?,
    );

    Ok(res)
}
