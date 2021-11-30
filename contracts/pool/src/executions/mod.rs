use cosmwasm_std::{
    from_binary, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use pylon_gateway::pool_msg::{Cw20HookMsg, ExecuteMsg};

use crate::error::ContractError;
use crate::states::config::Config;

pub mod config;
pub mod staking;

pub type ExecuteResult = Result<Response, ContractError>;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ExecuteResult {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => {
            let config = Config::load(deps.storage)?;
            if config.share_token != info.sender {
                return Err(ContractError::Unauthorized {
                    action: "deposit".to_string(),
                    expected: config.share_token.to_string(),
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
                        amount: cw20_msg.amount,
                    })?,
                    funds: vec![],
                })))
        }
        _ => Err(ContractError::UnsupportedReceiveMsg {
            typ: stringify!(cw20_msg).to_string(),
        }),
    }
}
