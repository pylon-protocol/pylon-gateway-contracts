use cosmwasm_std::{Decimal, QuerierWrapper, Uint128};
use pylon_gateway::swap_types;
use pylon_token::gov_msg;
use pylon_token::gov_resp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

pub type CapStrategyResult = (Uint128, bool);

pub const ERROR: CapStrategyResult = (Uint128::zero(), false);
pub const UNLIMITED: CapStrategyResult = (Uint128::zero(), true);

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

impl CapStrategy {
    // 1. amount
    // 2. unlimited
    pub fn available_cap_of(
        self,
        querier: QuerierWrapper,
        address: String,
        amount: Uint128,
    ) -> CapStrategyResult {
        match self {
            Self::Fixed {
                min_user_cap,
                max_user_cap,
            } => Self::handle_fixed_strategy(querier, address, amount, min_user_cap, max_user_cap),
            Self::GovFixed {
                contract,
                min_stake_amount,
                min_user_cap,
                max_user_cap,
            } => Self::handle_gov_fixed_strategy(
                querier,
                address,
                amount,
                contract,
                min_stake_amount,
                min_user_cap,
                max_user_cap,
            ),
            Self::GovLinear {
                contract,
                cap_start,
                cap_weight,
                min_stake_amount,
                max_stake_amount,
            } => Self::handle_gov_linear_strategy(
                querier,
                address,
                amount,
                contract,
                cap_start,
                cap_weight,
                min_stake_amount,
                max_stake_amount,
            ),
            Self::GovStaged { contract, stages } => {
                Self::handle_gov_staged_strategy(querier, address, amount, contract, stages)
            }
            Self::GovLinearStaged { contract, stages } => {
                Self::handle_gov_linear_staged_strategy(querier, address, amount, contract, stages)
            }
        }
    }

    fn handle_fixed_strategy(
        _querier: QuerierWrapper,
        _address: String,
        amount: Uint128,
        min_user_cap: Option<Uint128>,
        max_user_cap: Option<Uint128>,
    ) -> CapStrategyResult {
        if amount < min_user_cap.unwrap_or_else(Uint128::zero) {
            return ERROR;
        }

        match max_user_cap {
            Some(cap) => {
                if cap < amount {
                    ERROR
                } else {
                    (cap - amount, false)
                }
            }
            None => UNLIMITED,
        }
    }

