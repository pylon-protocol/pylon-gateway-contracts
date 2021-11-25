use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    from_binary, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use pylon_gateway::pool_msg::{Cw20HookMsg, ExecuteMsg};

use crate::error::ContractError;
use crate::state::config;

pub mod configure;
pub mod core;
pub mod query;
mod util_staking;

pub type ExecuteResult = Result<Response, ContractError>;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => {
            let config = config::read(deps.storage)?;
            if config.share_token.ne(&info.sender.to_string()) {
                return Err(ContractError::Unauthorized {
                    action: "deposit".to_string(),
                    expected: config.share_token,
                    actual: info.sender.to_string(),
                });
            }

            Ok(Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::Update {
                        target: Option::Some(cw20_msg.sender.clone()),
                    })?,
                    funds: vec![],
                }))
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::DepositInternal {
                        sender: cw20_msg.sender,
                        amount: Uint256::from(cw20_msg.amount),
                    })?,
                    funds: vec![],
                })))
        }
        _ => Err(ContractError::UnsupportedReceiveMsg {
            typ: stringify!(cw20_msg).to_string(),
        }),
    }
}
