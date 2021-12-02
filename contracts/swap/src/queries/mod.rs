use cosmwasm_std::Binary;

use crate::error::ContractError;

pub mod config;
pub mod state;
pub mod user;

pub type QueryResult = Result<Binary, ContractError>;
