use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    to_binary, Addr, Api, CosmosMsg, Decimal, Env, MessageInfo, ReplyOn, Response, SubMsg, Uint128,
    WasmMsg,
};
use cw2::{get_contract_version, ContractVersion};
use cw20::Denom;
use pylon_gateway::swap_msg::InstantiateMsg;
use pylon_gateway::swap_types::DistributionStrategy as SwapDistributionStrategy;

use crate::entrypoints::instantiate;
use crate::executions::ExecuteResult;
use crate::states::config::Config;
use crate::states::state::State;
use crate::testing::{
    mock_deps, MockDeps, TEST_BENEFICIARY, TEST_INPUT_TOKEN, TEST_OUTPUT_TOKEN, TEST_OWNER,
};
use crate::types::distribution_strategy::DistributionStrategy;

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut MockDeps) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);
    let res = exec(deps, env.clone(), info.clone(), default_msg()).unwrap();

    (env, info, res)
}

pub fn default_msg() -> InstantiateMsg {
    let default_blocktime = mock_env().block.time.seconds();

    InstantiateMsg {
        beneficiary: TEST_BENEFICIARY.to_string(),
        start: default_blocktime,
        period: default_blocktime + 100,
        price: Decimal::from_ratio(1u128, 10u128),
        amount: Uint128::from(10000u128),
        input_token: TEST_INPUT_TOKEN.to_string(),
        output_token: TEST_OUTPUT_TOKEN.to_string(),
        x_liquidity: Uint128::from(10000u128),
        y_liquidity: Uint128::from(100000u128),
        deposit_cap_strategy: None,
        distribution_strategies: vec![SwapDistributionStrategy::Lockup {
            release_time: default_blocktime + 100,
            release_amount: Decimal::one(),
        }],
        whitelist_enabled: false,
    }
}

#[test]
fn success() {
    let mut deps = mock_deps();

    let (env, info, resp) = default(&mut deps);
    assert_eq!(resp, Response::default());

    let api = deps.api;
    // check contract version
    assert_eq!(
        get_contract_version(deps.as_ref().storage).unwrap(),
        ContractVersion {
            contract: CONTRACT_NAME.to_string(),
            version: CONTRACT_VERSION.to_string()
        }
    );

    // check config
    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap(),
        Config {
            owner: api.addr_validate(TEST_OWNER).unwrap(),
            beneficiary: api.addr_validate(TEST_BENEFICIARY).unwrap(),
            start: env.block.time.seconds(),
            finish: env.block.time.seconds() + 100,
            price: Decimal::from_ratio(1u128, 10u128),
            amount: Uint128::from(10000u128),
            input_token: Denom::Native(TEST_INPUT_TOKEN.to_string()),
            output_token: Denom::Cw20(api.addr_validate(TEST_OUTPUT_TOKEN).unwrap()),
            deposit_cap_strategy: None,
            distribution_strategies: vec![DistributionStrategy::Lockup {
                release_time: env.block.time.seconds() + 100,
                release_amount: Decimal::one()
            }],
            whitelist_enabled: false
        }
    );

    // check state
    assert_eq!(
        State::load(deps.as_ref().storage).unwrap(),
        State {
            total_swapped: Uint128::zero(),
            total_claimed: Uint128::zero(),
            x_liquidity: Uint128::from(10000u128),
            y_liquidity: Uint128::from(100000u128)
        }
    );
}
