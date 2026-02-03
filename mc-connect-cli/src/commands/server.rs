use crate::utils::parse_allowed_ports;
use anyhow::{Context, Result};
use log::info;
use mc_connect_core::encryption::{CryptoKeyPair, KeyGenerator, RsaKeyGenerator, RsaKeyPair};
use mc_connect_core::models::packet::{ClientExportConfig, ServerConfig};
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
    config_path: Option<String>,
) -> Result<()> {
    // 最終的に使用する設定値
    let final_host: String;
    let final_port: u16;
    let final_allowed_ports_str: String;
    let key_pair_obj: RsaKeyPair;

    if let Some(ref path) = config_path {
        // --- 設定ファイルモード ---
        info!("設定ファイル {} を読み込んでいます...", path);
        let content = fs::read_to_string(path)
            .await
            .context(format!("設定ファイル {} の読み込みに失敗しました", path))?;
        let config: ServerConfig = serde_json::from_str(&content)
            .context("設定ファイルのフォーマットが正しくありません")?;

        final_host = config.bind_host;
        final_port = config.port;
        final_allowed_ports_str = config.allowed_ports; // 設定ファイル内の許可ポート設定を使用

        // 秘密鍵の復元
        let priv_key_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &config.private_key,
        )
        .context("秘密鍵の Base64 デコードに失敗しました")?;
        key_pair_obj = RsaKeyPair::from_private_der(&priv_key_bytes)
            .map_err(|e| anyhow::anyhow!("秘密鍵の読み込みに失敗しました: {}", e))?;

        info!(
            "設定ファイルから設定をロードしました: Host={}, Port={}",
            final_host, final_port
        );
    } else {
        // --- CLI モード (default.json 生成モード) ---
        final_host = host;
        final_port = port;
        final_allowed_ports_str = allowed_ports_str; // CLI 引数を使用

        // 鍵の読み込み (key_pair引数があればそれを使用、なければ default.json 用に新規生成 or default.json があればそれを読むべきだが、今回の要件では「コマンドで起動されたときは default.json として出力」なので新規生成して保存の流れ)
        // ただし、--key-pair が指定されていれば、それはそちらを優先して読み込む形にする（既存ロジック温存）
        if let Some(ref path_str) = key_pair_path {
            // 既存の --key-pair ロジック
            let path = Path::new(path_str);
            if path.exists() {
                info!("既存のキーペアを読み込んでいます: {}", path_str);
                let bytes = fs::read(path).await.context(format!(
                    "キーファイル {} の読み込みに失敗しました",
                    path_str
                ))?;
                key_pair_obj = RsaKeyPair::from_private_der(&bytes)
                    .map_err(|e| anyhow::anyhow!("キーのパースに失敗しました: {}", e))?;
            } else {
                info!("新しいキーペアを生成しています...");
                let generator = RsaKeyGenerator::default();
                let kp = generator
                    .generate()
                    .map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
                let priv_bytes = kp.private_key_bytes();
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).await?;
                    }
                }
                fs::write(path, &priv_bytes).await?;
                key_pair_obj = RsaKeyPair::from_private_der(&priv_bytes).unwrap();
            }
        } else {
            // 鍵指定なし -> 新規生成して default.json に埋め込む
            info!("ServerConfig 用のキーペアを生成中...");
            let generator = RsaKeyGenerator::default();
            let kp = generator
                .generate()
                .map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
            key_pair_obj = RsaKeyPair::from_private_der(&kp.private_key_bytes()).unwrap();
        }

        // default.json への保存
        let pub_key_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            key_pair_obj.public_key_bytes(),
        );
        let priv_key_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            key_pair_obj.private_key_bytes(),
        );

        // 公開ホストの決定
        let resolved_public_host = public_host.as_deref().unwrap_or("127.0.0.1").to_string();

        let server_config = ServerConfig {
            bind_host: final_host.clone(),
            public_host: resolved_public_host,
            port: final_port,
            public_key: pub_key_b64,
            private_key: priv_key_b64,
            allowed_ports: final_allowed_ports_str.clone(),
        };

        let json_output = serde_json::to_string_pretty(&server_config)?;
        let default_config_path = "default.json";
        fs::write(default_config_path, json_output)
            .await
            .context("default.json の保存に失敗しました")?;
        info!("現在の設定を {} に保存しました。", default_config_path);
    }

    // allowd_ports のパース
    let parsed_ports = parse_allowed_ports(&final_allowed_ports_str)?;

    let pub_key_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        key_pair_obj.public_key_bytes(),
    );

    // 設定のエクスポート (Client配布用)
    if let Some(path) = export {
        let export_data = ClientExportConfig {
            name: "Server Connection".to_string(),
            ws_url: format!(
                "ws://{}:{}/ws",
                public_host.unwrap_or_else(|| "127.0.0.1".to_string()),
                final_port
            ),
            mappings: parsed_ports.clone(),
            public_key: pub_key_b64.clone(),
            encryption_type: "RSA".to_string(),
        };
        let json = serde_json::to_string_pretty(&export_data)?;
        fs::write(&path, json)
            .await
            .context(format!("設定ファイル {} の書き込みに失敗しました", path))?;
        info!("クライアント用設定ファイルを書き出しました: {}", path);
    }

    info!("====================================================");
    info!("サーバーの公開鍵:");
    info!("{}", pub_key_b64);
    info!("====================================================");

    let rsa_pair = Arc::new(key_pair_obj);

    info!("Starting server on {}:{}", final_host, final_port);
    start_server(&final_host, final_port, parsed_ports, rsa_pair)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
