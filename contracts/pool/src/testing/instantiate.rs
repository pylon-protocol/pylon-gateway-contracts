use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Api, CosmosMsg, Decimal, Env, MessageInfo, ReplyOn, Response,
    SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use pylon_gateway::pool_msg::InstantiateMsg;
use pylon_gateway::pool_token_msg::InstantiateMsg as PoolInitMsg;
use pylon_gateway::time_range::TimeRange;

use crate::constants::INSTANTIATE_REPLY_ID;
use crate::entrypoints::instantiate;
use crate::executions::ExecuteResult;
use crate::states::config::Config;
use crate::states::reward::Reward;
use crate::testing::{mock_deps, MockDeps, TEST_OWNER, TEST_REWARD_TOKEN, TEST_SHARE_TOKEN};

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
        TEST_REWARD_TOKEN.to_string(),
        Box::new(|x| match from_binary::<Cw20QueryMsg>(x).unwrap() {
            Cw20QueryMsg::TokenInfo {} => to_binary(&TokenInfoResponse {
                name: "".to_string(),
                symbol: "".to_string(),
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
    let default_blocktime = mock_env().block.time.seconds();

    InstantiateMsg {
        pool_token_code_id: 1234,
        share_token: TEST_SHARE_TOKEN.to_string(),
        deposit_time: vec![TimeRange::from((
            default_blocktime,
            default_blocktime + 100,
            false,
        ))],
        withdraw_time: vec![TimeRange::from((
            default_blocktime,
            default_blocktime + 100,
            true,
        ))],
        deposit_cap_strategy: None,
        reward_token: TEST_REWARD_TOKEN.to_string(),
        reward_amount: Uint128::from(1000u128),
        reward_claim_time: vec![TimeRange::from((
            default_blocktime,
            default_blocktime + 75,
            true,
        ))],
        reward_distribution_time: TimeRange::from((
            default_blocktime,
            default_blocktime + 100,
            false,
        )),
    }
}

#[test]
fn success() {
    let mut deps = mock_deps();

    let (env, info, resp) = default(&mut deps);
    assert_eq!(
        resp.messages,
        vec![SubMsg {
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(info.sender.to_string()),
                code_id: 1234,
                msg: to_binary(&PoolInitMsg {
                    pool: env.contract.address.to_string()
                })
                .unwrap(),
                funds: vec![],
                label: "".to_string()
            }),
            gas_limit: None,
            id: INSTANTIATE_REPLY_ID,
            reply_on: ReplyOn::Success
        }]
    );

    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap(),
        Config {
            owner: info.sender,
            token: Addr::unchecked("".to_string()),
            share_token: deps.api.addr_validate(TEST_SHARE_TOKEN).unwrap(),
            deposit_time: vec![TimeRange {
                start: env.block.time.seconds(),
                finish: env.block.time.seconds() + 100,
                inverse: false
            }],
            withdraw_time: vec![TimeRange {
                start: env.block.time.seconds(),
                finish: env.block.time.seconds() + 100,
                inverse: true
            }],
            deposit_cap_strategy: None,
            reward_token: deps.api.addr_validate(TEST_REWARD_TOKEN).unwrap(),
            reward_rate: Decimal::from_ratio(1000u128, 100u128),
            reward_claim_time: vec![TimeRange {
                start: env.block.time.seconds(),
                finish: env.block.time.seconds() + 75,
                inverse: true
            }],
            reward_distribution_time: TimeRange {
                start: env.block.time.seconds(),
                finish: env.block.time.seconds() + 100,
                inverse: false
            }
        }
    );

    assert_eq!(
        Reward::load(deps.as_ref().storage).unwrap(),
        Reward {
            total_deposit: Default::default(),
            last_update_time: env.block.time.seconds(),
            reward_per_token_stored: Default::default()
        }
    )
}

#[test]
fn success_other_decimals() {
    let mut deps = mock_deps();
    deps.querier.register_wasm_smart_query_handler(
        TEST_REWARD_TOKEN.to_string(),
        Box::new(|x| match from_binary::<Cw20QueryMsg>(x).unwrap() {
            Cw20QueryMsg::TokenInfo {} => to_binary(&TokenInfoResponse {
                name: "".to_string(),
                symbol: "".to_string(),
                decimals: 6,
                total_supply: Default::default(),
            }),
            _ => panic!("Unsupported query"),
        }),
    );

    exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_OWNER, &[]),
        default_msg(),
    )
    .unwrap();

    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap().reward_rate,
        Decimal::from_ratio(1000u128 * 10u128.pow(6u32), 100u128)
    );
}
