use cosmwasm_std::Response;

use crate::error::ContractError;

pub mod legacy;

pub type MigrateResult = Result<Response, ContractError>;
