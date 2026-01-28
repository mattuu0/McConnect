use clap::{Parser, Subcommand};
use mc_connect_core::{start_server, WsClientService};
use mc_connect_core::models::packet::{AllowedPort, Protocol};
use log::{info, error};
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(name = "mc-connect-cli")]
#[command(author, version, about, long_about = Some("McConnect は Minecraft の TCP 通信を WebSocket に変換してトンネルするツールです。\n\
    サーバー側では許可するポートを指定し、クライアント側では WebSocket 経由でそれらのポートに接続します。"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// サーバーを起動します
    #[command(long_about = Some("プロキシサーバーを起動します。\n\n\
        使用例:\n\
        - 基本的な起動: mc-connect-cli server\n\
        - ポートと許可ポートを指定: mc-connect-cli server -p 8080 --allowed-ports \"25565:tcp,8123:tcp\"\n\
        - 特定のアドレスでバインド: mc-connect-cli server --host 127.0.0.1"))]
    Server {
        /// バインドするホスト名
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,

        /// サーバーが待受けるポート番号 (WebSocket ポート)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,

        /// 許可するターゲットポートとプロトコルのリスト (形式: "port:proto,port:proto")
        #[arg(short, long, default_value = "25565:tcp")]
        allowed_ports: String,
    },
    /// クライアントトンネルを開始します
    #[command(long_about = Some("ローカルからの接続をプロキシサーバーへ転送するトンネルを開始します。\n\n\
        使用例:\n\
        - 基本的なマイクラ接続: mc-connect-cli client --ws-url ws://example.com/ws\n\
        - ポートを指定して接続: mc-connect-cli client -l 25565 -r 25565 --ws-url ws://example.com/ws\n\
        - サーバー側で許可されているポートを確認: mc-connect-cli client --ws-url ws://example.com/ws --list-ports"))]
    Client {
        /// ローカルで待機するポート番号 (自分の PC でマイクラが接続する先)
        #[arg(short, long, default_value_t = 25565)]
        local_port: u16,

        /// サーバー側で接続するターゲットポート番号
        #[arg(short, long, default_value_t = 25565)]
        remote_port: u16,

        /// 使用するプロトコル (tcp)
        #[arg(short, long, default_value = "tcp")]
        protocol: String,

        /// プロキシサーバーの WebSocket URL (例: ws://example.com/ws)
        #[arg(short, long)]
        ws_url: String,

        /// サーバーが許可しているポート一覧を表示して終了します
        #[arg(long)]
        list_ports: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Server { host, port, allowed_ports } => {
            let parsed_ports = parse_allowed_ports(&allowed_ports)?;
            info!("Starting server on {}:{}", host, port);
            start_server(&host, port, parsed_ports).await.map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
        }
        Commands::Client { local_port, remote_port, protocol, ws_url, list_ports } => {
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

            info!("Starting client tunnel: local {} -> ws {} -> remote {} ({:?})", local_port, ws_url, remote_port, proto);
            WsClientService::start_tunnel_with_protocol(local_port, ws_url, remote_port, proto)
                .await
                .map_err(|e| anyhow::anyhow!("Client error: {}", e))?;
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
            return Err(anyhow::anyhow!("Invalid allowed-ports format: {}. Expected 'port:protocol'", part));
        }
        
        let port: u16 = subparts[0].parse().with_context(|| format!("Invalid port number: {}", subparts[0]))?;
        let protocol = match subparts[1].to_lowercase().as_str() {
            "tcp" => Protocol::TCP,
            "udp" => Protocol::UDP,
            _ => return Err(anyhow::anyhow!("Unsupported protocol in allowed-ports: {}", subparts[1])),
        };
        
        ports.push(AllowedPort { port, protocol });
    }
    Ok(ports)
}
