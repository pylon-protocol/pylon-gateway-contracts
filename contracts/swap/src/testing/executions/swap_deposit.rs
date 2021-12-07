use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, coin, coins, Api, Env, Fraction, MessageInfo, Timestamp, Uint128};
use pylon_gateway::swap_msg::ExecuteMsg;
use pylon_gateway::swap_types;

use crate::entrypoints::execute;
use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::states::state::State;
use crate::states::user::User;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_USER_1};

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo) -> ExecuteResult {
    execute(deps.as_mut(), env, info, ExecuteMsg::Deposit {})
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    const SWAP_IN_AMOUNT: u128 = 100;
    let swap_out_amount: u128 =
        SWAP_IN_AMOUNT * default_msg.price.denominator() / default_msg.price.numerator();

    let resp = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(SWAP_IN_AMOUNT, default_msg.input_token)),
    )
    .unwrap();
    assert_eq!(
        resp.attributes,
        vec![
            attr("action", "deposit"),
            attr("sender", TEST_USER_1.to_string()),
            attr("swapped_in", SWAP_IN_AMOUNT.to_string()),
            attr("swapped_out", (swap_out_amount).to_string())
        ]
    );

    let api = deps.api;

    assert_eq!(
        User::load(
            deps.as_ref().storage,
            &api.addr_canonicalize(TEST_USER_1).unwrap()
        ),
        User {
            swapped_in: Uint128::from(SWAP_IN_AMOUNT),
            swapped_out: Uint128::from(swap_out_amount),
            swapped_out_claimed: Uint128::zero(),
        }
    );
    assert_eq!(
        State::load(deps.as_ref().storage).unwrap(),
        State {
            total_swapped: Uint128::from(swap_out_amount),
            total_claimed: Uint128::zero(),
            x_liquidity: default_msg.x_liquidity,
            y_liquidity: default_msg.y_liquidity
        }
    );
}

#[test]
fn fail_swap_not_started() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start - 1);
    match exec(
        &mut deps,
        env,
        mock_info(TEST_USER_1, &coins(0, default_msg.input_token)),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::SwapNotStarted { start }) => assert_eq!(start, default_msg.start),
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_swap_finished() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + default_msg.period + 1);
    match exec(
        &mut deps,
        env,
        mock_info(TEST_USER_1, &coins(0, default_msg.input_token)),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::SwapFinished { finish }) => {
            assert_eq!(finish, default_msg.start + default_msg.period)
        }
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_not_allow_zero_amount() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(0, default_msg.input_token)),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NotAllowZeroAmount {}) => (),
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_not_allow_other_denoms() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(
            TEST_USER_1,
            &[
                coin(100u128, default_msg.input_token.clone()),
                coin(100u128, "ukrw"),
            ],
        ),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NotAllowOtherDenoms { denom }) => {
            assert_eq!(denom, default_msg.input_token)
        }
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_not_allow_non_whitelisted() {
    let mut deps = mock_deps();
    let mut default_msg = instantiate::default_msg();
    default_msg.whitelist_enabled = true;
    instantiate::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        default_msg.clone(),
    )
    .unwrap();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(100u128, default_msg.input_token)),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NotAllowNonWhitelisted { address }) => {
            assert_eq!(address, TEST_USER_1.to_string())
        }
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_available_cap_exceeded() {
    let mut deps = mock_deps();
    let mut default_msg = instantiate::default_msg();
    default_msg.deposit_cap_strategy = Some(swap_types::CapStrategy::Fixed {
        min_user_cap: None,
        max_user_cap: Some(Uint128::from(1u128)),
    });
    instantiate::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        default_msg.clone(),
    )
    .unwrap();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(100u128, default_msg.input_token)),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::AvailableCapExceeded { available }) => {
            assert_eq!(available, Uint128::from(1u128))
        }
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}
