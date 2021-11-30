use cosmwasm_std::{
    attr, to_binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use std::cmp::{max, min};

use crate::error::ContractError;
use crate::states::config::Config;
use crate::states::reward::Reward;
use crate::states::user::User;

pub fn update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target: Option<String>,
) -> super::ExecuteResult {
    let config = Config::load(deps.storage)?;
    let applicable_reward_time = min(
        max(
            env.block.time.seconds(),
            config.reward_distribution_time.start,
        ),
        config.reward_distribution_time.finish,
    );

    // reward
    let mut reward = Reward::load(deps.storage)?;

    reward.reward_per_token_stored = if reward.last_update_time == applicable_reward_time {
        reward.reward_per_token_stored // because it's already latest
    } else {
        reward.reward_per_token_stored
            + calculate_reward_per_token(&config, &reward, &applicable_reward_time)?
    };
    reward.last_update_time = applicable_reward_time;

    Reward::save(deps.storage, &reward)?;

    // user
    let mut resp = Response::new().add_attributes(vec![
        attr("action", "update"),
        attr("sender", info.sender.to_string()),
        attr("stored_rpt", reward.reward_per_token_stored.to_string()),
    ]);

    if let Some(target) = target {
        let t = deps.api.addr_canonicalize(target.as_str()).unwrap();
        let mut user = User::load(deps.storage, &t);

        user.reward = calculate_rewards(&config, &reward, &user, &applicable_reward_time)?;
        user.reward_per_token_paid = reward.reward_per_token_stored;

        User::save(deps.storage, &t, &user)?;
        resp = resp
            .add_attribute("target", target)
            .add_attribute("reward", user.reward.to_string())
    }

    Ok(resp)
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> super::ExecuteResult {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {
            action: "deposit".to_string(),
            expected: env.contract.address.to_string(),
            actual: info.sender.to_string(),
        });
    }

    let config = Config::load(deps.storage)?;
    config.check_deposit_time(&env)?;

    let mut reward = Reward::load(deps.storage)?;
    let mut user = User::load(deps.storage, &deps.api.addr_canonicalize(sender.as_str())?);

    reward.total_deposit += reward.total_deposit;
    user.amount += amount;

    Reward::save(deps.storage, &reward)?;
    User::save(
        deps.storage,
        &deps.api.addr_canonicalize(sender.as_str())?,
        &user,
    )
    .unwrap();

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", sender)
        .add_attribute("deposit_amount", amount.to_string()))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> super::ExecuteResult {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {
            action: "withdraw".to_string(),
            expected: env.contract.address.to_string(),
            actual: info.sender.to_string(),
        });
    }

    let config = Config::load(deps.storage)?;
    config.check_withdraw_time(&env)?;

    let owner = deps.api.addr_canonicalize(sender.as_str())?;
    let mut reward = Reward::load(deps.storage)?;
    let mut user = User::load(deps.storage, &owner);

    if amount > user.amount {
        return Err(ContractError::WithdrawAmountExceeded { amount });
    }

    reward.total_deposit -= amount;
    user.amount -= amount;

    Reward::save(deps.storage, &reward)?;
    User::save(deps.storage, &owner, &user)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.share_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount,
            })
            .unwrap(),
            funds: vec![],
        }))
        .add_attribute("action", "withdraw")
        .add_attribute("sender", sender)
        .add_attribute("withdraw_amount", amount.to_string()))
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo, sender: String) -> super::ExecuteResult {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {
            action: "claim".to_string(),
            expected: env.contract.address.to_string(),
            actual: info.sender.to_string(),
        });
    }

    let config = Config::load(deps.storage)?;
    config.check_claim_time(&env)?;

    let owner = deps.api.addr_canonicalize(sender.as_str())?;
    let mut user = User::load(deps.storage, &owner);

    let claim_amount = user.reward;
    user.reward = Uint128::zero();
    User::save(deps.storage, &owner, &user)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.reward_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount: claim_amount,
            })
            .unwrap(),
            funds: vec![],
        }))
        .add_attribute("action", "claim")
        .add_attribute("sender", sender)
        .add_attribute("claim_amount", claim_amount.to_string()))
}

pub fn transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> super::ExecuteResult {
    let config = Config::load(deps.storage)?;
    if config.token != info.sender {
        return Err(ContractError::Unauthorized {
            action: "transfer_internal".to_string(),
            expected: config.token.to_string(),
            actual: info.sender.to_string(),
        });
    }

    let owner_addr = deps.api.addr_canonicalize(owner.as_str())?;
    let recipient_addr = deps.api.addr_canonicalize(recipient.as_str())?;

    let mut owner = User::load(deps.storage, &owner_addr);
    let mut recipient = User::load(deps.storage, &recipient_addr);

    if owner.amount < amount {
        return Err(ContractError::TransferAmountExceeded { amount });
    }

    owner.amount -= amount;
    recipient.amount += amount;

    User::save(deps.storage, &owner_addr, &owner)?;
    User::save(deps.storage, &recipient_addr, &recipient)?;

    Ok(Response::new().add_attributes(vec![attr("action", "transfer_internal")]))
}

pub fn calculate_reward_per_token(
    config: &Config,
    reward: &Reward,
    timestamp: &u64,
) -> StdResult<Decimal> {
    let period = Uint128::from(max(timestamp, &reward.last_update_time) - reward.last_update_time);

    if reward.total_deposit.is_zero() {
        Ok(Decimal::zero())
    } else {
        Ok(Decimal::from_ratio(
            config.reward_rate * period,
            reward.total_deposit,
        ))
    }
}

pub fn calculate_rewards(
    config: &Config,
    reward: &Reward,
    user: &User,
    timestamp: &u64,
) -> StdResult<Uint128> {
    let mut rpt = reward.reward_per_token_stored - user.reward_per_token_paid;

    if reward.last_update_time > *timestamp {
        return Err(StdError::generic_err(
            "Gateway/Pool: timestamp must be greater than last update time",
        ));
    }

    if reward.last_update_time != *timestamp {
        rpt = rpt + calculate_reward_per_token(config, reward, timestamp)?;
    }

    Ok(user.reward + (rpt * user.amount))
}
