use actix::prelude::*;
use actix_web_actors::ws;
use log::{info, error, warn};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use crate::models::packet::{Message, Command, ConnectPayload, ConnectResponsePayload, Protocol};

/// WebSocket 1接続につき1つの TCP 接続を管理するセッションアクター
pub struct WsProxySession {
    /// TCP 側へデータを送信するためのチャネル
    tcp_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    /// 初期化（Connect パケット受信）済みフラグ
    initialized: bool,
}

impl WsProxySession {
    pub fn new() -> Self {
        Self {
            tcp_tx: None,
            initialized: false,
        }
    }

    /// WebSocket へ Message パケットを送る補助関数
    fn send_packet(&self, ctx: &mut ws::WebsocketContext<Self>, command: Command, payload: Vec<u8>) {
        let msg = Message::new(command, payload);
        if let Ok(bin) = msg.to_vec() {
            ctx.binary(bin);
        }
    }

    /// エラー応答を送って停止する
    fn stop_with_error(&self, ctx: &mut ws::WebsocketContext<Self>, message: String) {
        let res = ConnectResponsePayload { success: false, message: message.clone() };
        if let Ok(msg) = Message::from_payload(Command::ConnectResponse, &res) {
            if let Ok(bin) = msg.to_vec() {
                ctx.binary(bin);
            }
        }
        error!("セッション停止: {}", message);
        ctx.stop();
    }
}

impl Actor for WsProxySession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket 接続が確立されました。初期化を待機中...");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket セッションが終了しました。");
        // tcp_tx がドロップされることで、TCP 書き込みタスクも終了します。
    }
}

/// WebSocket からのメッセージ処理
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsProxySession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let bin = match msg {
            Ok(ws::Message::Binary(bin)) => bin,
            Ok(ws::Message::Ping(p)) => {
                ctx.pong(&p);
                return;
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                return;
            }
            _ => return,
        };

        // MessagePack デシリアライズ
        let packet = match Message::from_slice(&bin) {
            Ok(p) => p,
            Err(e) => {
                error!("パケット解析エラー: {}", e);
                return;
            }
        };

        match packet.command {
            Command::Connect => {
                if self.initialized {
                    warn!("既に初期化されています。無視します。");
                    return;
                }
                self.handle_connect(packet, ctx);
            }
            Command::Data => {
                if let Some(tx) = &self.tcp_tx {
                    // ペイロード（生データ）を TCP 側へ
                    let _ = tx.send(packet.payload);
                } else {
                    warn!("TCP 接続前にデータを受信しました。");
                }
            }
            Command::Disconnect => {
                info!("クライアントから切断通知を受信しました。");
                ctx.stop();
            }
            Command::Ping => {
                self.send_packet(ctx, Command::Pong, vec![]);
            }
            _ => {
                warn!("未対応のコマンド: {:?}", packet.command);
            }
        }
    }
}

impl WsProxySession {
    fn handle_connect(&mut self, packet: Message, ctx: &mut ws::WebsocketContext<Self>) {
        let payload: ConnectPayload = match packet.deserialize_payload() {
            Ok(p) => p,
            Err(e) => {
                self.stop_with_error(ctx, format!("Payload error: {}", e));
                return;
            }
        };

        if payload.protocol != Protocol::TCP {
             self.stop_with_error(ctx, "Unsupported protocol".to_string());
             return;
        }

        info!("TCP ターゲットへ接続中: 127.0.0.1:{}", payload.port);
        let target_addr = format!("127.0.0.1:{}", payload.port);
        let session_addr = ctx.address();

        // 非同期接続処理
        let fut = actix::fut::wrap_future::<_, Self>(async move {
            TcpStream::connect(&target_addr).await
        })
        .map(move |res, _act, ctx| {
            match res {
                Ok(stream) => {
                    info!("TCP ターゲットへの接続に成功しました");
                    session_addr.do_send(TcpConnected { stream });
                }
                Err(e) => {
                    error!("TCP ターゲットへの接続に失敗: {}", e);
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

/// TCP 接続成功時の内部メッセージ
#[derive(Message)]
#[rtype(result = "()")]
struct TcpConnected {
    stream: TcpStream,
}

impl Handler<TcpConnected> for WsProxySession {
    type Result = ();

    fn handle(&mut self, msg: TcpConnected, ctx: &mut Self::Context) {
        let (mut reader, mut writer) = msg.stream.into_split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        self.tcp_tx = Some(tx);

        // --- TCP Write タスク ---
        let session_addr_for_write = ctx.address();
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = writer.write_all(&data).await {
                    error!("TCP 書き込みエラー: {}", e);
                    break;
                }
            }
            let _ = session_addr_for_write.do_send(TcpStatusMsg::Disconnected);
        });

        // --- TCP Read タスク ---
        let session_addr_for_read = ctx.address();
        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let _ = session_addr_for_read.do_send(TcpStatusMsg::Data(buf[..n].to_vec()));
                    }
                    Err(e) => {
                        error!("TCP 読み取りエラー: {}", e);
                        break;
                    }
                }
            }
            let _ = session_addr_for_read.do_send(TcpStatusMsg::Disconnected);
        });

        // 成功をクライアントへ通知
        let res = ConnectResponsePayload { success: true, message: "Connected".to_string() };
        if let Ok(msg) = Message::from_payload(Command::ConnectResponse, &res) {
            if let Ok(bin) = msg.to_vec() {
                ctx.binary(bin);
            }
        }
    }
}

/// 内部通知用メッセージ
#[derive(Message)]
#[rtype(result = "()")]
enum TcpStatusMsg {
    Data(Vec<u8>),
    Disconnected,
}

impl Handler<TcpStatusMsg> for WsProxySession {
    type Result = ();
    fn handle(&mut self, msg: TcpStatusMsg, ctx: &mut Self::Context) {
        match msg {
            TcpStatusMsg::Data(data) => {
                // TCP から届いたデータを WS(Client) へ転送
                self.send_packet(ctx, Command::Data, data);
            }
            TcpStatusMsg::Disconnected => {
                ctx.stop();
            }
        }
    }
}
