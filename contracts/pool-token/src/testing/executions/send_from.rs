use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, to_binary, Api, Binary, CosmosMsg, Env, MessageInfo, SubMsg, Uint128, WasmMsg,
};
use cw20::{AllowanceResponse, Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration};
use cw20_base::allowances::execute_increase_allowance;
use cw20_base::state::ALLOWANCES;
use pylon_gateway::pool_msg::ExecuteMsg as PoolExecuteMsg;

use crate::executions::{execute_send_from, ExecuteResult};
use crate::testing::{
    instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_POOL, TEST_RECIPIENT, TEST_REWARD_TOKEN,
    TEST_SENDER,
};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    owner: &str,
    contract: &str,
    amount: u128,
    msg: Binary,
) -> ExecuteResult {
    execute_send_from(
        deps.as_mut(),
        env,
        info,
        owner.to_string(),
        contract.to_string(),
        Uint128::from(amount),
        msg,
    )
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    execute_increase_allowance(
        deps.as_mut(),
        mock_env(),
        mock_info(TEST_SENDER, &[]),
        TEST_OWNER.to_string(),
        Uint128::from(TRANSFER_AMOUNT),
        None,
    )
    .unwrap();

    const TRANSFER_AMOUNT: u128 = 1000;

    let payload = to_binary(&Cw20ExecuteMsg::Mint {
        recipient: TEST_REWARD_TOKEN.to_string(),
        amount: Uint128::from(TRANSFER_AMOUNT),
    })
    .unwrap();
    let res = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        TEST_SENDER,
        TEST_RECIPIENT,
        TRANSFER_AMOUNT,
        payload.clone(),
    )
    .unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_POOL.to_string(),
                msg: to_binary(&PoolExecuteMsg::Update {
                    target: Some(TEST_SENDER.to_string())
                })
                .unwrap(),
                funds: vec![]
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_POOL.to_string(),
                msg: to_binary(&PoolExecuteMsg::Update {
                    target: Some(TEST_RECIPIENT.to_string())
                })
                .unwrap(),
                funds: vec![]
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_POOL.to_string(),
                msg: to_binary(&PoolExecuteMsg::TransferInternal {
                    owner: TEST_SENDER.to_string(),
                    recipient: TEST_RECIPIENT.to_string(),
                    amount: Uint128::from(TRANSFER_AMOUNT)
                })
                .unwrap(),
                funds: vec![]
            })),
            SubMsg::new(
                Cw20ReceiveMsg {
                    sender: TEST_OWNER.to_string(),
                    amount: Uint128::from(TRANSFER_AMOUNT),
                    msg: payload
                }
                .into_cosmos_msg(TEST_RECIPIENT)
                .unwrap()
            )
        ]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "send_from"),
            attr("from", TEST_SENDER.to_string()),
            attr("to", TEST_RECIPIENT.to_string()),
            attr("by", TEST_OWNER.to_string()),
            attr("amount", TRANSFER_AMOUNT.to_string()),
        ]
    );

    let api = deps.api;
    assert_eq!(
        ALLOWANCES
            .load(
                deps.as_ref().storage,
                (
                    &api.addr_validate(TEST_SENDER).unwrap(),
                    &api.addr_validate(TEST_OWNER).unwrap()
                )
            )
            .unwrap(),
        AllowanceResponse {
            allowance: Uint128::zero(),
            expires: Expiration::Never {}
        }
    )
}
