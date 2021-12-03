use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Api, Env, MessageInfo, Response, Uint128};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use cw20_base::state::{TokenInfo, TOKEN_INFO};
use pylon_gateway::pool_msg::{QueryMsg as PoolQueryMsg, QueryMsg};
use pylon_gateway::pool_resp_v2::ConfigResponse;
use pylon_gateway::pool_token_msg::InstantiateMsg;
use pylon_gateway::time_range::TimeRange;

use crate::entrypoints::instantiate;
use crate::executions::ExecuteResult;
use crate::states::Config;
use crate::testing::{mock_deps, MockDeps, TEST_OWNER, TEST_POOL, TEST_REWARD_TOKEN};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut MockDeps) -> (Env, MessageInfo, Response) {
    deps.querier.register_wasm_smart_query_handler(
        TEST_POOL.to_string(),
        Box::new(|x| match from_binary::<PoolQueryMsg>(x).unwrap() {
            QueryMsg::ConfigV2 {} => to_binary(&ConfigResponse {
                owner: "".to_string(),
                token: "".to_string(),
                share_token: "".to_string(),
                deposit_time: vec![],
                withdraw_time: vec![],
                deposit_cap_strategy: None,
                reward_token: TEST_REWARD_TOKEN.to_string(),
                reward_rate: Default::default(),
                reward_claim_time: vec![],
                reward_distribution_time: TimeRange::from((30 * 86400, 180 * 86400, false)),
            }),
            _ => panic!("Unsupported query"),
        }),
    );

    deps.querier.register_wasm_smart_query_handler(
        TEST_REWARD_TOKEN.to_string(),
        Box::new(|x| match from_binary::<Cw20QueryMsg>(x).unwrap() {
            Cw20QueryMsg::TokenInfo {} => to_binary(&TokenInfoResponse {
                name: "".to_string(),
                symbol: "TRT".to_string(),
                decimals: 0,
                total_supply: Default::default(),
            }),
            _ => panic!("Unsupported query"),
        }),
    );

    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);
    let res = exec(deps, env.clone(), info.clone(), default_msg()).unwrap();

    (env, info, res)
}

pub fn default_msg() -> InstantiateMsg {
    InstantiateMsg {
        pool: TEST_POOL.to_string(),
    }
}

#[test]
fn success() {
    let mut deps = mock_deps();
    default(&mut deps);

    let api = deps.api;

    assert_eq!(
        TOKEN_INFO.load(deps.as_ref().storage).unwrap(),
        TokenInfo {
            name: "Pylon bDP Token for Gateway TRT 5m Pool".to_string(),
            symbol: "bTRTDP-5m".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
            mint: None
        }
    );

    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap(),
        Config {
            pool: api.addr_validate(TEST_POOL).unwrap()
        }
    );
}
