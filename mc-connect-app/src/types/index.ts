export interface Mapping {
    id: string;
    wsUrl: string;
    bindAddr: string;
    localPort: number;
    remotePort: number;
    protocol: string;
    isRunning: boolean;
    statusMessage: string;
    error?: string;
    loading?: boolean;
    hasFailed?: boolean;
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
