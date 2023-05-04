use crate::types::config::BroadcastError;

pub type Result<T> = std::result::Result<T, BroadcastError>;
