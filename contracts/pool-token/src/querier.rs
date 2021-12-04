use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use pylon_gateway::pool_msg;
use pylon_gateway::pool_resp;
use pylon_gateway::pool_resp_v2;
use pylon_utils::common::OrderBy;

pub struct Querier<'a> {
    querier: &'a QuerierWrapper<'a>,
}

impl Querier<'_> {
    pub fn new<'a>(querier: &'a QuerierWrapper<'a>) -> Querier<'a> {
        Querier { querier }
    }

    pub fn load_pool_config(&self, pool: &Addr) -> StdResult<pool_resp_v2::ConfigResponse> {
        let pool_config: pool_resp_v2::ConfigResponse = self
            .querier
            .query_wasm_smart(pool, &pool_msg::QueryMsg::ConfigV2 {})?;

        Ok(pool_config)
    }

    pub fn load_pool_reward(&self, pool: &Addr) -> StdResult<pool_resp::RewardResponse> {
        let pool_reward: pool_resp::RewardResponse = self
            .querier
            .query_wasm_smart(pool, &pool_msg::QueryMsg::Reward {})?;

        Ok(pool_reward)
    }

    pub fn load_pool_staker(
        &self,
        pool: &Addr,
        owner: &Addr,
    ) -> StdResult<pool_resp::StakerResponse> {
        let pool_staker: pool_resp::StakerResponse = self.querier.query_wasm_smart(
            pool,
            &pool_msg::QueryMsg::Staker {
                address: owner.to_string(),
            },
        )?;

        Ok(pool_staker)
    }

    pub fn load_pool_stakers(
        &self,
        pool: &Addr,
        start_after: Option<String>,
        limit: Option<u32>,
        order: Option<OrderBy>,
    ) -> StdResult<pool_resp::StakersResponse> {
        let pool_stakers: pool_resp::StakersResponse = self.querier.query_wasm_smart(
            pool,
            &pool_msg::QueryMsg::Stakers {
                start_after,
                limit,
                order,
            },
        )?;

        Ok(pool_stakers)
    }

    pub fn load_token_info(&self, token: &Addr) -> StdResult<TokenInfoResponse> {
        let dp_token_info: TokenInfoResponse = self
            .querier
            .query_wasm_smart(token, &Cw20QueryMsg::TokenInfo {})?;

        Ok(dp_token_info)
    }
}
