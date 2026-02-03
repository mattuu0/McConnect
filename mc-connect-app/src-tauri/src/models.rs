use mc_connect_core::models::packet::StatsPayload;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingInfo {
    pub id: String,
    pub ws_url: String,
    pub bind_addr: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String,
    pub ping_interval: u64,
    pub public_key: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StartServerConfig {
    pub port: u16,
    pub allowed_ports: Vec<(u16, String)>,
    pub private_key_b64: String,
}

#[derive(Serialize, Clone)]
pub struct TunnelStatus {
    pub id: String,
    pub running: bool,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct StatsEvent {
    pub id: String,
    pub stats: StatsPayload,
}
