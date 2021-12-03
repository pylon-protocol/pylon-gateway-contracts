use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Decimal, Env, MessageInfo, Response, Timestamp, Uint128};

use crate::executions::config::adjust_reward;
use crate::executions::ExecuteResult;
use crate::testing::{instantiate, mock_deps, MockDeps};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    _info: MessageInfo,
    amount: u128,
    remove: bool,
) -> ExecuteResult {
    adjust_reward(deps.as_mut(), env, Uint128::from(amount), remove)
}

pub fn default(
    deps: &mut MockDeps,
    after: u64,
    amount: u128,
    remove: bool,
) -> (Env, MessageInfo, Response) {
    let default_msg = instantiate::default_msg();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.reward_distribution_time.start + after);
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = exec(deps, env.clone(), info.clone(), amount, remove).unwrap();

    (env, info, res)
}

#[test]
fn success_add() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    let (_, _, res) = default(&mut deps, 50, 500, false);
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "add_reward"),
            attr(
                "reward_rate_before",
                Decimal::from_ratio(10u128, 1u128).to_string()
            ),
            attr(
                "reward_rate_after",
                Decimal::from_ratio(20u128, 1u128).to_string()
            )
        ]
    )
}

#[test]
fn success_sub() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    let (_, _, res) = default(&mut deps, 50, 250, true);
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "sub_reward"),
            attr(
                "reward_rate_before",
                Decimal::from_ratio(10u128, 1u128).to_string()
            ),
            attr(
                "reward_rate_after",
                Decimal::from_ratio(5u128, 1u128).to_string()
            )
        ]
    )
}
