use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, to_binary, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg};
use pylon_gateway::pool_msg::{QueryMsg as PoolQueryMsg, QueryMsg};
use pylon_gateway::pool_resp::StakerResponse;

use crate::entrypoints::query;
use crate::testing::{instantiate, mock_deps, TEST_POOL, TEST_SENDER};

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    const BALANCE: u128 = 1000;

    deps.querier.register_wasm_smart_query_handler(
        TEST_POOL.to_string(),
        Box::new(|x| match from_binary::<PoolQueryMsg>(x).unwrap() {
            QueryMsg::Staker { address } => match address.as_str() {
                TEST_SENDER => to_binary(&StakerResponse {
                    address,
                    staked: Uint128::from(BALANCE),
                    reward: Default::default(),
                    available_cap: None,
                }),
                _ => panic!("Unexpected staker address"),
            },
            _ => panic!("Unsupported query"),
        }),
    );

    let response = query(
        deps.as_ref(),
        mock_env(),
        Cw20QueryMsg::Balance {
            address: TEST_SENDER.to_string(),
        },
    )
    .unwrap();
    let response = from_binary::<BalanceResponse>(&response).unwrap();
    assert_eq!(
        response,
        BalanceResponse {
            balance: Uint128::from(BALANCE),
        }
    );
}
