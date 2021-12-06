use cosmwasm_std::{CanonicalAddr, Decimal, DepsMut, Env, Order, Response, Uint128};
use cosmwasm_storage::{ReadonlyBucket, ReadonlySingleton};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyVirtualPool {
    pub x_denom: String,
    pub y_addr: CanonicalAddr,
    pub liq_x: Uint128,
    pub liq_y: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyReward {
    pub total_supply: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyUser {
    pub amount: Uint128,
}

pub fn migrate(deps: DepsMut, _env: Env) -> super::MigrateResult {
    let api = deps.api;
    let legacy_config = ReadonlySingleton::<LegacyConfig>::new(deps.storage, KEY_CONFIG).load()?;
    let legacy_vpool =
        ReadonlySingleton::<LegacyVirtualPool>::new(deps.storage, KEY_VPOOL).load()?;
    let legacy_reward = ReadonlySingleton::<LegacyReward>::new(deps.storage, KEY_REWARD).load()?;

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

    let legacy_user: Vec<(CanonicalAddr, LegacyUser)> =
        ReadonlyBucket::<LegacyUser>::new(deps.storage, PREFIX_USER)
            .range(None, None, Order::Descending)
            .map(|item| -> (CanonicalAddr, LegacyUser) {
                let (addr, user) = item.unwrap();
                (CanonicalAddr::from(addr), user)
            })
            .collect();

    for (owner, user) in legacy_user.iter() {
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
