use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use pylon_utils::common::OrderBy;
use pylon_utils::range::{calc_range_end_addr, calc_range_start_addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::constant::{DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub amount: Uint128,
    pub reward: Uint128,
    pub reward_per_token_paid: Decimal,
}

impl User {
    pub fn load(storage: &dyn Storage, owner: &CanonicalAddr) -> Self {
        ReadonlyBucket::<User>::new(storage, super::PREFIX_USER)
            .load(owner.as_slice())
            .unwrap_or_default()
    }

    pub fn load_range(
        storage: &dyn Storage,
        start_after: Option<CanonicalAddr>,
        limit: Option<u32>,
        order: Option<OrderBy>,
    ) -> Vec<(CanonicalAddr, Self)> {
        let (start, end, order_by) = match order {
            Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
        };
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

        ReadonlyBucket::<Self>::new(storage, super::PREFIX_USER)
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| -> (CanonicalAddr, Self) {
                let (k, v) = item.unwrap();
                (CanonicalAddr::from(k.as_slice()), v)
            })
            .collect()
    }

    pub fn save(storage: &mut dyn Storage, owner: &CanonicalAddr, user: &Self) -> StdResult<()> {
        Bucket::<User>::new(storage, super::PREFIX_USER).save(owner.as_slice(), user)
    }

    pub fn remove(storage: &mut dyn Storage, owner: &CanonicalAddr) {
        Bucket::<User>::new(storage, super::PREFIX_USER).remove(owner.as_slice())
    }
}
