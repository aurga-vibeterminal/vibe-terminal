use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::SystemTime;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalAddressSnapshot {
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
}

#[derive(Debug, Clone, Default)]
pub struct DdnsStatus {
    pub enabled: bool,
    pub provider: Option<String>,
    pub hostname: Option<String>,
    pub last_checked_at: Option<SystemTime>,
    pub last_error: Option<String>,
    pub local_snapshot: LocalAddressSnapshot,
    pub ip: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DdnsError {
    #[error("DDNS configuration is incomplete: {0}")]
    IncompleteConfig(String),
    #[error("DDNS provider '{0}' is not supported")]
    UnsupportedProvider(String),
    #[error("DDNS addon execution failed: {0}")]
    AddonExecution(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
