import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { LogEntry } from "../types";

export const useLogs = (currentView: string) => {
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const logEndRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const unlistenLogs = listen<LogEntry>("log-event", (event) => {
            setLogs(prev => [...prev.slice(-199), event.payload]);
        });

        return () => {
            unlistenLogs.then(f => f());
        };
    }, []);

    useEffect(() => {
        if (logEndRef.current) {
            logEndRef.current.scrollIntoView({ behavior: "smooth" });
        }
    }, [logs, currentView]);

    return { logs, logEndRef };
};
