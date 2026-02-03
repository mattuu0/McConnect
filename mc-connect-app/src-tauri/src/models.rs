use mc_connect_core::models::packet::StatsPayload;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct StartServerConfig {
    pub port: u16,
    pub allowed_ports: Vec<(u16, String)>,
    pub private_key_b64: String,
    pub encryption_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MappingConfig {
    pub id: String,
    pub name: String,
    pub ws_url: String,
    pub bind_addr: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String,
    pub public_key: Option<String>,
    pub ping_interval: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SavedServerConfig {
    pub listen_port: u16,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub encryption_type: String,
    pub allowed_ports: Vec<(u16, String)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub server_mode_enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppPersistConfig {
    pub mappings: Vec<MappingConfig>,
    pub server_config: SavedServerConfig,
    pub app_settings: AppSettings,
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

#[derive(Serialize, Clone)]
pub struct TunnelStatus {
    pub id: String,
    pub running: bool,
    pub message: String,
}
