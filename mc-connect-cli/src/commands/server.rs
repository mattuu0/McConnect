use crate::utils::parse_allowed_ports;
use anyhow::{Context, Result};
use log::info;
use mc_connect_core::encryption::{CryptoKeyPair, KeyGenerator, RsaKeyGenerator, RsaKeyPair};
use mc_connect_core::models::packet::ServerExportConfig;
use mc_connect_core::start_server;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

pub async fn run_server(
    host: String,
    public_host: Option<String>,
    port: u16,
    allowed_ports_str: String,
    export: Option<String>,
    key_pair_path: Option<String>,
) -> Result<()> {
    let parsed_ports = parse_allowed_ports(&allowed_ports_str)?;

    // 鍵の読み込みまたは生成
    let key_pair_obj: RsaKeyPair = if let Some(ref path_str) = key_pair_path {
        let path = Path::new(path_str);
        if path.exists() {
            info!("既存のキーペアを読み込んでいます: {}", path_str);
            let bytes = fs::read(path).await.context(format!(
                "キーファイル {} の読み込みに失敗しました",
                path_str
            ))?;
            RsaKeyPair::from_private_der(&bytes)
                .map_err(|e| anyhow::anyhow!("キーのパースに失敗しました: {}", e))?
        } else {
            info!("新しいキーペアを生成しています...");
            let generator = RsaKeyGenerator::default();
            let kp = generator
                .generate()
                .map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
            let priv_bytes = kp.private_key_bytes();

            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .await
                        .context("キーファイルの保存先ディレクトリ作成に失敗しました")?;
                }
            }

            info!("キーペアを保存しています: {}", path_str);
            fs::write(path, &priv_bytes).await.context(format!(
                "キーファイル {} の書き込みに失敗しました",
                path_str
            ))?;
            // Reconstruct RsaKeyPair from bytes to ensure consistency
            RsaKeyPair::from_private_der(&priv_bytes).unwrap()
        }
    } else {
        info!("一時的なキーペアを生成しています (ファイルには保存されません)...");
        let generator = RsaKeyGenerator::default();
        let kp = generator
            .generate()
            .map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
        RsaKeyPair::from_private_der(&kp.private_key_bytes()).unwrap()
    };

    let pub_key_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        key_pair_obj.public_key_bytes(),
    );

    // 設定のエクスポート
    if let Some(path) = export {
        let export_data = ServerExportConfig {
            host: public_host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port,
            public_key: pub_key_b64.clone(),
            allowed_ports: Some(allowed_ports_str.clone()),
        };
        let json = serde_json::to_string_pretty(&export_data)?;
        fs::write(&path, json)
            .await
            .context(format!("設定ファイル {} の書き込みに失敗しました", path))?;
        info!("サーバー構成ファイルを書き出しました: {}", path);
    }

    info!("====================================================");
    info!("サーバーの公開鍵:");
    info!("{}", pub_key_b64);
    info!("====================================================");

    let rsa_pair = Arc::new(key_pair_obj);

    info!("Starting server on {}:{}", host, port);
    start_server(&host, port, parsed_ports, rsa_pair)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
