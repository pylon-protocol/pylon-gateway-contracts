use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Api, Env, MessageInfo, Response, Uint128};

use crate::error::ContractError;
use crate::executions::staking::deposit;
use crate::executions::ExecuteResult;
use crate::states::reward::Reward;
use crate::states::user::User;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_STAKER_1};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    sender: &str,
    amount: u128,
) -> ExecuteResult {
    deposit(
        deps.as_mut(),
        env,
        info,
        sender.to_string(),
        Uint128::from(amount),
    )
}

pub fn default(deps: &mut MockDeps, sender: &str, amount: u128) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = exec(deps, env.clone(), info.clone(), sender, amount).unwrap();

    (env, info, res)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    const DEPOSIT_AMOUNT: u128 = 1000;

    let mut reward = Reward::load(deps.as_ref().storage).unwrap();
    reward.total_deposit = Uint128::from(DEPOSIT_AMOUNT);
    Reward::save(deps.as_mut().storage, &reward).unwrap();

    let user_addr = deps.api.addr_canonicalize(TEST_STAKER_1).unwrap();
    let mut user = User::load(deps.as_ref().storage, &user_addr);
    user.amount = Uint128::from(DEPOSIT_AMOUNT);
    User::save(deps.as_mut().storage, &user_addr, &user).unwrap();

    let (_, _, res) = default(&mut deps, TEST_STAKER_1, DEPOSIT_AMOUNT);
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "deposit"),
            attr("sender", TEST_STAKER_1),
            attr("deposit_amount", DEPOSIT_AMOUNT.to_string())
        ]
    );

    assert_eq!(
        Reward::load(deps.as_ref().storage).unwrap().total_deposit,
        reward.total_deposit + Uint128::from(DEPOSIT_AMOUNT)
    );

    assert_eq!(
        User::load(deps.as_ref().storage, &user_addr).amount,
        user.amount + Uint128::from(DEPOSIT_AMOUNT)
    )
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
        1000,
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized { .. }) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
