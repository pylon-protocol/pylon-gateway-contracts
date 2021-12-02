use cosmwasm_std::{Decimal, Uint128};
use pylon_gateway::swap_msg::DistributionStrategy as SwapDistributionStrategy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// 1. release_amount
// 2. fulfilled
pub type DistributionStrategyResult = (Decimal, bool);

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

impl From<SwapDistributionStrategy> for DistributionStrategy {
    fn from(strategy: SwapDistributionStrategy) -> Self {
        match strategy {
            DistributionStrategy::Lockup {
                release_time,
                release_amount,
            } => Self::Lockup {
                release_time,
                release_amount,
            },
            DistributionStrategy::Vesting {
                release_start_time,
                release_finish_time,
                release_amount,
            } => Self::Vesting {
                release_start_time,
                release_finish_time,
                release_amount,
            },
        }
    }
}

impl DistributionStrategy {
    pub fn check_release_time(self, time: &u64) -> bool {
        match self {
            DistributionStrategy::Lockup { release_time, .. } => *time <= release_time,
            DistributionStrategy::Vesting {
                release_start_time, ..
            } => *time <= release_start_time,
        }
    }

    pub fn release_amount_at(self, time: &u64) -> DistributionStrategyResult {
        match self {
            Self::Lockup {
                release_time,
                release_amount,
            } => Self::handle_lockup_strategy(time, release_time, release_amount),
            Self::Vesting {
                release_start_time,
                release_finish_time,
                release_amount,
            } => Self::handle_vesting_strategy(
                time,
                release_start_time,
                release_finish_time,
                release_amount,
            ),
        }
    }

    fn handle_lockup_strategy(
        time: &u64,
        release_time: u64,
        release_amount: Decimal,
    ) -> DistributionStrategyResult {
        if time < &release_time {
            (Decimal::zero(), false)
        } else {
            (*release_amount, true)
        }
    }

    fn handle_vesting_strategy(
        time: &u64,
        release_start_time: u64,
        release_finish_time: u64,
        release_amount: Decimal,
    ) -> DistributionStrategyResult {
        if time <= &release_start_time {
            (Decimal::zero(), false)
        } else if release_finish_time < *time {
            (release_amount, true)
        } else {
            (
                Decimal(
                    release_amount * Uint128::from(*time - release_start_time)
                        / Uint128::from(release_finish_time - release_start_time),
                ),
                false,
            )
        }
    }
}
