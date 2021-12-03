use cosmwasm_std::{Env, MessageInfo};

use crate::executions::swap::claim;
use crate::executions::ExecuteResult;
use crate::testing::MockDeps;

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo) -> ExecuteResult {
    claim(deps.as_mut(), env, info)
}

#[test]
fn success() {}
