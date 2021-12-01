use crate::entrypoints::query;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, to_binary, Uint128};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use pylon_gateway::pool_msg::{QueryMsg as PoolQueryMsg, QueryMsg};
use pylon_gateway::pool_resp::RewardResponse;

use crate::testing::{instantiate, mock_deps, TEST_POOL};

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    const TOTAL_DEPOSIT_AMOUNT: u128 = 1000;

    deps.querier.register_wasm_smart_query_handler(
        TEST_POOL.to_string(),
        Box::new(|x| match from_binary::<PoolQueryMsg>(x).unwrap() {
            QueryMsg::Reward {} => to_binary(&RewardResponse {
                total_deposit: Uint128::from(TOTAL_DEPOSIT_AMOUNT),
                last_update_time: 0,
            }),
            _ => panic!("Unsupported query"),
        }),
    );

    let response = query(deps.as_ref(), mock_env(), Cw20QueryMsg::TokenInfo {}).unwrap();
    let response = from_binary::<TokenInfoResponse>(&response).unwrap();
    assert_eq!(
        response,
        TokenInfoResponse {
            name: "Pylon bDP Token for Gateway TRT 5m Pool".to_string(),
            symbol: "bTRTDP-5m".to_string(),
            decimals: 6,
            total_supply: Uint128::from(TOTAL_DEPOSIT_AMOUNT)
        }
    )
}
