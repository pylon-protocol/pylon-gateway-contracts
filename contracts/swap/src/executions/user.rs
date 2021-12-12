use crate::states::user::User;
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

pub fn whitelist(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    whitelist: bool,
    candidates: Vec<String>,
) -> super::ExecuteResult {
    let api = deps.api;
    let storage = deps.storage;
    candidates
        .iter()
        .map(|x| api.addr_canonicalize(x.as_str()).unwrap())
        .for_each(|candidate| match whitelist {
            true => User::register_whitelist(storage, &candidate).unwrap(),
            false => User::unregister_whitelist(storage, &candidate).unwrap(),
        });

    Ok(Response::new().add_attributes(vec![attr("action", "whitelist_user")]))
}
