use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Env, MessageInfo};

use crate::executions::swap::claim;
use crate::executions::ExecuteResult;
use crate::testing::{instantiate, mock_deps, MockDeps, TEST_USER_1, TEST_USER_2};

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo) -> ExecuteResult {
    claim(deps.as_mut(), env, info)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);

    exec(&mut deps, mock_env(), mock_info(TEST_USER_1, &[])).unwrap();
    exec(&mut deps, mock_env(), mock_info(TEST_USER_2, &[])).unwrap();
}
