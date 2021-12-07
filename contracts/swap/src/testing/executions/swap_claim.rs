use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, coins, to_binary, Api, CosmosMsg, Decimal, Env, Fraction, MessageInfo, Response, Storage,
    SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use pylon_gateway::swap_msg::ExecuteMsg;
use pylon_gateway::swap_types;

use crate::entrypoints::execute;
use crate::executions::ExecuteResult;
use crate::states::user::User;
use crate::testing::{
    instantiate, mock_deps, MockDeps, TEST_OUTPUT_TOKEN, TEST_OWNER, TEST_USER_1,
};

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo) -> ExecuteResult {
    execute(deps.as_mut(), env, info, ExecuteMsg::Claim {})
}

pub fn assert_claim_response(
    deps: &MockDeps,
    resp: Response,
    user: &str,
    amount: u128,
    accumulated: u128,
) {
    assert_eq!(
        resp.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_OUTPUT_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: user.to_string(),
                amount: Uint128::from(amount)
            })
            .unwrap(),
            funds: vec![]
        }))]
    );
    assert_eq!(
        resp.attributes,
        vec![
            attr("action", "claim"),
            attr("sender", user.to_string()),
            attr("amount", amount.to_string()) // 100%
        ]
    );
    assert_eq!(
        User::load(
            deps.as_ref().storage,
            &deps.api.addr_canonicalize(TEST_USER_1).unwrap()
        )
        .swapped_out_claimed,
        Uint128::from(accumulated),
    );
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    const SWAP_IN_AMOUNT: u128 = 1000;
    let swap_out_amount =
        SWAP_IN_AMOUNT * default_msg.price.denominator() / default_msg.price.numerator();

    super::swap_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(SWAP_IN_AMOUNT, default_msg.input_token)),
    )
    .unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + default_msg.period + 1);
    let resp = exec(&mut deps, env, mock_info(TEST_USER_1, &[])).unwrap();
    assert_claim_response(&deps, resp, TEST_USER_1, 10000u128, swap_out_amount);
}

#[test]
fn success_lockup_strategy() {
    let mut deps = mock_deps();
    let mut default_msg = instantiate::default_msg();
    default_msg.distribution_strategies = vec![
        swap_types::DistributionStrategy::Lockup {
            release_time: default_msg.start + 33,
            release_amount: Decimal::from_ratio(33333u128, 100000u128),
        },
        swap_types::DistributionStrategy::Lockup {
            release_time: default_msg.start + 66,
            release_amount: Decimal::from_ratio(33333u128, 100000u128),
        },
        swap_types::DistributionStrategy::Lockup {
            release_time: default_msg.start + 100,
            release_amount: Decimal::from_ratio(33334u128, 100000u128),
        },
    ];

    instantiate::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        default_msg.clone(),
    )
    .unwrap();

    const SWAP_IN_AMOUNT: u128 = 1000;

    super::swap_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(SWAP_IN_AMOUNT, default_msg.input_token)),
    )
    .unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + 33);
    let resp = exec(&mut deps, env, mock_info(TEST_USER_1, &[])).unwrap();
    assert_claim_response(&deps, resp, TEST_USER_1, 3333u128, 3333u128);

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + 66);
    let resp = exec(&mut deps, env, mock_info(TEST_USER_1, &[])).unwrap();
    assert_claim_response(&deps, resp, TEST_USER_1, 3333u128, 6666u128);

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + 100);
    let resp = exec(&mut deps, env, mock_info(TEST_USER_1, &[])).unwrap();
    assert_claim_response(&deps, resp, TEST_USER_1, 3334u128, 10000u128);
}

#[test]
fn success_vesting_strategy() {
    let mut deps = mock_deps();
    let mut default_msg = instantiate::default_msg();
    default_msg.distribution_strategies = vec![swap_types::DistributionStrategy::Vesting {
        release_start_time: default_msg.start,
        release_finish_time: default_msg.start + default_msg.period,
        release_amount: Decimal::one(),
    }];

    instantiate::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        default_msg.clone(),
    )
    .unwrap();

    const SWAP_IN_AMOUNT: u128 = 1000;

    super::swap_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(SWAP_IN_AMOUNT, default_msg.input_token)),
    )
    .unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + 33);
    let resp = exec(&mut deps, env, mock_info(TEST_USER_1, &[])).unwrap();
    assert_claim_response(&deps, resp, TEST_USER_1, 3300u128, 3300u128);

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + 66);
    let resp = exec(&mut deps, env, mock_info(TEST_USER_1, &[])).unwrap();
    assert_claim_response(&deps, resp, TEST_USER_1, 3300u128, 6600u128);

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + 100);
    let resp = exec(&mut deps, env, mock_info(TEST_USER_1, &[])).unwrap();
    assert_claim_response(&deps, resp, TEST_USER_1, 3400u128, 10000u128);
}
