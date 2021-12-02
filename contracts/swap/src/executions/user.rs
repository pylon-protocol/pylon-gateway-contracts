use crate::states::user::User;
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

pub fn whitelist(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    whitelist: bool,
    candidates: Vec<String>,
) -> super::ExecuteResult {
    candidates
        .iter()
        .map(|x| deps.api.addr_canonicalize(x.as_str()).unwrap())
        .for_each(|candidate| {
            let mut user = User::load(deps.storage, &candidate);
            user.whitelisted = whitelist;
            User::save(deps.storage, &candidate, &user).unwrap();
        });

    Ok(Response::new().add_attributes(vec![attr("action", "whitelist_user")]))
}
