use actix::prelude::*;
use actix_web_actors::ws;
use log::{info, error, warn};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use crate::models::packet::{Message, Command, ConnectPayload, ConnectResponsePayload, Protocol};

/// WebSocket 1接続につき1つの TCP 接続を管理するセッションアクター。
/// クライアント(WebSocket)とターゲットサーバー(TCP)の間のブリッジを行います。
pub struct WsProxySession {
    /// TCP 側へデータを送信するためのチャネル。
    /// この Sender を通じて、WS から来たデータを TCP 書き込みタスクへ渡します。
    tcp_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    /// 初期化（Connect パケットによるターゲット指定）が完了しているかどうかのフラグ。
    initialized: bool,
}

impl WsProxySession {
    pub fn new() -> Self {
        Self {
            tcp_tx: None,
            initialized: false,
        }
    }

    /// WebSocket へ Message パケットをシリアライズして送信するヘルパー関数。
    fn send_packet(&self, ctx: &mut ws::WebsocketContext<Self>, command: Command, payload: Vec<u8>) {
        let msg = Message::new(command, payload);
        if let Ok(bin) = msg.to_vec() {
            ctx.binary(bin);
        }
    }

    /// エラーが発生した際に、クライアントに応答を返してセッションを終了します。
    fn stop_with_error(&self, ctx: &mut ws::WebsocketContext<Self>, message: String) {
        let res = ConnectResponsePayload { success: false, message: message.clone() };
        if let Ok(msg) = Message::from_payload(Command::ConnectResponse, &res) {
            if let Ok(bin) = msg.to_vec() {
                ctx.binary(bin);
            }
        }
        error!("セッションを異常終了します: {}", message);
        ctx.stop();
    }
}

impl Actor for WsProxySession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket セッションが開始されました。クライアントからの初期化(Connect)を待機します。");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket セッションが終了しました。関連するリソースを解放します。");
        // tcp_tx がドロップされることで、TCP 書き込みタスクの受信側(rx)が閉じられ、タスクが自然に終了します。
    }
}

/// WebSocket から流れてくるバイナリデータ（MessagePack ラップ済みパケット）の処理。
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsProxySession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let bin = match msg {
            Ok(ws::Message::Binary(bin)) => bin,
            Ok(ws::Message::Ping(p)) => {
                ctx.pong(&p);
                return;
            }
            Ok(ws::Message::Close(reason)) => {
                info!("クライアントが WebSocket を閉じました: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
                return;
            }
            _ => return,
        };

        // 受け取ったバイナリを MessagePack 形式としてデシリアライズ
        let packet = match Message::from_slice(&bin) {
            Ok(p) => p,
            Err(e) => {
                error!("受信パケットの解析に失敗しました: {}", e);
                return;
            }
        };

        match packet.command {
            // 初期化要求：ターゲットポートへの TCP 接続を開始する
            Command::Connect => {
                if self.initialized {
                    warn!("二重の初期化要求を受信しました。無視します。");
                    return;
                }
                self.handle_connect(packet, ctx);
            }
            // データ転送：WebSocket から届いたバイナリをそのまま TCP 側へ流す
            Command::Data => {
                if let Some(tx) = &self.tcp_tx {
                    // packet.payload には生データが入っている
                    let _ = tx.send(packet.payload);
                } else {
                    warn!("TCP 接続が確立される前にデータパケットを受信しました。破棄します。");
                }
            }
            // クライアント側で TCP が切断された等の通知
            Command::Disconnect => {
                info!("クライアントから明示的な切断要求を受信しました。");
                ctx.stop();
            }
            // 生存確認
            Command::Ping => {
                self.send_packet(ctx, Command::Pong, vec![]);
            }
            _ => {
                warn!("未定義のコマンドを受信しました: {:?}", packet.command);
            }
        }
    }
}

