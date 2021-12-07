use cosmwasm_std::Response;

use crate::error::ContractError;

pub mod config;
pub mod state;
pub mod swap;
pub mod user;

pub type ExecuteResult = Result<Response, ContractError>;
