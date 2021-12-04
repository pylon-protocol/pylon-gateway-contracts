use cosmwasm_std::testing::{mock_env, mock_info};

use pylon_gateway::pool_msg::{ConfigureMsg, ExecuteMsg};

use crate::entrypoints::execute;
use crate::error::ContractError;
use crate::testing::{instantiate, mock_deps, TEST_STAKER_1};

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    match execute(
        deps.as_mut(),
        mock_env(),
        mock_info(TEST_STAKER_1, &[]),
        ExecuteMsg::Configure(ConfigureMsg::Config {
            owner: None,
            share_token: None,
            reward_token: None,
            claim_time: None,
            deposit_time: None,
            withdraw_time: None,
            deposit_cap_strategy: None,
        }),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized { .. }) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }

    match execute(
        deps.as_mut(),
        mock_env(),
        mock_info(TEST_STAKER_1, &[]),
        ExecuteMsg::Configure(ConfigureMsg::AddReward {
            amount: Default::default(),
        }),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized { .. }) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }

    match execute(
        deps.as_mut(),
        mock_env(),
        mock_info(TEST_STAKER_1, &[]),
        ExecuteMsg::Configure(ConfigureMsg::SubReward {
            amount: Default::default(),
        }),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized { .. }) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
