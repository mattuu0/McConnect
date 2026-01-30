import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Mapping, TunnelStatusEvent, StatsPayload } from "../types";

export const useMappings = () => {
    const [mappings, setMappings] = useState<Mapping[]>(() => {
        const saved = localStorage.getItem("mc-connect-mappings");
        if (saved) {
            const parsed = JSON.parse(saved);
            return parsed.map((m: any) => ({
                ...m,
                name: m.name || "名称未設定",
                isRunning: false,
                statusMessage: "待機中",
                loading: false,
                error: undefined,
                hasFailed: false,
                stats: undefined,
                pingInterval: m.pingInterval || 5,
                startedAt: undefined,
                speedHistory: { up: [], down: [] },
                latencyHistory: []
            }));
        }
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

    useEffect(() => {
        localStorage.setItem("mc-connect-mappings", JSON.stringify(mappings));
    }, [mappings]);

    useEffect(() => {
        const unlistenStatus = listen<TunnelStatusEvent>("tunnel-status", (event) => {
            const isError = !event.payload.running && event.payload.message.toLowerCase().includes("error");

            setMappings(prev => prev.map(m =>
                m.id === event.payload.id
                    ? {
                        ...m,
                        isRunning: event.payload.running,
                        statusMessage: event.payload.message,
                        loading: false,
                        error: isError ? "接続失敗" : m.error,
                        hasFailed: isError ? true : m.hasFailed,
                        stats: event.payload.running ? m.stats : undefined,
                        startedAt: event.payload.running ? (m.startedAt || Date.now()) : undefined,
                    }
                    : m
            ));

            if (isError) {
                setTimeout(() => {
                    setMappings(prev => prev.map(m =>
                        m.id === event.payload.id ? { ...m, hasFailed: false } : m
                    ));
                }, 3000);
            }
        });

        const unlistenStats = listen<{ id: string, stats: StatsPayload }>("tunnel-stats", (event) => {
            setMappings(prev => prev.map(m => {
                if (m.id === event.payload.id) {
                    const history = m.speedHistory || { up: [], down: [] };
                    const newUp = [...history.up, event.payload.stats.upload_speed].slice(-20);
                    const newDown = [...history.down, event.payload.stats.download_speed].slice(-20);

                    const latHistory = m.latencyHistory || [];
                    const newLat = [...latHistory, event.payload.stats.rtt_ms || 0].slice(-20);

                    const currentStats = m.stats;
                    const newStats = { ...event.payload.stats };

                    if (currentStats) {
                        newStats.upload_total = Math.max(currentStats.upload_total, newStats.upload_total);
                        newStats.download_total = Math.max(currentStats.download_total, newStats.download_total);
                    }

                    return {
                        ...m,
                        stats: newStats,
                        speedHistory: { up: newUp, down: newDown },
                        latencyHistory: newLat
                    };
                }
                return m;
            }));
        });

        return () => {
            unlistenStatus.then(f => f());
            unlistenStats.then(f => f());
        };
    }, []);

    const startMapping = async (id: string) => {
        const mapping = mappings.find(m => m.id === id);
        if (!mapping) return;

        setMappings(prev => prev.map(m => m.id === id ? { ...m, loading: true, error: undefined } : m));

        try {
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
        } catch (e) {
            setMappings(prev => prev.map(m => m.id === id ? { ...m, loading: false, error: `起動失敗`, hasFailed: true } : m));
            setTimeout(() => {
                setMappings(prev => prev.map(m => m.id === id ? { ...m, hasFailed: false } : m));
            }, 3000);
        }
    };

    const stopMapping = async (id: string) => {
        setMappings(prev => prev.map(m => m.id === id ? { ...m, loading: true } : m));
        try {
            await invoke("stop_mapping", { id });
        } catch (e) {
            setMappings(prev => prev.map(m => m.id === id ? { ...m, loading: false, error: `停止失敗` } : m));
        }
    };

    const triggerPing = async (id: string) => {
        try {
            await invoke("trigger_ping", { id });
        } catch (e) {
            console.error("Ping trigger failed", e);
        }
    };

    const addMapping = (newM: Partial<Mapping>) => {
        const id = Math.random().toString(36).substr(2, 9);
        setMappings(prev => [...prev, {
            ...newM as Mapping,
            id,
            name: newM.name || "新規トンネル",
            isRunning: false,
            statusMessage: "待機中",
            loading: false,
            hasFailed: false,
            speedHistory: { up: [], down: [] },
            latencyHistory: []
        }]);
    };

    const updateMapping = (updatedM: Mapping) => {
        setMappings(prev => prev.map(m => m.id === updatedM.id ? updatedM : m));
    };

    const deleteMappings = (ids: string[]) => {
        setMappings(prev => prev.filter(m => !ids.includes(m.id)));
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
