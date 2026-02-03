use actix::prelude::*;
use actix_web_actors::ws;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use log::{info, error, warn};

use crate::models::packet::{Message, Command, Protocol, ConnectPayload, ConnectResponsePayload, ServerInfoResponsePayload};
use super::session::WsProxySession;

/// [StreamHandler<ws::Message>]
/// WebSocket から届く生の下位レイヤーメッセージのハンドラです。
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsProxySession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let bin = match msg {
            Ok(ws::Message::Binary(bin)) => bin,
            Ok(ws::Message::Ping(p)) => {
                // WebSocket レイヤーの Ping には Pong で即死応答
                ctx.pong(&p);
                return;
            }
            Ok(ws::Message::Close(reason)) => {
                info!("Client closed WebSocket connection: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
                return;
            }
            _ => return, // テキストメッセージなどはサポート外
        };

        // アプリケーションレイヤーのパケット (MessagePack) をデシリアライズ
        let packet = match Message::from_slice(&bin) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to deserialize McConnect packet: {}", e);
                return;
            }
        };

        // コマンドごとの処理振り分け
        match packet.command {
            // トンネル確立要求
            Command::Connect => {
                if self.initialized {
                    warn!("Ignoring redundant Connect packet for an already initialized session.");
                    return;
                }
                self.handle_connect(packet, ctx);
            }
            // データ転送（トンネル内データ）
            Command::Data => {
                if let Some(tx) = &self.tcp_tx {
                    // TCP 書き込みタスクへメッセージを送る
                    if tx.send(packet.payload).is_err() {
                        error!("Failed to forward data to TCP target. Target connection might be closed.");
                        ctx.stop();
                    }
                } else {
                    warn!("Received Data packet but TCP connection is not established yet.");
                }
            }
            // 明示的な切断要求
            Command::Disconnect => {
                info!("Client context requested disconnection.");
                ctx.stop();
            }
            // サーバーステータスの問い合わせ
            Command::GetServerInfo => {
                let res = ServerInfoResponsePayload {
                    server_version: env!("CARGO_PKG_VERSION").to_string(),
                    allowed_ports: self.allowed_ports.clone(),
                };
                if let Ok(msg) = Message::from_payload(Command::ServerInfoResponse, &res) {
                    if let Ok(bin) = msg.to_vec() {
                        ctx.binary(bin);
                    }
                }
            }
            // アプリレイヤーの Ping (RTT計測用)
            Command::Ping => {
                // そのまま Pong パケットとして送り返す
                self.send_packet(ctx, Command::Pong, packet.payload);
            }
            _ => {
                warn!("Received unimplemented or unknown command: {:?}", packet.command);
            }
        }
    }
}

impl WsProxySession {
    /// [handle_connect]
    /// クライアントからの接続要求に基づき、ターゲットとなるローカル（127.0.0.1）のポートへ TCP 接続を試行します。
    pub(super) fn handle_connect(&mut self, packet: Message, ctx: &mut ws::WebsocketContext<Self>) {
        // ペイロード（接続先ポートなど）を解析
        let payload: ConnectPayload = match packet.deserialize_payload() {
            Ok(p) => p,
            Err(e) => {
                self.stop_with_error(ctx, format!("Invalid Connect payload: {}", e));
                return;
            }
        };

        // 1. セキュリティチェック: 許可されたポート/プロトコルか
        let is_allowed = self.allowed_ports.iter().any(|p| {
            p.port == payload.port && p.protocol == payload.protocol
        });

        if !is_allowed {
            self.stop_with_error(ctx, format!("Unauthorized access to port {}: {:?}", payload.port, payload.protocol));
            return;
        }

        // 2. プロトコルチェック (現状 TCP のみ)
        if payload.protocol == Protocol::UDP {
            self.stop_with_error(ctx, "UDP protocol is currently under development and not supported.".to_string());
            return;
        }

        info!("Attempting to connect to target target 127.0.0.1:{}...", payload.port);
        let target_addr = format!("127.0.0.1:{}", payload.port);
        let session_addr = ctx.address(); // 接続成功後のコールバック用

        // 3. 非同期接続の実行
        // Actix の `wait` メソッドを使い、接続が完了するまで WebSocket の次のメッセージ処理をブロック（キューイング）させます。
        // これにより、接続完了前に Data パケットが処理される順序逆転を防ぎます。
        let fut = actix::fut::wrap_future::<_, Self>(async move {
            TcpStream::connect(&target_addr).await
        })
        .map(move |res, _act, ctx| {
            match res {
                Ok(stream) => {
                    info!("Successfully connected to target TCP server.");
                    // 接続に成功したら、自分自身に TcpConnected メッセージを送って転送ループを開始
                    session_addr.do_send(TcpConnected { stream });
                }
                Err(e) => {
                    error!("Failed to connect to target 127.0.0.1:{}: {}", payload.port, e);
                    // 失敗の原因をクライアントへ通知して終了
                    let res = ConnectResponsePayload { success: false, message: e.to_string() };
                    if let Ok(msg) = Message::from_payload(Command::ConnectResponse, &res) {
                        if let Ok(bin) = msg.to_vec() {
                            ctx.binary(bin);
                        }
                    }
                    ctx.stop();
                }
            }
        });
        
        ctx.wait(fut);
        self.initialized = true;
    }
}

