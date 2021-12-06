use cosmwasm_std::{CanonicalAddr, Decimal, DepsMut, Env, Order, Response, Uint128};
use cosmwasm_storage::{ReadonlyBucket, ReadonlySingleton};
use cw20::Denom;
use pylon_gateway::swap_types;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::states::config::Config;
use crate::states::state::State;
use crate::states::user::User;
use crate::states::{KEY_CONFIG, KEY_STATE, PREFIX_USER};
use crate::types::distribution_strategy::DistributionStrategy;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyState {
    pub total_swapped: Uint128,
    pub total_claimed: Uint128,

    pub x_denom: String,
    pub y_addr: String,
    pub liq_x: Uint128,
    pub liq_y: Uint128,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyUser {
    pub whitelisted: bool,
    pub swapped_in: Uint128,
    pub swapped_out: Uint128,
    pub swapped_out_claimed: Uint128,
}

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    deposit_cap_strategy: Option<swap_types::CapStrategy>,
) -> super::MigrateResult {
    let api = deps.api;
    let storage = deps.storage;
    let legacy_config = ReadonlySingleton::<LegacyConfig>::new(storage, KEY_CONFIG).load()?;
    let legacy_state = ReadonlySingleton::<LegacyState>::new(storage, KEY_STATE).load()?;

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

            deposit_cap_strategy: deposit_cap_strategy.map(|x| x.into),
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

    let legacy_user: Vec<(CanonicalAddr, LegacyUser)> =
        ReadonlyBucket::<LegacyUser>::new(storage, PREFIX_USER)
            .range(None, None, Order::Descending)
            .map(|item| -> (CanonicalAddr, LegacyUser) {
                let (addr, user) = item.unwrap();
                (CanonicalAddr::from(addr), user)
            })
            .collect();

    for (owner, user) in legacy_user.iter() {
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
