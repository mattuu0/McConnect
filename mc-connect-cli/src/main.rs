use clap::{Parser, Subcommand};
use mc_connect_core::{start_server, WsClientService};
use mc_connect_core::models::packet::{AllowedPort, Protocol};
use mc_connect_core::encryption::{RsaKeyGenerator, KeyGenerator, RsaKeyPair};
use log::{info, error};
use anyhow::{Result, Context};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "mc-connect-cli")]
#[command(author, version, about, long_about = Some("McConnect は Minecraft の TCP 通信を WebSocket に変換してトンネルするツールです。"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// サーバーを起動します
    Server {
        /// バインド用のアドレス (ローカル)
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,

        /// クライアントが接続するための公開ドメインまたはIP
        #[arg(long)]
        public_host: Option<String>,

        /// サーバーが待受けるポート番号
        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        #[arg(short, long, default_value = "25565:tcp")]
        allowed_ports: String,

        /// 設定を JSON ファイルとして書き出します
        #[arg(short, long)]
        export: Option<String>,

        /// サーバーの秘密鍵/公開鍵ペア。指定しない場合は新規生成します。
        #[arg(long)]
        key_pair: Option<String>,
    },
    /// クライアントトンネルを開始します
    Client {
        #[arg(short, long, default_value_t = 25565)]
        local_port: u16,

        #[arg(short, long, default_value_t = 25565)]
        remote_port: u16,

        #[arg(short, long, default_value = "tcp")]
        protocol: String,

        /// プロキシサーバーの WebSocket URL (例: ws://example.com/ws)
        #[arg(short, long)]
        ws_url: Option<String>,

        #[arg(long)]
        list_ports: bool,

        /// サーバーの公開鍵（Base64形式）。セキュア接続に必須です。
        #[arg(long)]
        public_key: Option<String>,

        /// JSON 設定ファイルから接続情報を読み込みます
        #[arg(short, long)]
        config: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Server { host, public_host, port, allowed_ports, export, key_pair: _ } => {
            let parsed_ports = parse_allowed_ports(&allowed_ports)?;
            
            info!("RSA キーペアを生成中...");
            let generator = RsaKeyGenerator::default();
            let key_pair = generator.generate().map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
            
            let pub_key_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key_pair.public_key_bytes());
            
            // 設定のエクスポート
            if let Some(path) = export {
                let export_data = mc_connect_core::models::packet::ServerExportConfig {
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
        }
        Commands::Client { local_port, remote_port, protocol, ws_url, list_ports, public_key, config } => {
            let mut final_ws_url = ws_url;
            let mut final_pub_key = public_key;

            // 設定ファイルからの読み込み
            if let Some(path) = config {
                let content = std::fs::read_to_string(&path).context(format!("設定ファイル {} の読み取りに失敗しました", path))?;
                let cfg: mc_connect_core::models::packet::ServerExportConfig = serde_json::from_str(&content)
                    .context("設定ファイルのフォーマットが正しくありません")?;
                
                if final_ws_url.is_none() {
                    final_ws_url = Some(format!("ws://{}:{}/ws", cfg.host, cfg.port));
                }
                if final_pub_key.is_none() {
                    final_pub_key = Some(cfg.public_key);
                }
            }

            let ws_url_str = final_ws_url.ok_or_else(|| anyhow::anyhow!("--ws-url または --config が必要です"))?;
            let pub_key_str = final_pub_key.ok_or_else(|| anyhow::anyhow!("--public-key または --config が必要です"))?;

            let proto = match protocol.to_lowercase().as_str() {
                "tcp" => Protocol::TCP,
                "udp" => Protocol::UDP,
                _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", protocol)),
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

            let pub_key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, pub_key_str)
                .context("公開鍵の Base64 デコードに失敗しました")?;
            
            let rsa_pub_key = Arc::new(RsaKeyPair::from_public_der(&pub_key_bytes)
                .map_err(|e| anyhow::anyhow!("公開鍵の読み込みに失敗しました: {}", e))?);

            info!("Starting secure client tunnel: local {} -> ws {} -> remote {} ({:?})", local_port, ws_url_str, remote_port, proto);
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
                rsa_pub_key
            ).await.map_err(|e| anyhow::anyhow!("Client error: {}", e))?;
        }
    }

    Ok(())
}

fn parse_allowed_ports(input: &str) -> Result<Vec<AllowedPort>> {
    let mut ports = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        
        let subparts: Vec<&str> = part.split(':').collect();
        if subparts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid format: {}. Expected 'port:protocol'", part));
        }
        
        let port: u16 = subparts[0].parse().with_context(|| format!("Invalid port: {}", subparts[0]))?;
        let protocol = match subparts[1].to_lowercase().as_str() {
            "tcp" => Protocol::TCP,
            "udp" => Protocol::UDP,
            _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", subparts[1])),
        };
        
        ports.push(AllowedPort { port, protocol });
    }
    ports.sort_by_key(|p| p.port);
    Ok(ports)
}
