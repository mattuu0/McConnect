/**
 * トンネル接続（マッピング）の設定および現在の状態を表すインターフェース
 */
export interface Mapping {
    /** 一意識別ID */
    id: string;
    /** 表示名 */
    name: string;
    /** 接続先WebSocketプロキシのURL */
    wsUrl: string;
    /** ローカルで待ち受ける（バインドする）アドレス */
    bindAddr: string;
    /** ローカルで待ち受けるポート番号 */
    localPort: number;
    /** 接続先（外部）のポート番号 */
    remotePort: number;
    /** 使用プロトコル（"TCP" または "UDP"） */
    protocol: "TCP" | "UDP";
    /** プロキシサーバーの公開鍵（暗号化用） */
    publicKey?: string;
    /** PING送信の間隔（秒） */
    pingInterval: number;
    /** 現在トンネルが実行中かどうか */
    isRunning: boolean;
    /** 詳細なステータスメッセージ */
    statusMessage: string;
    /** エラーが発生している場合のメッセージ */
    error?: string;
    /** 処理中（開始/停止中）フラグ */
    loading?: boolean;
    /** 起動失敗フラグ（アニメーション等に使用） */
    hasFailed?: boolean;
    /** 通信統計データ */
    stats?: StatsPayload;
    /** 通信速度の履歴（20件分） */
    speedHistory?: { up: number[], down: number[] };
    /** 遅延（PING）の履歴（20件分） */
    latencyHistory?: number[];
    /** トンネル開始時刻（タイムスタンプ） */
    startedAt?: number;
}

/**
 * 転送速度や統計情報を表すペイロード
 */
export interface StatsPayload {
    /** 送信された合計バイト数 */
    upload_total: number;
    /** 受信された合計バイト数 */
    download_total: number;
    /** 現在の送信速度（bytes/s） */
    upload_speed: number;
    /** 現在の受信速度（bytes/s） */
    download_speed: number;
    /** ラウンドトリップタイム（ミリ秒） */
    rtt_ms?: number;
}

/**
 * バックエンドからのトンネル状態変更通知イベント
 */
export interface TunnelStatusEvent {
    /** 対象マッピングID */
    id: string;
    /** 実行中かどうか */
    running: boolean;
    /** 状態メッセージ */
    message: string;
}

/**
 * システムログの1件分のエントリー
 */
export interface LogEntry {
    /** ログ発生時刻（ISO形式文字列） */
    timestamp: string;
    /** ログレベル（"INFO", "ERROR", "WARN"等） */
    level: string;
    /** ログ本文 */
    message: string;
}

/**
 * アプリケーションの表示画面（ビュー）の識別子
 */
export type View = "dashboard" | "console" | "about";

