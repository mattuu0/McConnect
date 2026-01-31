import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Mapping, TunnelStatusEvent, StatsPayload } from "../types";

/**
 * 接続設定（マッピング）の一覧管理、保存、およびバックエンドとの通信を制御するカスタムフック
 */
export const useMappings = () => {
    // マッピングデータのリストを管理。初期値はlocalStorageから読み込む。
    const [mappings, setMappings] = useState<Mapping[]>(() => {
        const savedData = localStorage.getItem("mc-connect-mappings");
        if (savedData) {
            const parsedData = JSON.parse(savedData);
            // 保存されたデータに実行時の一時ステータスを付与して初期化
            return parsedData.map((mapping: any) => ({
                ...mapping,
                name: mapping.name || "名称未設定",
                isRunning: false,
                statusMessage: "待機中",
                loading: false,
                error: undefined,
                hasFailed: false,
                stats: undefined,
                pingInterval: mapping.pingInterval || 5, // デフォルト5秒
                startedAt: undefined,
                speedHistory: { up: [], down: [] },
                latencyHistory: []
            }));
        }
        // 初期データが存在しない場合のデフォルト値
        return [{
            id: "default",
            name: "Default Tunnel",
            wsUrl: "ws://localhost:8080/ws",
            bindAddr: "127.0.0.1",
            localPort: 25565,
            remotePort: 25565,
            protocol: "TCP",
            pingInterval: 5,
            isRunning: false,
            statusMessage: "待機中",
            speedHistory: { up: [], down: [] },
            latencyHistory: []
        }];
    });

    /**
     * マッピングが変更されるたびにlocalStorageに保存するエフェクト
     */
    useEffect(() => {
        localStorage.setItem("mc-connect-mappings", JSON.stringify(mappings));
    }, [mappings]);

    /**
     * バックエンドからのステータス更新と統計データを受信するエフェクト
     */
    useEffect(() => {
        // トンネルの実行状態（開始/停止/エラー）のイベントをリッスン
        const unlistenStatusPromise = listen<TunnelStatusEvent>("tunnel-status", (event) => {
            const isErrorMessage = !event.payload.running && event.payload.message.toLowerCase().includes("error");

            setMappings(prevMappings => prevMappings.map(mapping =>
                mapping.id === event.payload.id
                    ? {
                        ...mapping,
                        isRunning: event.payload.running,
                        statusMessage: event.payload.message,
                        loading: false,
                        error: isErrorMessage ? "接続失敗" : mapping.error,
                        hasFailed: isErrorMessage ? true : mapping.hasFailed,
                        stats: event.payload.running ? mapping.stats : undefined,
                        startedAt: event.payload.running ? (mapping.startedAt || Date.now()) : undefined,
                    }
                    : mapping
            ));

            // エラー表示を3秒後に消去するタイマー
            if (isErrorMessage) {
                setTimeout(() => {
                    setMappings(prevMappings => prevMappings.map(mapping =>
                        mapping.id === event.payload.id ? { ...mapping, hasFailed: false } : mapping
                    ));
                }, 1000);
            }
        });

        // 通信統計データ（速度、遅延等）のイベントをリッスン
        const unlistenStatsPromise = listen<{ id: string, stats: StatsPayload }>("tunnel-stats", (event) => {
            setMappings(prevMappings => prevMappings.map(mapping => {
                if (mapping.id === event.payload.id) {
                    const history = mapping.speedHistory || { up: [], down: [] };
                    // 過去20件の履歴を保持
                    const newUploadHistory = [...history.up, event.payload.stats.upload_speed].slice(-20);
                    const newDownloadHistory = [...history.down, event.payload.stats.download_speed].slice(-20);

                    const latencyHistory = mapping.latencyHistory || [];
                    const newLatencyHistory = [...latencyHistory, event.payload.stats.rtt_ms || 0].slice(-20);

                    const currentStats = mapping.stats;
                    const newStats = { ...event.payload.stats };

                    // 合計通信量は累積させる（バックエンドからの値がリセットされる場合があるため）
                    if (currentStats) {
                        newStats.upload_total = Math.max(currentStats.upload_total, newStats.upload_total);
                        newStats.download_total = Math.max(currentStats.download_total, newStats.download_total);
                    }

                    return {
                        ...mapping,
                        stats: newStats,
                        speedHistory: { up: newUploadHistory, down: newDownloadHistory },
                        latencyHistory: newLatencyHistory
                    };
                }
                return mapping;
            }));
        });

        // クリーンアップ：イベントリスナーの解除
        return () => {
            unlistenStatusPromise.then(unlistenFn => unlistenFn());
            unlistenStatsPromise.then(unlistenFn => unlistenFn());
        };
    }, []);

    /**
     * トンネル接続を開始する
     * @param id 開始するマッピングのID
     */
    const startMapping = async (id: string) => {
        const mapping = mappings.find(m => m.id === id);
        if (!mapping) return;

        // ローディング状態に設定
        setMappings(prevMappings => prevMappings.map(m => m.id === id ? { ...m, loading: true, error: undefined } : m));

        try {
            // Rust側のコマンドを呼び出し
            await invoke("start_mapping", {
                info: {
                    id: mapping.id,
                    ws_url: mapping.wsUrl,
                    bind_addr: mapping.bindAddr,
                    local_port: mapping.localPort,
                    remote_port: mapping.remotePort,
                    protocol: mapping.protocol,
                    ping_interval: mapping.pingInterval
                }
            });
        } catch (error) {
            // 起動に失敗した場合の処理
            setMappings(prevMappings => prevMappings.map(m => m.id === id ? { ...m, loading: false, error: `起動失敗`, hasFailed: true } : m));
            setTimeout(() => {
                setMappings(prevMappings => prevMappings.map(m => m.id === id ? { ...m, hasFailed: false } : m));
            }, 3000);
        }
    };

    /**
     * トンネル接続を停止する
     * @param id 停止するマッピングのID
     */
    const stopMapping = async (id: string) => {
        setMappings(prevMappings => prevMappings.map(m => m.id === id ? { ...m, loading: true } : m));
        try {
            await invoke("stop_mapping", { id });
        } catch (error) {
            setMappings(prevMappings => prevMappings.map(m => m.id === id ? { ...m, loading: false, error: `停止失敗` } : m));
        }
    };

    /**
     * PING計測を手動で実行する
     * @param id 対象のマッピングID
     */
    const triggerPing = async (id: string) => {
        try {
            await invoke("trigger_ping", { id });
        } catch (error) {
            console.error("Ping trigger failed", error);
        }
    };

    /**
     * 新しいマッピングを追加する
     * @param partialMapping 追加するマッピングデータ（一部）
     */
    const addMapping = (partialMapping: Partial<Mapping>) => {
        const newId = Math.random().toString(36).substr(2, 9);
        setMappings(prevMappings => [...prevMappings, {
            ...partialMapping as Mapping,
            id: newId,
            name: partialMapping.name || "新規トンネル",
            isRunning: false,
            statusMessage: "待機中",
            loading: false,
            hasFailed: false,
            speedHistory: { up: [], down: [] },
            latencyHistory: []
        }]);
    };

    /**
     * 既存のマッピング情報を更新する
     * @param updatedMapping 更新後のマッピングデータ
     */
    const updateMapping = (updatedMapping: Mapping) => {
        setMappings(prevMappings => prevMappings.map(mapping => mapping.id === updatedMapping.id ? updatedMapping : mapping));
    };

    /**
     * 指定されたIDのマッピングを削除する
     * @param ids 削除対象のIDリスト
     */
    const deleteMappings = (ids: string[]) => {
        setMappings(prevMappings => prevMappings.filter(mapping => !ids.includes(mapping.id)));
    };

    return {
        mappings,
        startMapping,
        stopMapping,
        triggerPing,
        addMapping,
        updateMapping,
        deleteMappings
    };
};

