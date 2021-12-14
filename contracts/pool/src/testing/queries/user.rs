use crate::queries::user::{query_staker, query_stakers};
use crate::states::reward::Reward;
use crate::states::user::User;
use crate::testing::{instantiate, mock_deps, TEST_STAKER_1, TEST_STAKER_2};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, Api, Decimal, Uint128};
use pylon_gateway::pool_resp::{StakerResponse, StakersResponse};

#[test]
fn success() {
    let mut deps = mock_deps();
    let (env, _, _) = instantiate::default(&mut deps);

    let staker1_addr = deps.api.addr_canonicalize(TEST_STAKER_1).unwrap();
    let mut staker1 = User::load(deps.as_ref().storage, &staker1_addr);
    staker1.amount = Uint128::from(1000u128);
    staker1.reward = Uint128::from(1234u128);
    staker1.reward_per_token_paid = Decimal::one();
    User::save(deps.as_mut().storage, &staker1_addr, &staker1).unwrap();

    let staker2_addr = deps.api.addr_canonicalize(TEST_STAKER_2).unwrap();
    let mut staker2 = User::load(deps.as_ref().storage, &staker2_addr);
    staker2.amount = Uint128::from(2000u128);
    staker2.reward = Uint128::from(4321u128);
    staker2.reward_per_token_paid = Decimal::zero();
    User::save(deps.as_mut().storage, &staker2_addr, &staker2).unwrap();

    Reward::save(
        deps.as_mut().storage,
        &Reward {
            total_deposit: Uint128::from(3000u128),
            last_update_time: env.block.time.seconds(),
            reward_per_token_stored: Decimal::from_ratio(2u128, 1u128),
        },
    )
    .unwrap();

    let response = query_staker(deps.as_ref(), mock_env(), TEST_STAKER_1.to_string()).unwrap();
    let response = from_binary::<StakerResponse>(&response).unwrap();
    assert_eq!(
        response,
        StakerResponse {
            address: TEST_STAKER_1.to_string(),
            staked: staker1.amount,
            reward: staker1.reward + staker1.amount,
            available_cap: None
        }
    );

    let response = query_stakers(
        deps.as_ref(),
        mock_env(),
        Some(TEST_STAKER_1.to_string()),
        None,
        None,
    )
    .unwrap();
    let response = from_binary::<StakersResponse>(&response).unwrap();
    assert_eq!(
        response,
        StakersResponse {
            stakers: vec![StakerResponse {
                address: TEST_STAKER_2.to_string(),
                staked: staker2.amount,
                reward: staker2.reward + staker2.amount * Uint128::from(2u128),
                available_cap: None
            }]
        }
    )
}
