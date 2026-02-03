use std::sync::atomic::{AtomicU64, Ordering};
use crate::models::packet::StatsPayload;

/// [TunnelStats]
/// トンネル内の通信量やレイテンシ（RTT）をスレッドセーフに記録するための構造体です。
/// 
/// 複数のスレッド（アップロード・タスク、ダウンロード・タスク、GUI更新タスク等）から
/// 同時にアクセスされるため、全てのフィールドに `AtomicU64` を使用しています。
#[derive(Debug, Default)]
pub struct TunnelStats {
    /// 通算のアップロード転送量 (バイト単位)
    /// TCPから読み取ってWebSocketへ送る際に加算されます。
    pub upload_total: AtomicU64,

    /// 通算のダウンロード転送量 (バイト単位)
    /// WebSocketから受信してTCPへ書き込む際に加算されます。
    pub download_total: AtomicU64,

    /// 最後に計測された RTT（往復遅延時間、ミリ秒）
    /// WebSocket経由の Ping/Pong 応答の間隔に基づいて更新されます。
    pub last_rtt_ms: AtomicU64,
}

impl TunnelStats {
    /// 新しい統計情報インスタンスを初期状態で作成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// [get_snapshot]
    /// 現在の統計数値のコピー（スナップショット）を取得します。
    /// 
    /// このメソッドは主にフロントエンド（UI）やログ出力のために、
    /// その瞬間の転送状態をシリアライズ可能な `StatsPayload` 形式で提供します。
    /// `Ordering::Relaxed` を使用しているのは、厳密な同期よりもパフォーマンスを優先し、
    /// かつ転送量の計測において厳密な順序が重要ではないためです。
    pub fn get_snapshot(&self) -> StatsPayload {
        StatsPayload {
            upload_total: self.upload_total.load(Ordering::Relaxed),
            download_total: self.download_total.load(Ordering::Relaxed),
            rtt_ms: Some(self.last_rtt_ms.load(Ordering::Relaxed)),
        }
    }
}
