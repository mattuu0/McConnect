use anyhow::{Result, Context};
use mc_connect_core::start_server;
use mc_connect_core::encryption::{RsaKeyGenerator, KeyGenerator, RsaKeyPair};
use mc_connect_core::models::packet::ServerExportConfig;
use log::info;
use std::sync::Arc;
use crate::utils::parse_allowed_ports;

pub async fn run_server(
    host: String,
    public_host: Option<String>,
    port: u16,
    allowed_ports_str: String,
    export: Option<String>,
    _key_pair_path: Option<String>,
) -> Result<()> {
    let parsed_ports = parse_allowed_ports(&allowed_ports_str)?;
    
    info!("RSA キーペアを生成中...");
    let generator = RsaKeyGenerator::default();
    let key_pair = generator.generate().map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
    
    let pub_key_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key_pair.public_key_bytes());
    
    // 設定のエクスポート
    if let Some(path) = export {
        let export_data = ServerExportConfig {
            host: public_host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port,
            public_key: pub_key_b64.clone(),
        };
        let json = serde_json::to_string_pretty(&export_data)?;
        std::fs::write(&path, json).context(format!("設定ファイル {} の書き込みに失敗しました", path))?;
        info!("サーバー構成ファイルを書き出しました: {}", path);
    }

    info!("====================================================");
    info!("サーバーの公開鍵:");
    info!("{}", pub_key_b64);
    info!("====================================================");

    let rsa_pair = Arc::new(RsaKeyPair::from_private_der(&key_pair.private_key_bytes()).unwrap());

    info!("Starting server on {}:{}", host, port);
    start_server(&host, port, parsed_ports, rsa_pair).await.map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
    
    Ok(())
}
