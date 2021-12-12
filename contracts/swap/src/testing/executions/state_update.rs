use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Env, MessageInfo, Uint128};
use pylon_gateway::swap_msg::{ConfigureMsg, ExecuteMsg};

use crate::entrypoints::execute;
use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::states::state::State;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_USER_1};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    liq_x: Option<Uint128>,
    liq_y: Option<Uint128>,
) -> ExecuteResult {
    execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::Configure(Box::new(ConfigureMsg::State {
            x_liquidity: liq_x,
            y_liquidity: liq_y,
        })),
    )
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    let resp = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        Some(Uint128::from(1234u128)),
        Some(Uint128::from(4321u128)),
    )
    .unwrap();
    assert_eq!(resp.attributes, vec![attr("action", "update_state")]);

    assert_eq!(
        State::load(deps.as_ref().storage).unwrap(),
        State {
            total_swapped: Default::default(),
            total_claimed: Default::default(),
            x_liquidity: Uint128::from(1234u128),
            y_liquidity: Uint128::from(4321u128)
        }
    );
}

#[test]
fn test_unauthorized() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &[]),
        None,
        None,
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {
            action,
            expected,
            actual,
        }) => {
            assert_eq!(
                (action, expected, actual),
                (
                    "update_state".to_string(),
                    TEST_OWNER.to_string(),
                    TEST_USER_1.to_string()
                )
            );
        }
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}
