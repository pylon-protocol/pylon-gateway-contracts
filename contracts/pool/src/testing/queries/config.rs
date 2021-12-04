use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, Decimal};
use pylon_gateway::pool_resp_v2::ConfigResponse;
use pylon_gateway::time_range::TimeRange;

use crate::queries::config::query_config_v2;
use crate::testing::{
    instantiate, mock_deps, reply, TEST_OWNER, TEST_REWARD_TOKEN, TEST_SHARE_TOKEN, TEST_TOKEN,
};

#[test]
fn success() {
    let mut deps = mock_deps();
    let (env, _, _) = instantiate::default(&mut deps);
    reply::default(&mut deps);

    let response = query_config_v2(deps.as_ref(), mock_env()).unwrap();
    let response = from_binary::<ConfigResponse>(&response).unwrap();
    assert_eq!(
        response,
        ConfigResponse {
            owner: TEST_OWNER.to_string(),
            token: TEST_TOKEN.to_string(),
            share_token: TEST_SHARE_TOKEN.to_string(),
            deposit_time: vec![TimeRange::from((
                env.block.time.seconds(),
                env.block.time.seconds() + 100,
                false
            ))],
            withdraw_time: vec![TimeRange::from((
                env.block.time.seconds(),
                env.block.time.seconds() + 100,
                true
            ))],
            deposit_cap_strategy: None,
            reward_token: TEST_REWARD_TOKEN.to_string(),
            reward_rate: Decimal::from_ratio(1000u128, 100u128),
            reward_claim_time: vec![TimeRange::from((
                env.block.time.seconds(),
                env.block.time.seconds() + 75,
                true
            ))],
            reward_distribution_time: TimeRange::from((
                env.block.time.seconds(),
                env.block.time.seconds() + 100,
                false
            ))
        }
    )
}
