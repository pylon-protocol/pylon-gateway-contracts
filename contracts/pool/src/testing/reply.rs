use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{
    attr, Api, Binary, ContractResult, Env, Reply, Response, SubMsgExecutionResponse,
};
use protobuf::Message;

use crate::constants::INSTANTIATE_REPLY_ID;
use crate::entrypoints::reply;
use crate::executions::ExecuteResult;
use crate::response::MsgInstantiateContractResponse;
use crate::states::config::Config;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_TOKEN};

pub fn exec(deps: &mut MockDeps, env: Env, msg: Reply) -> ExecuteResult {
    reply(deps.as_mut(), env, msg)
}

pub fn default(deps: &mut MockDeps) -> Response {
    exec(
        deps,
        mock_env(),
        Reply {
            id: INSTANTIATE_REPLY_ID,
            result: ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![],
                data: Some(Binary::from(
                    Message::write_to_bytes(&MsgInstantiateContractResponse {
                        contract_address: TEST_TOKEN.to_string(),
                        data: vec![],
                        unknown_fields: Default::default(),
                        cached_size: Default::default(),
                    })
                    .unwrap(),
                )),
            }),
        },
    )
    .unwrap()
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let res = default(&mut deps);
    assert_eq!(res.attributes, vec![attr("action", "register_token")]);

    let api = deps.api;
    assert_eq!(
        Config::load(deps.as_ref().storage).unwrap().token,
        api.addr_validate(TEST_TOKEN).unwrap()
    )
}
