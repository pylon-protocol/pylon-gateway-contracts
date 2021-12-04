use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, to_binary, Api, CosmosMsg, Env, MessageInfo, Response, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::executions::staking::withdraw;
use crate::executions::ExecuteResult;
use crate::states::reward::Reward;
use crate::states::user::User;
use crate::testing::{
    instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_SHARE_TOKEN, TEST_STAKER_1,
};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    sender: &str,
    amount: u128,
) -> ExecuteResult {
    withdraw(
        deps.as_mut(),
        env,
        info,
        sender.to_string(),
        Uint128::from(amount),
    )
}

pub fn default(deps: &mut MockDeps, sender: &str, amount: u128) -> (Env, MessageInfo, Response) {
    let default_msg = instantiate::default_msg();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.reward_distribution_time.finish);
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = exec(deps, env.clone(), info.clone(), sender, amount).unwrap();

    (env, info, res)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    const WITHDRAW_AMOUNT: u128 = 1000;

    let mut reward = Reward::load(deps.as_ref().storage).unwrap();
    reward.total_deposit = Uint128::from(WITHDRAW_AMOUNT);
    Reward::save(deps.as_mut().storage, &reward).unwrap();

    let user_addr = deps.api.addr_canonicalize(TEST_STAKER_1).unwrap();
    let mut user = User::load(deps.as_ref().storage, &user_addr);
    user.amount = Uint128::from(WITHDRAW_AMOUNT);
    User::save(deps.as_mut().storage, &user_addr, &user).unwrap();

    let (_, _, res) = default(&mut deps, TEST_STAKER_1, WITHDRAW_AMOUNT);
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_SHARE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_STAKER_1.to_string(),
                amount: Uint128::from(WITHDRAW_AMOUNT)
            })
            .unwrap(),
            funds: vec![]
        }))]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "withdraw"),
            attr("sender", TEST_STAKER_1),
            attr("withdraw_amount", WITHDRAW_AMOUNT.to_string())
        ]
    );
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    let default_msg = instantiate::default_msg();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.reward_distribution_time.finish);

    match exec(
        &mut deps,
        env,
        mock_info(TEST_OWNER, &[]),
        TEST_STAKER_1,
        1000,
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized { .. }) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fail_withdraw_amount_exceeded() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    let default_msg = instantiate::default_msg();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.reward_distribution_time.finish);

    match exec(
        &mut deps,
        env,
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_STAKER_1,
        1000,
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::WithdrawAmountExceeded { .. }) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