    fn handle_gov_fixed_strategy(
        querier: QuerierWrapper,
        address: String,
        amount: Uint128,
        contract: String,
        min_stake_amount: Uint128,
        min_user_cap: Option<Uint128>,
        max_user_cap: Option<Uint128>,
    ) -> CapStrategyResult {
        let staker: gov_resp::StakerResponse = querier
            .query_wasm_smart(contract, &gov_msg::QueryMsg::Staker { address })
            .unwrap();
        if staker.balance < min_stake_amount {
            return ERROR;
        }

        if amount < min_user_cap.unwrap_or_else(Uint128::zero) {
            return ERROR;
        }

        match max_user_cap {
            Some(cap) => {
                if cap < amount {
                    ERROR
                } else {
                    (cap - amount, false)
                }
            }
            None => UNLIMITED,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_gov_linear_strategy(
        querier: QuerierWrapper,
        address: String,
        amount: Uint128,
        contract: String,
        cap_start: Uint128,
        cap_weight: Decimal,
        min_stake_amount: Option<Uint128>,
        max_stake_amount: Option<Uint128>,
    ) -> CapStrategyResult {
        let staker: gov_resp::StakerResponse = querier
            .query_wasm_smart(contract, &gov_msg::QueryMsg::Staker { address })
            .unwrap();

        let min_stake_amount = min_stake_amount.unwrap_or_else(Uint128::zero);
        if staker.balance < min_stake_amount {
            return ERROR;
        }

        match max_stake_amount {
            Some(max_stake_amount) => {
                let dx = min(max_stake_amount, staker.balance) - min_stake_amount;
                let cap = cap_start + (dx * cap_weight);

                if cap < amount {
                    ERROR
                } else {
                    (cap - amount, false)
                }
            }
            None => UNLIMITED,
        }
    }

    fn handle_gov_staged_strategy(
        querier: QuerierWrapper,
        address: String,
        amount: Uint128,
        contract: String,
        stages: Vec<(Option<Uint128>, Option<Uint128>, Uint128)>,
    ) -> CapStrategyResult {
        let staker: gov_resp::StakerResponse = querier
            .query_wasm_smart(contract, &gov_msg::QueryMsg::Staker { address })
            .unwrap();

        let mut cap = Uint128::zero();
        for (from, to, applied_cap) in stages.iter() {
            let from = from.unwrap_or(Uint128::zero());
            cap = max(
                cap,
                if from <= staker.balance {
                    match to {
                        Some(to) => {
                            if staker.balance < *to {
                                *applied_cap
                            } else {
                                Uint128::zero()
                            }
                        }
                        None => *applied_cap,
                    }
                } else {
                    Uint128::zero()
                },
            );
        }

        if cap < amount {
            ERROR
        } else {
            (cap - amount, false)
        }
    }

    fn handle_gov_linear_staged_strategy(
        querier: QuerierWrapper,
        address: String,
        amount: Uint128,
        contract: String,
        stages: Vec<(Option<Uint128>, Option<Uint128>, Uint128, Decimal)>,
    ) -> CapStrategyResult {
        let staker: gov_resp::StakerResponse = querier
            .query_wasm_smart(contract, &gov_msg::QueryMsg::Staker { address })
            .unwrap();

        let mut cap = Uint128::zero();
        for (from, to, cap_start, cap_weight) in stages.iter() {
            let from = from.unwrap_or(Uint128::zero());
            cap = max(
                cap,
                if from <= staker.balance {
                    match to {
                        Some(to) => {
                            let dx = min(*to, staker.balance) - from;
                            *cap_start + (*cap_weight * dx)
                        }
                        None => return UNLIMITED, // unlimited
                    }
                } else {
                    Uint128::zero()
                },
            );
        }

        if cap < amount {
            ERROR
        } else {
            (cap - amount, false)
        }
    }
}

impl From<swap_types::CapStrategy> for CapStrategy {
    fn from(strategy: swap_types::CapStrategy) -> Self {
        match strategy {
            swap_types::CapStrategy::Fixed {
                min_user_cap,
                max_user_cap,
            } => Self::Fixed {
                min_user_cap,
                max_user_cap,
            },
            swap_types::CapStrategy::GovFixed {
                contract,
                min_stake_amount,
                min_user_cap,
                max_user_cap,
            } => Self::GovFixed {
                contract,
                min_stake_amount,
                min_user_cap,
                max_user_cap,
            },
            swap_types::CapStrategy::GovLinear {
                contract,
                cap_start,
                cap_weight,
                min_stake_amount,
                max_stake_amount,
            } => Self::GovLinear {
                contract,
                cap_start,
                cap_weight,
                min_stake_amount,
                max_stake_amount,
            },
            swap_types::CapStrategy::GovStaged { contract, stages } => {
                Self::GovStaged { contract, stages }
            }
            swap_types::CapStrategy::GovLinearStaged { contract, stages } => {
                Self::GovLinearStaged { contract, stages }
            }
        }
    }
}

impl From<CapStrategy> for swap_types::CapStrategy {
    fn from(strategy: CapStrategy) -> Self {
        match strategy {
            CapStrategy::Fixed {
                min_user_cap,
                max_user_cap,
            } => Self::Fixed {
                min_user_cap,
                max_user_cap,
            },
            CapStrategy::GovFixed {
                contract,
                min_stake_amount,
                min_user_cap,
                max_user_cap,
            } => Self::GovFixed {
                contract,
                min_stake_amount,
                min_user_cap,
                max_user_cap,
            },
            CapStrategy::GovLinear {
                contract,
                cap_start,
                cap_weight,
                min_stake_amount,
                max_stake_amount,
            } => Self::GovLinear {
                contract,
                cap_start,
                cap_weight,
                min_stake_amount,
                max_stake_amount,
            },
            CapStrategy::GovStaged { contract, stages } => Self::GovStaged { contract, stages },
            CapStrategy::GovLinearStaged { contract, stages } => {
                Self::GovLinearStaged { contract, stages }
            }
        }
    }
}
