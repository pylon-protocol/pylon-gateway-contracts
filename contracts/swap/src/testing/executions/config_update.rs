use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Api, Env, MessageInfo};
use cw20::Denom;
use pylon_gateway::swap_msg::{ConfigureMsg, ExecuteMsg};
use pylon_gateway::swap_types;

use crate::entrypoints::execute;
use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::states::config::Config;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_OWNER, TEST_USER_1, TEST_USER_2};
use crate::types::cap_strategy::CapStrategy;

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    beneficiary: Option<String>,
    input_token: Option<String>,
    output_token: Option<String>,
    deposit_cap_strategy: Option<swap_types::CapStrategy>,
    distribution_strategies: Option<Vec<swap_types::DistributionStrategy>>,
    whitelist_enabled: Option<bool>,
) -> ExecuteResult {
    execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::Configure(Box::new(ConfigureMsg::Config {
            owner,
            beneficiary,
            input_token,
            output_token,
            deposit_cap_strategy,
            distribution_strategies,
            whitelist_enabled,
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
        Some(TEST_USER_1.to_string()),
        Some(TEST_USER_2.to_string()),
        Some("ukrw".to_string()),
        Some(TEST_OWNER.to_string()),
        Some(swap_types::CapStrategy::Fixed {
            min_user_cap: None,
            max_user_cap: None,
        }),
        Some(vec![]),
        Some(true),
    )
    .unwrap();
    assert_eq!(resp.attributes, vec![attr("action", "update_config")]);

    let api = deps.api;
    let default_msg = instantiate::default_msg();
    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap(),
        Config {
            owner: api.addr_validate(TEST_USER_1).unwrap(),
            beneficiary: api.addr_validate(TEST_USER_2).unwrap(),
            start: default_msg.start,
            finish: default_msg.start + default_msg.period,
            price: default_msg.price,
            amount: default_msg.amount,
            input_token: Denom::Native("ukrw".to_string()),
            output_token: Denom::Cw20(api.addr_validate(TEST_OWNER).unwrap()),
            deposit_cap_strategy: Some(CapStrategy::Fixed {
                min_user_cap: None,
                max_user_cap: None
            }),
            distribution_strategies: vec![],
            whitelist_enabled: true
        }
    );
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_USER_1, &[]),
        None,
        None,
        None,
        None,
        None,
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
                    "update_config".to_string(),
                    TEST_OWNER.to_string(),
                    TEST_USER_1.to_string()
                )
            );
        }
        Err(e) => panic!("Unexpected error {:?}", e),
    }
}