/// [TcpConnected]
/// ターゲットへの TCP 接続が完了したことを知らせるためのアクター内メッセージです。
#[derive(Message)]
#[rtype(result = "()")]
pub struct TcpConnected {
    pub stream: TcpStream,
}

impl Handler<TcpConnected> for WsProxySession {
    type Result = ();

    /// 接続済みの TCP ストリームを受け取り、双方向ブリッジタスクを起動します。
    fn handle(&mut self, msg: TcpConnected, ctx: &mut Self::Context) {
        // ストリームを R/W に分割
        let (mut reader, mut writer) = msg.stream.into_split();
        
        // WS(アクター) から TCP 側へデータを渡すためのチャネルを作成
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        self.tcp_tx = Some(tx);

        // --- タスク A: WebSocket -> TCP ---
        // WebSocket から届き、アクター経由で tx に流し込まれたデータを TCP サーバーへ書き込みます。
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = writer.write_all(&data).await {
                    error!("TCP Target write error: {}", e);
                    break;
                }
            }
            info!("TCP target write task finished.");
        });

        // --- タスク B: TCP -> WebSocket ---
        // TCP サーバーから返ってきたデータを読み取り、アクター経由で WebSocket へ転送します。
        let session_addr = ctx.address();
        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break, // 接続終了
                    Ok(n) => {
                        // アクターへ「データを WS へ流して」と依頼
                        session_addr.do_send(TcpStatusMsg::Data(buf[..n].to_vec()));
                    }
                    Err(e) => {
                        error!("TCP Target read error: {}", e);
                        break;
                    }
                }
            }
            // TCP が切れたらセッション全体を閉じるよう依頼
            let _ = session_addr.do_send(TcpStatusMsg::Disconnected);
        });

        // 最後に、クライアントへ「準備完了 (ConnectResponse: success=true)」を送信
        let res = ConnectResponsePayload { success: true, message: "OK".to_string() };
        if let Ok(msg) = Message::from_payload(Command::ConnectResponse, &res) {
            if let Ok(bin) = msg.to_vec() {
                ctx.binary(bin);
            }
        }
        info!("Handshake completed. Bridge established.");
    }
}

/// [TcpStatusMsg]
/// TCP 接続側からの状態（データ着信・切断）をアクターへ伝えるための内部メッセージです。
#[derive(Message)]
#[rtype(result = "()")]
pub enum TcpStatusMsg {
    /// ターゲットからのデータ受信
    Data(Vec<u8>),
    /// ターゲット側の切断
    Disconnected,
}

impl Handler<TcpStatusMsg> for WsProxySession {
    type Result = ();
    
    /// TCP 側の状態変化を WebSocket 側へ反映させます。
    fn handle(&mut self, msg: TcpStatusMsg, ctx: &mut Self::Context) {
        match msg {
            TcpStatusMsg::Data(data) => {
                // 受信した生データを Message::Data パケットに包んで送信
                self.send_packet(ctx, Command::Data, data);
            }
            TcpStatusMsg::Disconnected => {
                info!("TCP Target disconnected. Closing session.");
                ctx.stop();
            }
        }
    }
}
