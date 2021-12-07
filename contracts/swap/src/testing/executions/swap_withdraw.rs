use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, coins, Api, BankMsg, CosmosMsg, Env, Fraction, MessageInfo, SubMsg, Timestamp, Uint128,
};
use pylon_gateway::swap_msg::ExecuteMsg;

use crate::entrypoints::execute;
use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::states::user::User;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_BENEFICIARY, TEST_USER_1};

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo, amount: Uint128) -> ExecuteResult {
    execute(deps.as_mut(), env, info, ExecuteMsg::Withdraw { amount })
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    const SWAP_IN_AMOUNT: u128 = 1000;
    let swap_out_amount =
        SWAP_IN_AMOUNT * default_msg.price.denominator() / default_msg.price.numerator();
    let withdraw_amount = default_msg.x_liquidity
        - ((default_msg.x_liquidity * default_msg.y_liquidity)
            / (default_msg.y_liquidity + Uint128::from(swap_out_amount)));
    let penalty = (Uint128::from(swap_out_amount) * default_msg.price) - withdraw_amount;

    super::swap_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(
            TEST_USER_1,
            &coins(SWAP_IN_AMOUNT, default_msg.input_token.to_string()),
        ),
    )
    .unwrap();

    let resp = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &[]),
        Uint128::from(swap_out_amount),
    )
    .unwrap();
    assert_eq!(
        resp.messages,
        vec![
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_USER_1.to_string(),
                amount: coins(withdraw_amount.u128(), default_msg.input_token.to_string()),
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_BENEFICIARY.to_string(),
                amount: coins(penalty.u128(), default_msg.input_token.to_string()),
            }))
        ]
    );
    assert_eq!(
        resp.attributes,
        vec![
            attr("action", "withdraw"),
            attr("sender", TEST_USER_1.to_string()),
            attr("amount", withdraw_amount.to_string()),
            attr(
                "penalty",
                ((Uint128::from(swap_out_amount) * default_msg.price) - withdraw_amount)
                    .to_string()
            ),
        ]
    );
}

#[test]
fn fail_not_allow_withdraw_after_release() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    const SWAP_IN_AMOUNT: u128 = 1000;

    super::swap_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(SWAP_IN_AMOUNT, default_msg.input_token)),
    )
    .unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(default_msg.start + default_msg.period + 1);
    match exec(
        &mut deps,
        env,
        mock_info(TEST_USER_1, &[]),
        Uint128::from(SWAP_IN_AMOUNT),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NotAllowWithdrawAfterRelease {}) => (),
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_not_allow_withdraw_after_claim() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let default_msg = instantiate::default_msg();

    const SWAP_IN_AMOUNT: u128 = 1000;

    super::swap_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &coins(SWAP_IN_AMOUNT, default_msg.input_token)),
    )
    .unwrap();

    let user_addr = deps.api.addr_canonicalize(TEST_USER_1).unwrap();
    let mut user = User::load(deps.as_mut().storage, &user_addr);
    user.swapped_out_claimed = Uint128::from(1u128);
    User::save(deps.as_mut().storage, &user_addr, &user).unwrap();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &[]),
        Uint128::from(SWAP_IN_AMOUNT),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NotAllowWithdrawAfterClaim {}) => (),
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}

#[test]
fn fail_withdraw_amount_exceeded() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &[]),
        Uint128::from(1u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::WithdrawAmountExceeded { available }) => {
            assert_eq!(available, Uint128::zero())
        }
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}
