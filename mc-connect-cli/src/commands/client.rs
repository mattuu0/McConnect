use anyhow::{Context, Result};
use log::{error, info};
use mc_connect_core::WsClientService;
use mc_connect_core::encryption::RsaKeyPair;
use mc_connect_core::models::packet::{ClientExportConfig, Protocol};
use std::sync::Arc;

pub async fn run_client(
    local_port: u16,
    remote_port: u16,
    protocol_str: String,
    ws_url: Option<String>,
    list_ports: bool,
    public_key: Option<String>,
    config: Option<String>,
) -> Result<()> {
    let mut final_ws_url = ws_url;
    let mut final_pub_key = public_key;

    // 設定ファイルからの読み込み
    if let Some(path) = config {
        let content = std::fs::read_to_string(&path)
            .context(format!("設定ファイル {} の読み取りに失敗しました", path))?;
        let cfg: ClientExportConfig = serde_json::from_str(&content)
            .context("設定ファイルのフォーマットが正しくありません")?;

        if final_ws_url.is_none() {
            final_ws_url = Some(cfg.ws_url);
        }
        if final_pub_key.is_none() {
            final_pub_key = Some(cfg.public_key);
        }
    }

    let ws_url_str =
        final_ws_url.ok_or_else(|| anyhow::anyhow!("--ws-url または --config が必要です"))?;
    let pub_key_str =
        final_pub_key.ok_or_else(|| anyhow::anyhow!("--public-key または --config が必要です"))?;

    let proto = match protocol_str.to_lowercase().as_str() {
        "tcp" => Protocol::TCP,
        "udp" => Protocol::UDP,
        _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", protocol_str)),
    };

    if list_ports {
        info!("Fetching allowed ports from {}...", ws_url_str);
        match WsClientService::get_server_info(&ws_url_str).await {
            Ok(info) => {
                println!("Server Version: {}", info.server_version);
                println!("Allowed Ports:");
                for p in info.allowed_ports {
                    println!("  - {}: {:?}", p.port, p.protocol);
                }
            }
            Err(e) => {
                error!("Failed to fetch server info: {}", e);
            }
        }
        return Ok(());
    }

    let pub_key_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, pub_key_str)
            .context("公開鍵の Base64 デコードに失敗しました")?;

    let rsa_pub_key = Arc::new(
        RsaKeyPair::from_public_der(&pub_key_bytes)
            .map_err(|e| anyhow::anyhow!("公開鍵の読み込みに失敗しました: {}", e))?,
    );

    info!(
        "Starting secure client tunnel: local {} -> ws {} -> remote {} ({:?})",
        local_port, ws_url_str, remote_port, proto
    );
    let stats = Arc::new(mc_connect_core::services::ws_client::TunnelStats::new());
    let (_ping_tx, ping_rx) = tokio::sync::mpsc::unbounded_channel();

    WsClientService::start_tunnel_with_protocol(
        "127.0.0.1".into(),
        local_port,
        ws_url_str,
        remote_port,
        proto,
        stats,
        ping_rx,
        rsa_pub_key,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Client error: {}", e))?;

    Ok(())
}
