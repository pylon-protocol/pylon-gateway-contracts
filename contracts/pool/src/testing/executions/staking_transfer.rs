use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Api, Env, MessageInfo, Uint128};

use crate::error::ContractError;
use crate::executions::staking::transfer;
use crate::executions::ExecuteResult;
use crate::states::user::User;
use crate::testing::{
    instantiate, mock_deps, reply, MockDeps, TEST_STAKER_1, TEST_STAKER_2, TEST_TOKEN,
};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    owner: &str,
    recipient: &str,
    amount: u128,
) -> ExecuteResult {
    transfer(
        deps.as_mut(),
        env,
        info,
        owner.to_string(),
        recipient.to_string(),
        Uint128::from(amount),
    )
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    reply::default(&mut deps);

    super::staking_deposit::default(&mut deps, TEST_STAKER_1, 10000u128);

    let res = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_TOKEN, &[]),
        TEST_STAKER_1,
        TEST_STAKER_2,
        5000u128,
    )
    .unwrap();
    assert_eq!(res.attributes, vec![attr("action", "transfer_internal")]);

    assert_eq!(
        User::load(
            deps.as_ref().storage,
            &deps.api.addr_canonicalize(TEST_STAKER_1).unwrap()
        ),
        User {
            amount: Uint128::from(5000u128),
            reward: Default::default(),
            reward_per_token_paid: Default::default()
        }
    );

    assert_eq!(
        User::load(
            deps.as_ref().storage,
            &deps.api.addr_canonicalize(TEST_STAKER_2).unwrap()
        ),
        User {
            amount: Uint128::from(5000u128),
            reward: Default::default(),
            reward_per_token_paid: Default::default()
        }
    );
}

#[test]
fn fail_transfer_amount_exceeded() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    reply::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_TOKEN, &[]),
        TEST_STAKER_1,
        TEST_STAKER_2,
        100u128,
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::TransferAmountExceeded { .. }) => {}
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
