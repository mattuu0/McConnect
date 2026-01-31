import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { LogEntry } from "../types";

/**
 * システムログをリアルタイムで取得・管理するためのカスタムフック
 * @param currentView 現在の表示ビュー（ログ画面が表示された時の自動スクロール制御に使用）
 */
export const useLogs = (currentView: string) => {
    // ログエントリのリストを保持するステート
    const [logs, setLogs] = useState<LogEntry[]>([]);
    // ログ表示エリアの最下部にスクロールするための参照
    const logEndRef = useRef<HTMLDivElement>(null);

    /**
     * Tauriのイベントシステムを介してバックエンドからのログ出力を監視するエフェクト
     */
    useEffect(() => {
        // "log-event"という名前のイベントをバックエンドから待ち受ける
        const unlistenPromise = listen<LogEntry>("log-event", (event) => {
            setLogs(prevLogs => {
                // 最大200件のログを保持するように制限
                const newLogs = [...prevLogs, event.payload];
                return newLogs.slice(-200);
            });
        });

        // クリーンアップ関数：コンポーネントのアンマウント時に監視を停止
        return () => {
            unlistenPromise.then(unlistenFn => unlistenFn());
        };
    }, []);

    /**
     * 新しいログが追加された時、またはログ画面に切り替わった時に最下部までスクロールするエフェクト
     */
    useEffect(() => {
        if (logEndRef.current) {
            logEndRef.current.scrollIntoView({ behavior: "smooth" });
        }
    }, [logs, currentView]);

    return { logs, logEndRef };
};