impl WsProxySession {
    /// ターゲットサーバーへの TCP 接続処理の開始
    fn handle_connect(&mut self, packet: Message, ctx: &mut ws::WebsocketContext<Self>) {
        // ペイロード（ポート番号などの接続情報）をデシリアライズ
        let payload: ConnectPayload = match packet.deserialize_payload() {
            Ok(p) => p,
            Err(e) => {
                self.stop_with_error(ctx, format!("ConnectPayload の解析失敗: {}", e));
                return;
            }
        };

        // TCP 以外は未実装
        if payload.protocol != Protocol::TCP {
             self.stop_with_error(ctx, "サポートされていないプロトコルです".to_string());
             return;
        }

        info!("ターゲットサーバー 127.0.0.1:{} へ TCP 接続を開始します...", payload.port);
        let target_addr = format!("127.0.0.1:{}", payload.port);
        let session_addr = ctx.address();

        // 非同期に TCP 接続を試行。Actix の Future ラッパーを使用
        let fut = actix::fut::wrap_future::<_, Self>(async move {
            TcpStream::connect(&target_addr).await
        })
        .map(move |res, _act, ctx| {
            match res {
                Ok(stream) => {
                    info!("ターゲットへの TCP 接続に成功しました。");
                    // 完了通知を自分自身(アクター)へ送る
                    session_addr.do_send(TcpConnected { stream });
                }
                Err(e) => {
                    error!("ターゲットへの TCP 接続に失敗しました: {}", e);
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
        
        // 接続が完了するまで WebSocket の次のメッセージ処理を待機(ウェイト)させる
        ctx.wait(fut);
        self.initialized = true;
    }
}

/// TCP 接続が成功したことを通知する内部メッセージ
#[derive(Message)]
#[rtype(result = "()")]
struct TcpConnected {
    stream: TcpStream,
}

impl Handler<TcpConnected> for WsProxySession {
    type Result = ();

    fn handle(&mut self, msg: TcpConnected, ctx: &mut Self::Context) {
        // TCP ストリームを読み取り用と書き込み用に分割
        let (mut reader, mut writer) = msg.stream.into_split();
        
        // WS(Actor) から TCP へデータを渡すためのチャネルを作成
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        self.tcp_tx = Some(tx);

        // --- TCP Write タスク (WebSocket から来たデータを TCP サーバーへ書き込む) ---
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = writer.write_all(&data).await {
                    error!("TCP ターゲットへの書き込みに失敗しました: {}", e);
                    break;
                }
            }
            info!("TCP 書き込みタスクが終了しました。");
        });

        // --- TCP Read タスク (TCP サーバーから来たデータを WebSocket クライアントへ転送) ---
        let session_addr = ctx.address();
        tokio::spawn(async move {
            let mut buf = [0u8; 8192]; // バッファサイズ 8KB
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => {
                        info!("TCP ターゲット側から接続が閉じられました。");
                        break;
                    }
                    Ok(n) => {
                        // 読み取ったデータをアクターへ送って WS 送信を依頼
                        let _ = session_addr.do_send(TcpStatusMsg::Data(buf[..n].to_vec()));
                    }
                    Err(e) => {
                        error!("TCP ターゲットからの読み取りエラー: {}", e);
                        break;
                    }
                }
            }
            // 接続終了をアクターに通知
            let _ = session_addr.do_send(TcpStatusMsg::Disconnected);
        });

        // 初期化成功をクライアントへパケットで通知
        let res = ConnectResponsePayload { success: true, message: "OK".to_string() };
        if let Ok(msg) = Message::from_payload(Command::ConnectResponse, &res) {
            if let Ok(bin) = msg.to_vec() {
                ctx.binary(bin);
            }
        }
        info!("ハンドシェイク完了。双方向ブリッジを開始します。");
    }
}

/// TCP 側の状態（データ着信、切断）をアクターへ伝えるためのメッセージ
#[derive(Message)]
#[rtype(result = "()")]
enum TcpStatusMsg {
    /// データを受信した
    Data(Vec<u8>),
    /// 接続が終了した
    Disconnected,
}

impl Handler<TcpStatusMsg> for WsProxySession {
    type Result = ();
    fn handle(&mut self, msg: TcpStatusMsg, ctx: &mut Self::Context) {
        match msg {
            TcpStatusMsg::Data(data) => {
                // TCP から届いたデータを MessagePack パケット(Command::Data)にして WS へ送信
                self.send_packet(ctx, Command::Data, data);
            }
            TcpStatusMsg::Disconnected => {
                // TCP 側が切れたらセッションを終了
                ctx.stop();
            }
        }
    }
}
