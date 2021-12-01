use crate::entrypoints::query;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, to_binary, Uint128};
use cw20::{AllAccountsResponse, Cw20QueryMsg};
use pylon_gateway::pool_msg::{QueryMsg as PoolQueryMsg, QueryMsg};
use pylon_gateway::pool_resp::{StakerResponse, StakersResponse};

use crate::testing::{instantiate, mock_deps, TEST_POOL, TEST_RECIPIENT, TEST_SENDER};

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    const BALANCE_1: u128 = 1000;
    const BALANCE_2: u128 = 2000;

    deps.querier.register_wasm_smart_query_handler(
        TEST_POOL.to_string(),
        Box::new(|x| match from_binary::<PoolQueryMsg>(x).unwrap() {
            QueryMsg::Stakers { .. } => to_binary(&StakersResponse {
                stakers: vec![
                    StakerResponse {
                        address: TEST_SENDER.to_string(),
                        staked: Uint128::from(BALANCE_1),
                        reward: Default::default(),
                    },
                    StakerResponse {
                        address: TEST_RECIPIENT.to_string(),
                        staked: Uint128::from(BALANCE_2),
                        reward: Default::default(),
                    },
                ],
            }),
            _ => panic!("Unsupported query"),
        }),
    );

    let response = query(
        deps.as_ref(),
        mock_env(),
        Cw20QueryMsg::AllAccounts {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let response = from_binary::<AllAccountsResponse>(&response).unwrap();
    assert_eq!(
        response,
        AllAccountsResponse {
            accounts: vec![TEST_SENDER.to_string(), TEST_RECIPIENT.to_string()]
        }
    );
}
