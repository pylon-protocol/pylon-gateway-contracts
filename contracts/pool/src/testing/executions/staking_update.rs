use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Api, Decimal, Env, MessageInfo, Response, Timestamp, Uint128};

use crate::executions::staking::update;
use crate::executions::ExecuteResult;
use crate::states::reward::Reward;
use crate::states::user::User;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_STAKER_1};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    target: Option<String>,
) -> ExecuteResult {
    update(deps.as_mut(), env, info, target)
}

pub fn default(
    deps: &mut MockDeps,
    after: u64,
    target: Option<&str>,
) -> (Env, MessageInfo, Response) {
    let default_msg = instantiate::default_msg();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.reward_distribution_time.0 + after);
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = exec(
        deps,
        env.clone(),
        info.clone(),
        target.map(|x| x.to_string()),
    )
    .unwrap();

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

    let (_, info, res) = default(&mut deps, 10, None);
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "update"),
            attr("sender", info.sender.to_string()),
            attr(
                "stored_rpt",
                Decimal::from_ratio(100u128, DEPOSIT_AMOUNT).to_string()
            )
        ]
    );

    let (_, info, res) = default(&mut deps, 10, Some(TEST_STAKER_1));
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "update"),
            attr("sender", info.sender.to_string()),
            attr(
                "stored_rpt",
                Decimal::from_ratio(100u128, DEPOSIT_AMOUNT).to_string()
            ),
            attr("target", TEST_STAKER_1.to_string()),
            attr("reward", 100u128.to_string())
        ]
    );
}
