use cosmwasm_std::Response;

use crate::error::ContractError;

pub mod nexus;
pub mod pylon;
pub mod valkyrie;

pub type MigrateResult = Result<Response, ContractError>;
