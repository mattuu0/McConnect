//! McConnect コアライブラリ
//! 
//! Minecraft の TCP 通信を WebSocket にラップして転送するための基幹ロジックを提供します。

pub mod controllers;
pub mod services;
pub mod models;

// 主要な機能を外部に再公開
pub use controllers::start_server;
pub use services::ws_client::WsClientService;

// ネットワーク処理の低レイヤーモジュール
pub mod tcp;
pub mod ws;
pub mod bridge;
