use serde::{Deserialize, Serialize};

/// 通信プロトコルの種類を定義します。
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Protocol {
    /// 通常の TCP 通信
    TCP,
    /// UDP 通信
    UDP,
}

/// 通信制御やデータ転送のためのコマンドを定義します。
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Command {
    /// 接続初期化要求 (Client -> Server)
    /// クライアントが WebSocket 接続後に最初に送るパケットです。
    Connect,
    /// 接続初期化応答 (Server -> Client)
    /// サーバーがターゲットへの接続成否を返します。
    ConnectResponse,
    /// サーバー情報を取得 (Client -> Server)
    GetServerInfo,
    /// サーバー情報を返却 (Server -> Client)
    ServerInfoResponse,
    /// データ転送 (双方向)
    /// 実際のバイナリデータ（マイクラのパケット等）を運びます。
    Data,
    /// 接続切断通知 (双方向)
    /// 片方が TCP 接続を閉じた際に、もう片方へ通知するために使用します。
    Disconnect,
    /// ハートビート (生きてるかの確認)
    Ping,
    /// ハートビート応答
    Pong,
}

/// 許可されたポートの情報
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllowedPort {
    pub port: u16,
    pub protocol: Protocol,
}

/// 接続初期化時に送信される詳細情報の構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectPayload {
    /// 使用するプロトコル
    pub protocol: Protocol,
    /// サーバー側から最終的に接続してほしいターゲットポート
    pub port: u16,
    /// 将来的な拡張用の圧縮設定 (例: "zlib", "none")
    pub compression: Option<String>,
}

/// 接続初期化の成否を伝える構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectResponsePayload {
    /// 接続に成功したかどうか
    pub success: bool,
    /// 失敗時のエラー理由などのメッセージ
    pub message: String,
}

/// サーバーの構成情報を伝える構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfoResponsePayload {
    /// サーバー名などの識別子
    pub server_version: String,
    /// 許可されているポートの一覧
    pub allowed_ports: Vec<AllowedPort>,
}

/// McConnect ネットワーク上を流れる基本の「コンテナ」構造体。
/// すべてのパケットはこの形式に MessagePack でラップされて通信されます。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    /// どのような命令かを示すコマンド種別
    pub command: Command,
    /// コマンドに関連するデータ。
    /// Command::Data の場合は生バイナリ、それ以外は各ペイロード構造体の MessagePack バイナリが入ります。
    pub payload: Vec<u8>,
}

impl Message {
    /// 新しいメッセージを作成します。
    pub fn new(command: Command, payload: Vec<u8>) -> Self {
        Self { command, payload }
    }

    /// 特定の構造体（ペイロード）を MessagePack でシリアライズし、
    /// それを包んだ Message コンテナを生成します。
    pub fn from_payload<T: Serialize>(command: Command, payload: &T) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let serialized = rmp_serde::to_vec(payload)?;
        Ok(Self::new(command, serialized))
    }

    /// Message コンテナ全体を MessagePack 形式のバイナリに変換します。
    /// これにより、WebSocket 経由で送信可能な状態になります。
    pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(rmp_serde::to_vec(self)?)
    }

    /// WebSocket 等から受け取ったバイナリを Message 構造体に復元します。
    pub fn from_slice(slice: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(rmp_serde::from_slice(slice)?)
    }

    /// Message の payload 部分を特定の構造体にデシリアライズします。
    pub fn deserialize_payload<'a, T: Deserialize<'a>>(&'a self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        Ok(rmp_serde::from_slice(&self.payload)?)
    }
}
