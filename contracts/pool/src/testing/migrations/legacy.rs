use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Api, Decimal, Env, Uint128};
use cosmwasm_storage::{Bucket, Singleton};
use pylon_gateway::time_range::TimeRange;

use crate::migrations::legacy::{
    migrate, DepositConfig, DistributionConfig, LegacyConfig, LegacyReward, LegacyUser,
};
use crate::migrations::MigrateResult;
use crate::states::config::Config;
use crate::states::reward::Reward;
use crate::states::user::User;
use crate::states::{KEY_CONFIG, KEY_REWARD, PREFIX_USER};
use crate::testing::{
    mock_deps, MockDeps, TEST_CAP_STRATEGY, TEST_OWNER, TEST_REWARD_TOKEN, TEST_SHARE_TOKEN,
    TEST_STAKER_1, TEST_STAKER_2,
};

pub fn exec(deps: &mut MockDeps, env: Env) -> MigrateResult {
    migrate(deps.as_mut(), env)
}

pub fn setup_legacy_states(deps: &mut MockDeps, _env: Env) {
    let api = deps.api;

    Singleton::<LegacyConfig>::new(deps.as_mut().storage, KEY_CONFIG)
        .save(&LegacyConfig {
            // addresses
            owner: TEST_OWNER.to_string(),
            share_token: TEST_SHARE_TOKEN.to_string(),
            reward_token: TEST_REWARD_TOKEN.to_string(),
            cap_strategy: Some(TEST_CAP_STRATEGY.to_string()),
            // time
            deposit_config: DepositConfig {
                time: TimeRange::from((2, 4, false)),
                user_cap: Default::default(),
                total_cap: Default::default(),
            },
            withdraw_time: vec![TimeRange::from((2, 4, true))],
            claim_time: TimeRange::from((3, 4, false)),
            distribution_config: DistributionConfig {
                time: TimeRange::from((2, 4, false)),
                reward_rate: Decimal256::from_ratio(
                    Uint256::from(100u128),
                    Uint256::from(1234u128),
                ),
            },
        })
        .unwrap();

    Singleton::<LegacyReward>::new(deps.as_mut().storage, KEY_REWARD)
        .save(&LegacyReward {
            total_deposit: Uint256::from(30000u128),
            last_update_time: 1234567u64,
            reward_per_token_stored: Decimal256::from_ratio(
                Uint256::from(1000u128),
                Uint256::from(1234u128),
            ),
        })
        .unwrap();

    Bucket::<LegacyUser>::new(deps.as_mut().storage, PREFIX_USER)
        .save(
            api.addr_canonicalize(TEST_STAKER_1).unwrap().as_slice(),
            &LegacyUser {
                amount: Uint256::from(10000u128),
                reward: Uint256::from(12345u128),
                reward_per_token_paid: Decimal256::from_ratio(
                    Uint256::from(1234u128),
                    Uint256::from(1000u128),
                ),
            },
        )
        .unwrap();

    Bucket::<LegacyUser>::new(deps.as_mut().storage, PREFIX_USER)
        .save(
            api.addr_canonicalize(TEST_STAKER_2).unwrap().as_slice(),
            &LegacyUser {
                amount: Uint256::from(20000u128),
                reward: Uint256::from(54321u128),
                reward_per_token_paid: Decimal256::from_ratio(
                    Uint256::from(1000u128),
                    Uint256::from(1234u128),
                ),
            },
        )
        .unwrap();
}

#[test]
fn success() {
    let mut deps = mock_deps();
    setup_legacy_states(&mut deps, mock_env());

    exec(&mut deps, mock_env()).unwrap();

    let api = deps.api;
    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap(),
        Config {
            owner: api.addr_validate(TEST_OWNER).unwrap(),
            token: Addr::unchecked("".to_string()),
            share_token: api.addr_validate(TEST_SHARE_TOKEN).unwrap(),
            deposit_time: vec![TimeRange::from((2, 4, false))],
            withdraw_time: vec![TimeRange::from((2, 4, true))],
            deposit_cap_strategy: Some(api.addr_validate(TEST_CAP_STRATEGY).unwrap()),
            reward_token: api.addr_validate(TEST_REWARD_TOKEN).unwrap(),
            reward_rate: Decimal::from_ratio(100u128, 1234u128),
            reward_claim_time: vec![TimeRange::from((3, 4, false))],
            reward_distribution_time: TimeRange::from((2, 4, false))
        }
    );

    assert_eq!(
        Reward::load(deps.as_ref().storage).unwrap(),
        Reward {
            total_deposit: Uint128::from(30000u128),
            last_update_time: 1234567u64,
            reward_per_token_stored: Decimal::from_ratio(1000u128, 1234u128)
        }
    );

    assert_eq!(
        User::load(
            deps.as_ref().storage,
            &deps.api.addr_canonicalize(TEST_STAKER_1).unwrap(),
        ),
        User {
            amount: Uint128::from(10000u128),
            reward: Uint128::from(12345u128),
            reward_per_token_paid: Decimal::from_ratio(1234u128, 1000u128)
        }
    );

    assert_eq!(
        User::load(
            deps.as_ref().storage,
            &deps.api.addr_canonicalize(TEST_STAKER_2).unwrap(),
        ),
        User {
            amount: Uint128::from(20000u128),
            reward: Uint128::from(54321u128),
            reward_per_token_paid: Decimal::from_ratio(1000u128, 1234u128)
        }
    );
}
