use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{Env, OwnedDeps, Timestamp};

use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};

mod executions;
mod instantiate;
mod migrations;
mod mock_querier;
mod queries;

const TEST_OWNER: &str = "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v";
const TEST_BENEFICIARY: &str = "terra18wlvftxzj6zt0xugy2lr9nxzu402690ltaf4ss";
const TEST_USER_1: &str = "terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9";
const TEST_USER_2: &str = "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp";
const TEST_INPUT_TOKEN: &str = "uusd";
const TEST_OUTPUT_TOKEN: &str = "terra17tv2hvwpg0ukqgd2y5ct2w54fyan7z0zxrm2f9";

type MockDeps = OwnedDeps<MockStorage, MockApi, WasmMockQuerier>;

fn mock_deps() -> MockDeps {
    mock_dependencies(&[])
}

#[allow(dead_code)]
fn mock_env_height(height: u64, time: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(time);
    env
}
