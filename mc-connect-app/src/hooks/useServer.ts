import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ServerConfig, AppSettings } from "../types";

export const useServer = () => {
    const [settings, setSettings] = useState<AppSettings>(() => {
        const saved = localStorage.getItem("mc-connect-settings");
        return saved ? JSON.parse(saved) : { serverModeEnabled: false };
    });

    const [serverConfig, setServerConfig] = useState<ServerConfig>(() => {
        const saved = localStorage.getItem("mc-connect-server-config");
        return saved ? JSON.parse(saved) : {
            isRunning: false,
            listenPort: 8080,
            allowedPorts: [{ port: 25565, protocol: "TCP" }]
        };
    });

    useEffect(() => {
        localStorage.setItem("mc-connect-settings", JSON.stringify(settings));
    }, [settings]);

    useEffect(() => {
        localStorage.setItem("mc-connect-server-config", JSON.stringify(serverConfig));
    }, [serverConfig]);

    // Check actual status from backend on mount
    useEffect(() => {
        invoke<boolean>("is_server_running").then(running => {
            setServerConfig(prev => ({ ...prev, isRunning: running }));
        });
    }, []);

    const generateKeys = async () => {
        try {
            const [priv, pub] = await invoke<[string, string]>("generate_server_keys");
            setServerConfig(prev => ({ ...prev, privateKey: priv, publicKey: pub }));
            return true;
        } catch (error) {
            console.error("Key generation failed", error);
            return false;
        }
    };

    const startServer = async () => {
        if (!serverConfig.privateKey) {
            alert("サーバーを起動する前に鍵を生成してください。");
            return;
        }
        try {
            await invoke("start_server", {
                port: serverConfig.listenPort,
                allowed_ports: serverConfig.allowedPorts.map(p => [p.port, p.protocol]),
                private_key_b64: serverConfig.privateKey
            });
            setServerConfig(prev => ({ ...prev, isRunning: true }));
        } catch (error) {
            alert(`サーバー起動失敗: ${error}`);
        }
    };

    const stopServer = async () => {
        try {
            await invoke("stop_server");
            setServerConfig(prev => ({ ...prev, isRunning: false }));
        } catch (error) {
            console.error("Stop server failed", error);
        }
    };

    const exportConfig = () => {
        if (!serverConfig.publicKey) return null;
        const config = {
            name: "Server Connection",
            ws_url: `ws://localhost:${serverConfig.listenPort}/ws`,
            mappings: serverConfig.allowedPorts,
            public_key: serverConfig.publicKey
        };
        return JSON.stringify(config, null, 2);
    };

    return {
        settings,
        setSettings,
        serverConfig,
        setServerConfig,
        generateKeys,
        startServer,
        stopServer,
        exportConfig
    };
};
