pub const CONTRACT_NAME: &str = "crates.io:pylon-gateway-pool";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// reply
pub const INSTANTIATE_REPLY_ID: u64 = 1;

// pagination
pub const MAX_QUERY_LIMIT: u32 = 30;
pub const DEFAULT_QUERY_LIMIT: u32 = 10;
