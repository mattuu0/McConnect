use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use futures_util::{StreamExt, SinkExt};
use url::Url;
use log::{info, error};

use crate::models::packet::{Message, Command, ConnectPayload, ConnectResponsePayload, Protocol, ServerInfoResponsePayload};
use super::stats::TunnelStats;
use super::tunnel::handle_tunnel;

/// [WsClientService]
/// WebSocket クライアント側の全体の管理を行うサービス構造体です。
/// 
/// 主な役割：
/// - ローカルでの TCP ポート待機 (Listen)
/// - 新規接続に対するトンネルセッション (`handle_tunnel`) の起動
/// - ゲートウェイへの接続事前チェック
/// - ゲートウェイからのサーバー情報の取得
pub struct WsClientService;

impl WsClientService {
    /// [start_tunnel_with_protocol]
    /// ローカル待機を開始し、トンネル機能のメインループを実行します。
    /// 
    /// # 動作の流れ：
    /// 1. 接続先ゲートウェイへの疎通確認を行い、異常があれば即座に終了します。
    /// 2. 指定された `bind_addr` と `local_port` で TCP リスナーを開きます。
    /// 3. 無限ループで新しい TCP 接続を待ち受けます。
    /// 4. 接続を受けたら、その接続専用の非同期タスク (`handle_tunnel`) を起動します。
    /// 5. 外部からの Ping 要求を一括で各タスクに配信する仕組みも持ちます。
    pub async fn start_tunnel_with_protocol(
        bind_addr: String, 
        local_port: u16, 
        ws_url: String, 
        remote_target_port: u16, 
        protocol: Protocol,
        stats: Arc<TunnelStats>,
        mut ping_rx: tokio::sync::mpsc::UnboundedReceiver<()>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // 1. 事前の疎通確認
        // プログラム起動直後の設定ミス（URLの間違い、ゲートウェイのダウンなど）を早く見つけるためです。
        info!("ゲートウェイへの疎通を確認中: {}...", ws_url);
        match Self::check_connectivity(&ws_url, remote_target_port, protocol.clone()).await {
            Ok(_) => info!("ゲートウェイとのハンドシェイクに成功しました。準備完了です。"),
            Err(e) => {
                error!("ゲートウェイとのハンドシェイクに失敗しました: {}", e);
                return Err(e);
            }
        }

        // 2. ローカルポートでの待機開始
        let listener = TcpListener::bind(format!("{}:{}", bind_addr, local_port)).await?;
        info!("TCP リスナーを {}:{} で開始しました。ローカルからの接続を待っています。", bind_addr, local_port);

        // 管理用データ: 起動中の各セッションタスク
        let mut join_set = tokio::task::JoinSet::new();
        // 各タスクへ手動Pingを配信するための送信口リスト
        let mut session_ping_txs = Vec::<tokio::sync::mpsc::UnboundedSender<()>>::new();

        loop {
            tokio::select! {
                // 新しいローカル接続（Minecraftアプリなどからのアクセス）を受信
                conn = listener.accept() => {
                    match conn {
                        Ok((tcp_stream, addr)) => {
                            info!("新しい接続を検知: {} (現在のセッション数: {})", addr, session_ping_txs.len() + 1);
                            
                            let ws_url_clone = ws_url.clone();
                            let proto_clone = protocol.clone();
                            let stats_clone = Arc::clone(&stats);
                            
                            // セッション個別の Ping チャネルを作成し管理リストに追加
                            let (session_ping_tx, session_ping_rx) = tokio::sync::mpsc::unbounded_channel();
                            session_ping_txs.push(session_ping_tx);

                            // セッションごとの独立したタスクを開始
                            join_set.spawn(async move {
                                if let Err(e) = handle_tunnel(tcp_stream, ws_url_clone, remote_target_port, proto_clone, stats_clone, session_ping_rx).await {
                                    error!("セッションが異常終了しました ({}): {}", addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("TCP 接続の受け入れ中に致命的なエラーが発生しました: {}", e);
                            break;
                        }
                    }
                }

                // 外部（GUIのボタン押下など）からの Ping 要求を受信
                // 全ての稼働中セッションに対して Ping 送信を指示します。
                Some(_) = ping_rx.recv() => {
                    // 送信に失敗したもの（＝既に終了したセッション）をリストから除外しながら一斉送信
                    session_ping_txs.retain(|tx| tx.send(()).is_ok());
                }

                // 終了したタスクを裏側で回収してメモリをクリーンに保つ
                _ = join_set.join_next(), if !join_set.is_empty() => {}
            }
        }
        Ok(())
    }

    /// [check_connectivity]
    /// 指定された設定でトンネルが確立できるか一度だけテストします。
    /// 一時的に接続してプロトコルメッセージをやり取りしたあと、すぐ切断します。
    async fn check_connectivity(ws_url: &str, remote_port: u16, protocol: Protocol) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        // ターゲット情報を送信
        let connect_content = ConnectPayload { protocol, port: remote_port, compression: None };
        let packet = Message::from_payload(Command::Connect, &connect_content)?;
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        // 応答を待機
        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ConnectResponse {
                let res: ConnectResponsePayload = res_packet.deserialize_payload()?;
                if res.success { return Ok(()); }
                return Err(res.message.into());
            }
        }
        Err("Gateway から有効な応答がありませんでした".into())
    }

    /// [get_server_info]
    /// ゲートウェイのステータス情報（バージョンや許可ポート一覧）を取得します。
    pub async fn get_server_info(ws_url: &str) -> Result<ServerInfoResponsePayload, Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();
        
        // サーバー情報取得コマンドを送信
        let packet = Message::new(Command::GetServerInfo, vec![]);
        ws_write.send(WsMessage::Binary(packet.to_vec()?)).await?;

        // 応答を待機
        if let Some(msg) = ws_read.next().await {
            let bin = msg?.into_data();
            let res_packet = Message::from_slice(&bin)?;
            if res_packet.command == Command::ServerInfoResponse {
                return Ok(res_packet.deserialize_payload()?);
            }
        }
        Err("サーバー情報の取得に失敗しました".into())
    }
}
