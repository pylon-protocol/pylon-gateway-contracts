pub mod config;
pub mod state;
pub mod user;

pub static KEY_STATE: &[u8] = b"state";
pub static KEY_CONFIG: &[u8] = b"config";
pub static PREFIX_USER: &[u8] = b"user";
