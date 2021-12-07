use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Api, Decimal, Env, Response, Uint128};
use cw20::Denom;
use pylon_gateway::swap_msg::MigrateMsg;
use pylon_gateway::swap_types;

use crate::entrypoints::migrate;
use crate::migrations::nexus::{LegacyConfig, LegacyState, LegacyUser};
use crate::migrations::MigrateResult;
use crate::states::config::Config;
use crate::states::state::State;
use crate::states::user::User;
use crate::testing::{
    instantiate, mock_deps, MockDeps, TEST_BENEFICIARY, TEST_OWNER, TEST_USER_1, TEST_USER_2,
};
use crate::types::cap_strategy::CapStrategy;
use crate::types::distribution_strategy::DistributionStrategy;

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    deposit_cap_strategy: Option<swap_types::CapStrategy>,
) -> MigrateResult {
    migrate(
        deps.as_mut(),
        env,
        MigrateMsg::Nexus {
            deposit_cap_strategy,
        },
    )
}

pub fn setup_legacy_states(deps: &mut MockDeps, _env: Env) {
    let api = deps.api;
    let default_msg = instantiate::default_msg();

    LegacyConfig::save(
        deps.as_mut().storage,
        &LegacyConfig {
            // accounts
            owner: TEST_OWNER.to_string(),
            beneficiary: TEST_BENEFICIARY.to_string(),
            // details
            start: default_msg.start,
            finish: default_msg.start + default_msg.period,
            price: default_msg.price,
            swap_pool_size: default_msg.amount,
            // strategies
            cap_strategy: None,
            distribution_strategy: vec![swap_types::DistributionStrategy::Lockup {
                release_time: default_msg.start + default_msg.period,
                release_amount: Decimal::one(),
            }],
            whitelist_enabled: false,
        },
    )
    .unwrap();

    LegacyState::save(
        deps.as_mut().storage,
        &LegacyState {
            total_swapped: Uint128::from(20000u128),
            total_claimed: Uint128::from(10000u128),
            x_denom: default_msg.input_token,
            y_addr: default_msg.output_token,
            liq_x: default_msg.x_liquidity,
            liq_y: default_msg.y_liquidity,
        },
    )
    .unwrap();

    LegacyUser::save(
        deps.as_mut().storage,
        &api.addr_canonicalize(TEST_USER_1).unwrap(),
        &LegacyUser {
            whitelisted: false,
            swapped_in: Uint128::from(1500u128),
            swapped_out: Uint128::from(15000u128),
            swapped_out_claimed: Uint128::from(10000u128),
        },
    )
    .unwrap();

    LegacyUser::save(
        deps.as_mut().storage,
        &api.addr_canonicalize(TEST_USER_2).unwrap(),
        &LegacyUser {
            whitelisted: false,
            swapped_in: Uint128::from(500u128),
            swapped_out: Uint128::from(5000u128),
            swapped_out_claimed: Default::default(),
        },
    )
    .unwrap();
}

#[test]
fn success() {
    let mut deps = mock_deps();
    let default_msg = instantiate::default_msg();
    setup_legacy_states(&mut deps, mock_env());

    let resp = exec(
        &mut deps,
        mock_env(),
        Some(swap_types::CapStrategy::Fixed {
            min_user_cap: None,
            max_user_cap: None,
        }),
    )
    .unwrap();
    assert_eq!(resp, Response::default());

    let api = deps.api;
    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap(),
        Config {
            owner: api.addr_validate(TEST_OWNER).unwrap(),
            beneficiary: api.addr_validate(TEST_BENEFICIARY).unwrap(),
            start: default_msg.start,
            finish: default_msg.start + default_msg.period,
            price: default_msg.price,
            amount: default_msg.amount,
            input_token: Denom::Native(default_msg.input_token),
            output_token: Denom::Cw20(
                api.addr_validate(default_msg.output_token.as_str())
                    .unwrap()
            ),
            deposit_cap_strategy: Some(CapStrategy::Fixed {
                min_user_cap: None,
                max_user_cap: None
            }),
            distribution_strategies: default_msg
                .distribution_strategies
                .iter()
                .map(|x| DistributionStrategy::from(x.clone()))
                .collect(),
            whitelist_enabled: default_msg.whitelist_enabled
        }
    );

    assert_eq!(
        State::load(deps.as_ref().storage).unwrap(),
        State {
            total_swapped: Uint128::from(20000u128),
            total_claimed: Uint128::from(10000u128),
            x_liquidity: default_msg.x_liquidity,
            y_liquidity: default_msg.y_liquidity
        }
    );

    assert_eq!(
        User::load_range(deps.as_ref().storage, None, None, None),
        vec![
            (
                api.addr_canonicalize(TEST_USER_1).unwrap(),
                User {
                    swapped_in: Uint128::from(1500u128),
                    swapped_out: Uint128::from(15000u128),
                    swapped_out_claimed: Uint128::from(10000u128),
                }
            ),
            (
                api.addr_canonicalize(TEST_USER_2).unwrap(),
                User {
                    swapped_in: Uint128::from(500u128),
                    swapped_out: Uint128::from(5000u128),
                    swapped_out_claimed: Default::default(),
                }
            )
        ]
    );
}
