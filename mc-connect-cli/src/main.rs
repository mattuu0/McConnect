mod commands;
mod utils;

use crate::commands::client::run_client;
use crate::commands::server::run_server;
use anyhow::Result;
use clap::{Parser, Subcommand};

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

        /// サーバー設定ファイル (JSON)。指定された場合、このファイルから設定を読み込みます。
        /// 指定がない場合、CLI 引数を使用して起動し、設定を default.json に書き出します。
        #[arg(long)]
        config: Option<String>,
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
        Commands::Server {
            host,
            public_host,
            port,
            allowed_ports,
            export,
            key_pair,
            config,
        } => {
            run_server(
                host,
                public_host,
                port,
                allowed_ports,
                export,
                key_pair,
                config,
            )
            .await
        }
        Commands::Client {
            local_port,
            remote_port,
            protocol,
            ws_url,
            list_ports,
            public_key,
            config,
        } => {
            run_client(
                local_port,
                remote_port,
                protocol,
                ws_url,
                list_ports,
                public_key,
                config,
            )
            .await
        }
    }
}
