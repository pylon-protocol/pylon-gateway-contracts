use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub beneficiary: String,
    pub start: u64,
    pub finish: u64,
    pub price: Decimal,
    pub total_sale_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalanceOfResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsWhitelistedResponse {
    pub whitelisted: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AvailableCapOfResponse {
    pub amount: Option<Uint128>,
    pub unlimited: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableTokenOfResponse {
    pub amount: Uint128,
    pub remaining: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalSupplyResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentPriceResponse {
    pub price: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateWithdrawResponse {
    pub amount: Uint128,
    pub penalty: Uint128,
    pub withdrawable: bool,
}
