use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, to_binary, CosmosMsg, Env, SubMsg, Uint128, WasmMsg};
use pylon_gateway::pool_msg::ExecuteMsg as PoolExecuteMsg;

use crate::executions::{execute_transfer, ExecuteResult};
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_POOL, TEST_RECIPIENT, TEST_SENDER};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    owner: &str,
    recipient: &str,
    amount: u128,
) -> ExecuteResult {
    execute_transfer(
        deps.as_mut(),
        env,
        mock_info(owner, &[]),
        recipient.to_string(),
        Uint128::from(amount),
    )
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    const TRANSFER_AMOUNT: u128 = 1000;

    let res = exec(
        &mut deps,
        mock_env(),
        TEST_SENDER,
        TEST_RECIPIENT,
        TRANSFER_AMOUNT,
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
            }))
        ]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "transfer"),
            attr("from", TEST_SENDER.to_string()),
            attr("to", TEST_RECIPIENT.to_string()),
            attr("amount", TRANSFER_AMOUNT.to_string())
        ]
    );
}
