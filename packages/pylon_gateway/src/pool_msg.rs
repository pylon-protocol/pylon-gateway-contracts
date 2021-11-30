use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use pylon_utils::common::OrderBy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub pool_token_code_id: u64,
    // pool
    pub share_token: String,
    pub deposit_time: Vec<(u64, u64, bool)>,
    pub withdraw_time: Vec<(u64, u64, bool)>,
    pub deposit_cap_strategy: Option<String>,
    // reward
    pub reward_token: String,
    pub reward_amount: Uint128,
    pub reward_claim_time: Vec<(u64, u64, bool)>,
    pub reward_distribution_time: (u64, u64),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigureMsg {
    Config {
        owner: Option<String>,
        share_token: Option<String>,
        reward_token: Option<String>,
        claim_time: Option<Vec<(u64, u64, bool)>>,
        deposit_time: Option<Vec<(u64, u64, bool)>>,
        withdraw_time: Option<Vec<(u64, u64, bool)>>,
        deposit_cap_strategy: Option<String>,
    },
    SubReward {
        amount: Uint128,
    },
    AddReward {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // core
    Receive(Cw20ReceiveMsg),
    Update {
        target: Option<String>,
    },
    Withdraw {
        amount: Uint128,
    },
    Claim {
        target: Option<String>,
    },
    // internal
    TransferInternal {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    DepositInternal {
        sender: String,
        amount: Uint128,
    },
    WithdrawInternal {
        sender: String,
        amount: Uint128,
    },
    ClaimInternal {
        sender: String,
    },
    // owner
    Configure(ConfigureMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {}, // state::Config
    Reward {}, // state::Reward
    Staker {
        address: String,
    },
    Stakers {
        start_after: Option<String>,
        limit: Option<u32>,
        order: Option<OrderBy>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
