use cosmwasm_std::{OverflowError, StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error(
        "Gateway/Pool: unauthorized (action: {action:?}, expected: {expected:?}, actual: {actual:?})"
    )]
    Unauthorized {
        action: String,
        expected: String,
        actual: String,
    },

    #[error("Gateway/Pool: Invalid reply ID (ID: {id:?}")]
    InvalidReplyId { id: u64 },

    #[error("Gateway/Pool: unsupported receive message. (type: {typ:?})")]
    UnsupportedReceiveMsg { typ: String },

    #[error("Gateway/Pool: invalid deposit time.")]
    InvalidDepositTime {},

    #[error("Gateway/Pool: invalid withdraw time.")]
    InvalidWithdrawTime {},

    #[error("Gateway/Pool: invalid claim time.")]
    InvalidClaimTime {},

    #[error("Gateway/Pool: deposit user cap exceeded. (cap: {cap:?})")]
    DepositUserCapExceeded { cap: Uint128 },

    #[error("Gateway/Pool: deposit total cap exceeded. (cap: {cap:?})")]
    DepositTotalCapExceeded { cap: Uint128 },

    #[error("Gateway/Pool: withdraw amount exceeds balance. (balance: {amount:?})")]
    WithdrawAmountExceeded { amount: Uint128 },

    #[error("Gateway/Pool: transfer amount exceeds balance. (balance: {amount:?})")]
    TransferAmountExceeded { amount: Uint128 },

    #[error("Gateway/Pool: sale finished. (now: {now:?}, finished: {finished:?})")]
    SaleFinished { now: u64, finished: u64 },

    #[error("Gateway/Pool: withdraw strategy length exceeds limit. (limit: {limit:?}, length: {length:?})")]
    WithdrawStrategyLengthExceeded { limit: usize, length: usize },

    #[error("Gateway/Pool: invalid contract version for migration.")]
    InvalidContractVersionForMigration {},
}
