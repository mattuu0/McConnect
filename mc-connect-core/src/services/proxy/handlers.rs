use actix::prelude::*;
use actix_web_actors::ws;
use log::{error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use super::session::WsProxySession;
use crate::encryption::handle_server_handshake;
use crate::models::packet::{
    Command, ConnectResponsePayload, Message, Protocol, ServerInfoResponsePayload,
};

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
        let mut packet = match Message::from_slice(&bin) {
            Ok(p) => p,
            Err(e) => {
                error!("McConnect パケットのデシリアライズに失敗: {}", e);
                return;
            }
        };

        // コンテキストを使用してペイロードを復号
        if packet.command != Command::SecureConnect && packet.command != Command::GetServerInfo {
            packet = match self.secure_context.unseal_message(packet) {
                Ok(m) => m,
                Err(e) => {
                    error!(
                        "パケットの復号に失敗しました (共通鍵の不一致や改ざんの可能性): {}. セッションを終了します。",
                        e
                    );
                    ctx.stop();
                    return;
                }
            };
        }

        // コマンドごとの処理振り分け
        match packet.command {
            Command::SecureConnect => {
                info!("SecureConnect リクエストを受信しました。ハンドシェイクを開始します。");
                if self.initialized {
                    warn!(
                        "既に初期化済みのセッションで SecureConnect を受信しました。無視します。"
                    );
                    return;
                }
                self.handle_secure_connect(packet, ctx);
            }
            Command::Connect => {
                error!(
                    "暗号化されていない接続要求 (Connect) を受信しました。本サーバーはセキュア接続のみを許可します。"
                );
                self.stop_with_error(ctx, "Secure connection is required.".to_string());
            }
            Command::Data => {
                if let Some(tx) = &self.tcp_tx {
                    if tx.send(packet.payload).is_err() {
                        error!(
                            "TCP ターゲットへのデータ転送に失敗しました。接続が切断されている可能性があります。"
                        );
                        ctx.stop();
                    }
                } else {
                    warn!(
                        "Data パケットを受信しましたが、TCP 接続が確立されていません。無視します。"
                    );
                }
            }
            Command::Disconnect => {
                info!("クライアントから明示的な切断要求を受信しました。");
                ctx.stop();
            }
            Command::GetServerInfo => {
                info!("サーバー情報の問い合わせ (GetServerInfo) に応答します。");
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
            Command::Ping => {
                self.send_packet(ctx, Command::Pong, packet.payload);
            }
            _ => {
                warn!(
                    "未実装または未知のコマンドを受信しました: {:?}",
                    packet.command
                );
            }
        }
    }
}

impl WsProxySession {
    pub(super) fn handle_secure_connect(
        &mut self,
        packet: Message,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        info!("SecureConnect リクエストを解析中...");

        // 1. ハンドシェイク処理
        let (secure_context, protocol, port) = match handle_server_handshake(
            packet,
            self.server_key.as_ref(),
        ) {
            Ok(res) => res,
            Err(e) => {
                error!(
                    "セキュアハンドシェイクに失敗: {}. クライアントの鍵が古い可能性があります。",
                    e
                );
                self.stop_with_error(ctx, format!("Handshake failed: {}", e));
                return;
            }
        };

        info!(
            "ハンドシェイクに成功しました。プロトコル: {:?}, ポート: {}",
            protocol, port
        );
        self.secure_context = secure_context;

        // 2. 許可されたポート/プロトコルかチェック
        let is_allowed = self
            .allowed_ports
            .iter()
            .any(|p| p.port == port && p.protocol == protocol);

        if !is_allowed {
            error!(
                "不許可なポートへのアクセス要求をブロックしました: {}:{:?}",
                port, protocol
            );
            self.stop_with_error(
                ctx,
                format!("Unauthorized access to port {}: {:?}", port, protocol),
            );
            return;
        }

        info!(
            "ターゲット (127.0.0.1:{}) への TCP 接続を試行します...",
            port
        );
        let target_addr = format!("127.0.0.1:{}", port);
        let session_addr = ctx.address();

        // 3. 非同期接続の実行
        let fut =
            actix::fut::wrap_future::<_, Self>(
                async move { TcpStream::connect(&target_addr).await },
            )
            .map(move |res, _act, ctx| {
                match res {
                    Ok(stream) => {
                        info!("Successfully connected to target TCP server.");
                        // 接続に成功したら、自分自身に TcpConnected メッセージを送って転送ループを開始
                        session_addr.do_send(TcpConnected { stream });
                    }
                    Err(e) => {
                        error!("Failed to connect to target: {}", e);
                        let res = ConnectResponsePayload {
                            success: false,
                            message: e.to_string(),
                        };
                        // 応答を暗号化して送信 (send_packet を使用)
                        _act.send_packet(
                            ctx,
                            Command::ConnectResponse,
                            rmp_serde::to_vec(&res).unwrap(),
                        );
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

    fn handle(&mut self, msg: TcpConnected, ctx: &mut Self::Context) {
        let (mut reader, mut writer) = msg.stream.into_split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        self.tcp_tx = Some(tx);

        // TCP への書き込みタスク
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = writer.write_all(&data).await {
                    error!("TCP Target write error: {}", e);
                    break;
                }
            }
        });

        // TCP からの読み取りタスク
        let session_addr = ctx.address();
        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        session_addr.do_send(TcpStatusMsg::Data(buf[..n].to_vec()));
                    }
                    Err(e) => {
                        error!("TCP Target read error: {}", e);
                        break;
                    }
                }
            }
            let _ = session_addr.do_send(TcpStatusMsg::Disconnected);
        });

        // 最後に、クライアントへ「準備完了 (ConnectResponse: success=true)」を送信
        let res = ConnectResponsePayload {
            success: true,
            message: "OK".to_string(),
        };
        // この時点では SecureContext が確立されているため、暗号化されて送信されます
        self.send_packet(
            ctx,
            Command::ConnectResponse,
            rmp_serde::to_vec(&res).unwrap(),
        );
        info!("Handshake completed. Secure bridge established.");
    }
}

/// [TcpStatusMsg]
/// TCP 接続側からの状態（データ着信・切断）をアクターへ伝えるための内部メッセージです。
#[derive(Message)]
#[rtype(result = "()")]
pub enum TcpStatusMsg {
    Data(Vec<u8>),
    Disconnected,
}

impl Handler<TcpStatusMsg> for WsProxySession {
    type Result = ();

    fn handle(&mut self, msg: TcpStatusMsg, ctx: &mut Self::Context) {
        match msg {
            TcpStatusMsg::Data(data) => {
                // send_packet を通じて暗号化して WS へ送信
                self.send_packet(ctx, Command::Data, data);
            }
            TcpStatusMsg::Disconnected => {
                info!("TCP Target disconnected. Closing session.");
                ctx.stop();
            }
        }
    }
}
