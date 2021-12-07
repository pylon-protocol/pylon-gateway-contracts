use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, coins, BankMsg, CosmosMsg, Env, MessageInfo, SubMsg, Timestamp};
use pylon_gateway::swap_msg::ExecuteMsg;

use crate::constants::EARN_LOCK_PERIOD;
use crate::entrypoints::execute;
use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::testing::{
    instantiate, mock_deps, mock_deps_with_balance, MockDeps, TEST_BENEFICIARY, TEST_OWNER,
};

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo) -> ExecuteResult {
    execute(deps.as_mut(), env, info, ExecuteMsg::Earn {})
}

#[test]
fn success() {
    const SWAP_AMOUNT: u128 = 100000;
    let default_msg = instantiate::default_msg();
    let mut deps = mock_deps_with_balance(&coins(SWAP_AMOUNT, default_msg.input_token.clone()));
    instantiate::exec(
        &mut deps,
        mock_env(),
        mock_info(
            TEST_OWNER,
            &coins(SWAP_AMOUNT, default_msg.input_token.clone()),
        ),
        default_msg.clone(),
    )
    .unwrap();

    let mut env = mock_env();
    env.block.time =
        Timestamp::from_seconds(default_msg.start + default_msg.period + EARN_LOCK_PERIOD);
    let res = exec(&mut deps, env, mock_info(TEST_BENEFICIARY, &[])).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: TEST_BENEFICIARY.to_string(),
            amount: coins(SWAP_AMOUNT, default_msg.input_token)
        }))]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "earn"),
            attr("sender", TEST_BENEFICIARY.to_string())
        ]
    );
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    match exec(&mut deps, mock_env(), mock_info(TEST_OWNER, &[])) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {
            action,
            expected,
            actual,
        }) => assert_eq!(
            (action, expected, actual),
            (
                "earn".to_string(),
                TEST_BENEFICIARY.to_string(),
                TEST_OWNER.to_string()
            )
        ),
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_not_allow_earn_before_lock_period() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    match exec(&mut deps, mock_env(), mock_info(TEST_BENEFICIARY, &[])) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NotAllowEarnBeforeLockPeriod {}) => (),
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}
