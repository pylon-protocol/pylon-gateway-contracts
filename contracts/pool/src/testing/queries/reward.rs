use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, Uint128};
use pylon_gateway::pool_resp::RewardResponse;

use crate::queries::reward::query_reward;
use crate::states::reward::Reward;
use crate::testing::{instantiate, mock_deps};

#[test]
fn success() {
    let mut deps = mock_deps();
    let (env, _, _) = instantiate::default(&mut deps);

    let mut reward = Reward::load(deps.as_ref().storage).unwrap();
    reward.total_deposit = Uint128::from(1000u128);
    Reward::save(deps.as_mut().storage, &reward).unwrap();

    let response = query_reward(deps.as_ref(), mock_env()).unwrap();
    let response = from_binary::<RewardResponse>(&response).unwrap();
    assert_eq!(
        response,
        RewardResponse {
            total_deposit: reward.total_deposit,
            last_update_time: env.block.time.seconds()
        }
    )
}
