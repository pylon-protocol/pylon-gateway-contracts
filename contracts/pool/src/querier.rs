use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use cw20::{Cw20QueryMsg, TokenInfoResponse};

pub struct Querier<'a> {
    querier: &'a QuerierWrapper<'a>,
}

impl Querier<'_> {
    pub fn new<'a>(querier: &'a QuerierWrapper<'a>) -> Querier<'a> {
        Querier { querier }
    }

    pub fn load_token_info(&self, address: &Addr) -> StdResult<TokenInfoResponse> {
        let token_info: TokenInfoResponse = self
            .querier
            .query_wasm_smart(address, &Cw20QueryMsg::TokenInfo {})?;

        Ok(token_info)
    }
}
