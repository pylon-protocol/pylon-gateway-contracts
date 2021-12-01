use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use pylon_gateway::pool_msg::QueryMsg as PoolQueryMsg;
use pylon_gateway::pool_resp::{
    ConfigResponse as PoolConfigResponse, RewardResponse as PoolRewardResponse,
    StakerResponse as PoolStakerResponse, StakersResponse as PoolStakersResponse,
};
use pylon_utils::common::OrderBy;

pub struct Querier<'a> {
    querier: &'a QuerierWrapper<'a>,
}

impl Querier<'_> {
    pub fn new<'a>(querier: &'a QuerierWrapper<'a>) -> Querier<'a> {
        Querier { querier }
    }

    pub fn load_pool_config(&self, pool: &Addr) -> StdResult<PoolConfigResponse> {
        let pool_config: PoolConfigResponse = self
            .querier
            .query_wasm_smart(pool, &PoolQueryMsg::Config {})?;

        Ok(pool_config)
    }

    pub fn load_pool_reward(&self, pool: &Addr) -> StdResult<PoolRewardResponse> {
        let pool_reward: PoolRewardResponse = self
            .querier
            .query_wasm_smart(pool, &PoolQueryMsg::Reward {})?;

        Ok(pool_reward)
    }

    pub fn load_pool_staker(&self, pool: &Addr, owner: &Addr) -> StdResult<PoolStakerResponse> {
        let pool_staker: PoolStakerResponse = self.querier.query_wasm_smart(
            pool,
            &PoolQueryMsg::Staker {
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
    ) -> StdResult<PoolStakersResponse> {
        let pool_stakers: PoolStakersResponse = self.querier.query_wasm_smart(
            pool,
            &PoolQueryMsg::Stakers {
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
