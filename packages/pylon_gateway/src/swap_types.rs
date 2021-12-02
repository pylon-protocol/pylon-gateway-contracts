use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapStrategy {
    Fixed {
        min_user_cap: Option<Uint128>,
        max_user_cap: Option<Uint128>,
    },
    GovFixed {
        contract: String,
        min_stake_amount: Uint128,
        min_user_cap: Option<Uint128>,
        max_user_cap: Option<Uint128>,
    },
    GovLinear {
        contract: String,
        cap_start: Uint128,
        cap_weight: Decimal,
        min_stake_amount: Option<Uint128>,
        max_stake_amount: Option<Uint128>,
    },
    GovStaged {
        contract: String,
        // 1. from
        // 2. to
        // 3. applied_cap
        stages: Vec<(Option<Uint128>, Option<Uint128>, Uint128)>,
    },
    GovLinearStaged {
        contract: String,
        // 1. from
        // 2. to
        // 3. cap_start
        // 4. cap_weight
        stages: Vec<(Option<Uint128>, Option<Uint128>, Uint128, Decimal)>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionStrategy {
    Lockup {
        release_time: u64,
        release_amount: Decimal,
    },
    Vesting {
        release_start_time: u64,
        release_finish_time: u64,
        release_amount: Decimal,
    },
}
