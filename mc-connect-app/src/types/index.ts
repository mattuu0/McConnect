export interface Mapping {
    id: string;
    wsUrl: string;
    bindAddr: string;
    localPort: number;
    remotePort: number;
    protocol: string;
    pingInterval: number;
    isRunning: boolean;
    statusMessage: string;
    error?: string;
    loading?: boolean;
    hasFailed?: boolean;
    stats?: StatsPayload;
    speedHistory?: { up: number[], down: number[] };
    latencyHistory?: number[];
}

export interface StatsPayload {
    upload_total: number;
    download_total: number;
    upload_speed: number;
    download_speed: number;
    rtt_ms?: number;
}

export interface TunnelStatusEvent {
    id: string;
    running: boolean;
    message: string;
}

export interface LogEntry {
    timestamp: string;
    level: string;
    message: string;
}

export type View = "dashboard" | "console" | "about";
