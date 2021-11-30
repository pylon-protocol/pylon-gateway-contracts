use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, to_binary, Api, CosmosMsg, Env, MessageInfo, Response, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::executions::staking::claim;
use crate::executions::ExecuteResult;
use crate::states::user::User;
use crate::testing::{
    instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_REWARD_TOKEN, TEST_STAKER_1,
};

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo, sender: &str) -> ExecuteResult {
    claim(deps.as_mut(), env, info, sender.to_string())
}

pub fn default(deps: &mut MockDeps, sender: &str) -> (Env, MessageInfo, Response) {
    let default_msg = instantiate::default_msg();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.reward_distribution_time.0 + 75);
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = exec(deps, env.clone(), info.clone(), sender).unwrap();

    (env, info, res)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    const CLAIM_AMOUNT: u128 = 1000;

    let user_addr = deps.api.addr_canonicalize(TEST_STAKER_1).unwrap();
    let mut user = User::load(deps.as_ref().storage, &user_addr);
    user.reward = Uint128::from(CLAIM_AMOUNT);
    User::save(deps.as_mut().storage, &user_addr, &user).unwrap();

    let (_, _, res) = default(&mut deps, TEST_STAKER_1);
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_REWARD_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_STAKER_1.to_string(),
                amount: Uint128::from(CLAIM_AMOUNT)
            })
            .unwrap(),
            funds: vec![]
        }))]
    );
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("sender", TEST_STAKER_1.to_string()),
            attr("claim_amount", Uint128::from(CLAIM_AMOUNT).to_string()),
        ]
    );

    assert_eq!(
        User::load(deps.as_ref().storage, &user_addr).amount,
        Uint128::zero(),
    );
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        TEST_STAKER_1,
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized { .. }) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
