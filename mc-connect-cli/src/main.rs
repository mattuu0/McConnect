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
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,

        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        #[arg(short, long, default_value = "25565:tcp")]
        allowed_ports: String,

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

        #[arg(short, long)]
        ws_url: String,

        #[arg(long)]
        list_ports: bool,

        /// サーバーの公開鍵（Base64形式）。セキュア接続に必須です。
        #[arg(long)]
        public_key: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Server { host, port, allowed_ports, key_pair: _ } => {
            let parsed_ports = parse_allowed_ports(&allowed_ports)?;
            
            // サーバーキーの準備（本来はファイルから読み込むが、今回はデモ用に常に新規生成）
            info!("RSA キーペアを生成中...");
            let generator = RsaKeyGenerator::default();
            let key_pair = generator.generate().map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
            
            // Base64 で公開鍵を表示（クライアント側に渡すため）
            let pub_key_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key_pair.public_key_bytes());
            info!("====================================================");
            info!("サーバーの公開鍵 (クライアントで使用してください):");
            info!("{}", pub_key_b64);
            info!("====================================================");

            // RsaKeyPair にダウンキャスト（start_server が RsaKeyPair を要求するため）
            // トレイト経由での受け渡しを後でリファクタリングするまでの暫定的な対応
            let rsa_pair = Arc::new(RsaKeyPair::from_private_der(&key_pair.private_key_bytes()).unwrap());

            info!("Starting server on {}:{}", host, port);
            start_server(&host, port, parsed_ports, rsa_pair).await.map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
        }
        Commands::Client { local_port, remote_port, protocol, ws_url, list_ports, public_key } => {
            let proto = match protocol.to_lowercase().as_str() {
                "tcp" => Protocol::TCP,
                "udp" => Protocol::UDP,
                _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", protocol)),
            };

            if list_ports {
                info!("Fetching allowed ports from {}...", ws_url);
                match WsClientService::get_server_info(&ws_url).await {
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

            // 公開鍵のバリデーション
            let pub_key_str = public_key.ok_or_else(|| anyhow::anyhow!("--public-key が指定されていません。セキュア接続には必須です。"))?;
            let pub_key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, pub_key_str)
                .context("公開鍵の Base64 デコードに失敗しました")?;
            
            let rsa_pub_key = Arc::new(RsaKeyPair::from_public_der(&pub_key_bytes)
                .map_err(|e| anyhow::anyhow!("公開鍵の読み込みに失敗しました: {}", e))?);

            info!("Starting secure client tunnel: local {} -> ws {} -> remote {} ({:?})", local_port, ws_url, remote_port, proto);
            let stats = Arc::new(mc_connect_core::services::ws_client::TunnelStats::new());
            let (_ping_tx, ping_rx) = tokio::sync::mpsc::unbounded_channel();
            
            WsClientService::start_tunnel_with_protocol(
                "127.0.0.1".into(), 
                local_port, 
                ws_url, 
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
