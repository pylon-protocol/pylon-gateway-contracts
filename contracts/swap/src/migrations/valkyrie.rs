use cosmwasm_std::{
    CanonicalAddr, Decimal, DepsMut, Env, Order, Response, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use cw20::Denom;
use pylon_gateway::swap_types;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::states::config::Config;
use crate::states::state::State;
use crate::states::user::User;
use crate::types::distribution_strategy::DistributionStrategy;

static KEY_STATE: &[u8] = b"state";
static KEY_CONFIG: &[u8] = b"config";
static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub owner: String,
    pub beneficiary: String,
    pub price: Decimal,
    pub start: u64,
    pub finish: u64,
    pub cap_strategy: Option<String>,
    pub distribution_strategy: Vec<swap_types::DistributionStrategy>,
    pub whitelist_enabled: bool,
    pub swap_pool_size: Uint128,
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
pub struct LegacyState {
    pub total_swapped: Uint128,
    pub total_claimed: Uint128,

    pub x_denom: String,
    pub y_addr: String,
    pub liq_x: Uint128,
    pub liq_y: Uint128,
}

impl LegacyState {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        ReadonlySingleton::<Self>::new(storage, KEY_STATE).load()
    }

    pub fn save(storage: &mut dyn Storage, data: &Self) -> StdResult<()> {
        Singleton::<Self>::new(storage, KEY_STATE).save(data)
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyUser {
    pub whitelisted: bool,
    pub swapped_in: Uint128,
    pub swapped_out: Uint128,
    pub swapped_out_claimed: Uint128,
}

impl LegacyUser {
    pub fn load(storage: &dyn Storage, owner: &CanonicalAddr) -> Self {
        ReadonlyBucket::<Self>::new(storage, PREFIX_USER)
            .load(owner.as_slice())
            .unwrap_or_default()
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

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    deposit_cap_strategy: Option<swap_types::CapStrategy>,
) -> super::MigrateResult {
    let api = deps.api;
    let storage = deps.storage;
    let legacy_config = LegacyConfig::load(storage)?;
    let legacy_state = LegacyState::load(storage)?;

    Config::save(
        storage,
        &Config {
            owner: api.addr_validate(legacy_config.owner.as_str())?,
            beneficiary: api.addr_validate(legacy_config.beneficiary.as_str())?,

            start: legacy_config.start,
            finish: legacy_config.finish,
            price: legacy_config.price,
            amount: legacy_config.swap_pool_size,
            input_token: Denom::Native(legacy_state.x_denom),
            output_token: Denom::Cw20(api.addr_validate(legacy_state.y_addr.as_str())?),

            deposit_cap_strategy: deposit_cap_strategy.map(|x| x.into()),
            distribution_strategies: legacy_config
                .distribution_strategy
                .iter()
                .map(|x| DistributionStrategy::from(x.clone()))
                .collect(),
            whitelist_enabled: legacy_config.whitelist_enabled,
        },
    )?;

    State::save(
        storage,
        &State {
            total_swapped: legacy_state.total_swapped,
            total_claimed: legacy_state.total_claimed,

            x_liquidity: legacy_state.liq_x,
            y_liquidity: legacy_state.liq_y,
        },
    )?;

    for (owner, user) in LegacyUser::load_range(storage).iter() {
        User::save(
            storage,
            owner,
            &User {
                swapped_in: user.swapped_in,
                swapped_out: user.swapped_out,
                swapped_out_claimed: user.swapped_out_claimed,
            },
        )?;
        if user.whitelisted {
            User::register_whitelist(storage, owner)?;
        } else {
            User::unregister_whitelist(storage, owner)?;
        }
    }

    Ok(Response::default())
}
