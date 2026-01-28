use serde::{Deserialize, Serialize};

/// 通信プロトコルの種類
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Protocol {
    TCP,
}

/// メッセージコマンドの種類
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Command {
    /// 接続初期化要求 (Client -> Server)
    Connect,
    /// 接続初期化応答 (Server -> Client)
    ConnectResponse,
    /// データ転送 (双方向)
    Data,
    /// 接続切断通知 (双方向)
    Disconnect,
    /// ハートビート
    Ping,
    /// ハートビート応答
    Pong,
}

/// 初期接続パケットのペイロード
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectPayload {
    pub protocol: Protocol,
    pub port: u16,
    pub compression: Option<String>,
}

/// 接続応答パケットのペイロード
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectResponsePayload {
    pub success: bool,
    pub message: String,
}

/// 基本となるメッセージ構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    /// コマンド種別
    pub command: Command,
    /// ペイロード（MessagePack でシリアライズされたデータ、または生バイナリ）
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(command: Command, payload: Vec<u8>) -> Self {
        Self { command, payload }
    }

    /// 特定の構造体をシリアライズしてペイロードとして持つメッセージを作成
    pub fn from_payload<T: Serialize>(command: Command, payload: &T) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let serialized = rmp_serde::to_vec(payload)?;
        Ok(Self::new(command, serialized))
    }

    /// メッセージ全体を MessagePack バイナリに変換
    pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(rmp_serde::to_vec(self)?)
    }

    /// MessagePack バイナリからメッセージを復元
    pub fn from_slice(slice: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(rmp_serde::from_slice(slice)?)
    }

    /// ペイロードを特定の構造体としてデシリアライズ
    pub fn deserialize_payload<'a, T: Deserialize<'a>>(&'a self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        Ok(rmp_serde::from_slice(&self.payload)?)
    }
}
