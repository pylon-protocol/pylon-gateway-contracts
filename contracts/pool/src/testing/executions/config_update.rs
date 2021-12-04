use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Addr, Api, Decimal, Env, MessageInfo};
use pylon_gateway::time_range::TimeRange;

use crate::executions::config::update;
use crate::executions::ExecuteResult;
use crate::states::config::Config;
use crate::testing::{
    instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_REWARD_TOKEN, TEST_SHARE_TOKEN,
    TEST_STAKER_1, TEST_STAKER_2,
};

#[allow(clippy::too_many_arguments)]
pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    owner: Option<&str>,
    share_token: Option<&str>,
    reward_token: Option<&str>,
    claim_time: Option<Vec<TimeRange>>,
    deposit_time: Option<Vec<TimeRange>>,
    withdraw_time: Option<Vec<TimeRange>>,
    deposit_cap_strategy: Option<&str>,
) -> ExecuteResult {
    update(
        deps.as_mut(),
        env,
        info,
        owner.map(|x| x.to_string()),
        share_token.map(|x| x.to_string()),
        reward_token.map(|x| x.to_string()),
        claim_time,
        deposit_time,
        withdraw_time,
        deposit_cap_strategy.map(|x| x.to_string()),
    )
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    let res = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        Some(TEST_STAKER_1),
        Some(TEST_REWARD_TOKEN),
        Some(TEST_SHARE_TOKEN),
        Some(vec![TimeRange::from((1, 2, false))]),
        Some(vec![TimeRange::from((3, 4, false))]),
        Some(vec![TimeRange::from((5, 6, false))]),
        Some(TEST_STAKER_2),
    )
    .unwrap();
    assert_eq!(res.attributes, vec![attr("action", "update_config")]);

    let default_msg = instantiate::default_msg();
    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap(),
        Config {
            owner: deps.api.addr_validate(TEST_STAKER_1).unwrap(),
            token: Addr::unchecked("".to_string()),
            share_token: deps.api.addr_validate(TEST_REWARD_TOKEN).unwrap(),
            deposit_time: vec![TimeRange::from((3, 4, false))],
            withdraw_time: vec![TimeRange::from((5, 6, false))],
            deposit_cap_strategy: Some(deps.api.addr_validate(TEST_STAKER_2).unwrap()),
            reward_token: deps.api.addr_validate(TEST_SHARE_TOKEN).unwrap(),
            reward_rate: Decimal::from_ratio(10u128, 1u128),
            reward_claim_time: vec![TimeRange::from((1, 2, false))],
            reward_distribution_time: default_msg.reward_distribution_time,
        }
    );
}
