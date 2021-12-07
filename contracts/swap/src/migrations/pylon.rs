use cosmwasm_std::{
    CanonicalAddr, Decimal, DepsMut, Env, Order, Response, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::states::config::Config;
use crate::states::state::State;
use crate::states::user::User;
use crate::types::distribution_strategy::DistributionStrategy;

static KEY_CONFIG: &[u8] = b"config";
static KEY_VPOOL: &[u8] = b"vpool";
static KEY_REWARD: &[u8] = b"reward";
static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub this: String,
    pub owner: String,
    pub beneficiary: String,
    pub start: u64,
    pub finish: u64,
    pub price: Decimal,
    pub max_cap: Uint128,
    pub total_sale_amount: Uint128,
}

impl LegacyConfig {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Self>::new(storage, KEY_CONFIG).load()
    }

    pub fn save(storage: &mut dyn Storage, data: &Self) -> StdResult<()> {
        Singleton::<Self>::new(storage, KEY_CONFIG).save(data)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyVirtualPool {
    pub x_denom: String,
    pub y_addr: CanonicalAddr,
    pub liq_x: Uint128,
    pub liq_y: Uint128,
}

impl LegacyVirtualPool {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Self>::new(storage, KEY_VPOOL).load()
    }

    pub fn save(storage: &mut dyn Storage, data: &Self) -> StdResult<()> {
        Singleton::<Self>::new(storage, KEY_VPOOL).save(data)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyReward {
    pub total_supply: Uint128,
}

impl LegacyReward {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Self>::new(storage, KEY_REWARD).load()
    }

    pub fn save(storage: &mut dyn Storage, data: &Self) -> StdResult<()> {
        Singleton::<Self>::new(storage, KEY_REWARD).save(data)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyUser {
    pub amount: Uint128,
}

impl LegacyUser {
    pub fn load(storage: &dyn Storage, owner: &CanonicalAddr) -> Self {
        ReadonlyBucket::<Self>::new(storage, PREFIX_USER)
            .load(owner.as_slice())
            .unwrap()
    }

    pub fn load_range(storage: &dyn Storage) -> Vec<(CanonicalAddr, Self)> {
        ReadonlyBucket::<Self>::new(storage, PREFIX_USER)
            .range(None, None, Order::Ascending)
            .map(|item| -> (CanonicalAddr, Self) {
                let (k, v) = item.unwrap();
                (CanonicalAddr::from(k.as_slice()), v)
            })
            .collect()
    }

    pub fn save(storage: &mut dyn Storage, owner: &CanonicalAddr, user: &Self) -> StdResult<()> {
        Bucket::<Self>::new(storage, PREFIX_USER).save(owner.as_slice(), user)
    }
}

pub fn migrate(deps: DepsMut, _env: Env) -> super::MigrateResult {
    let api = deps.api;
    let legacy_config = LegacyConfig::load(deps.storage)?;
    let legacy_vpool = LegacyVirtualPool::load(deps.storage)?;
    let legacy_reward = LegacyReward::load(deps.storage)?;

    Config::save(
        deps.storage,
        &Config {
            owner: api.addr_validate(legacy_config.owner.as_str())?,
            beneficiary: api.addr_validate(legacy_config.beneficiary.as_str())?,
            start: legacy_config.start,
            finish: legacy_config.finish,
            price: legacy_config.price,
            amount: legacy_config.total_sale_amount,
            input_token: Denom::Native(legacy_vpool.x_denom),
            output_token: Denom::Cw20(api.addr_humanize(&legacy_vpool.y_addr)?),
            deposit_cap_strategy: None,
            distribution_strategies: vec![DistributionStrategy::Lockup {
                release_time: legacy_config.finish,
                release_amount: Decimal::one(),
            }],
            whitelist_enabled: false,
        },
    )?;

    State::save(
        deps.storage,
        &State {
            total_swapped: legacy_config.total_sale_amount,
            total_claimed: legacy_config.total_sale_amount - legacy_reward.total_supply,
            x_liquidity: legacy_vpool.liq_x,
            y_liquidity: legacy_vpool.liq_y,
        },
    )?;

    for (owner, user) in LegacyUser::load_range(deps.storage).iter() {
        User::save(
            deps.storage,
            owner,
            &User {
                swapped_in: user.amount * legacy_config.price,
                swapped_out: user.amount,
                swapped_out_claimed: Uint128::zero(),
            },
        )?;
    }

    Ok(Response::default())
}
