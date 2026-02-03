import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ServerConfig, AppSettings } from "../types";

export const useServer = () => {
    const [settings, setSettings] = useState<AppSettings>({ serverModeEnabled: false });

    const [isGeneratingKeys, setIsGeneratingKeys] = useState(false);

    const [serverConfig, setServerConfig] = useState<ServerConfig>({
        isRunning: false,
        listenPort: 8080,
        encryptionType: "RSA",
        allowedPorts: [{ port: 25565, protocol: "TCP" }]
    });


    // Check actual status from backend on mount
    useEffect(() => {
        invoke<boolean>("is_server_running").then(running => {
            setServerConfig(prev => ({ ...prev, isRunning: running }));
        });
    }, []);

    const generateKeys = async () => {
        if (serverConfig.encryptionType !== "RSA") {
            alert("現在、RSA以外の暗号化方式はバックエンドで未実装です。");
            return false;
        }

        setIsGeneratingKeys(true);
        try {
            // Simulate some delay for UI feedback if it's too fast, 
            // but generate_server_keys 2048bit is usually fast on modern PCs.
            // Still, 4096bit can take a bit.
            const [priv, pub] = await invoke<[string, string]>("generate_server_keys");
            setServerConfig(prev => ({ ...prev, privateKey: priv, publicKey: pub }));
            return true;
        } catch (error) {
            console.error("Key generation failed", error);
            alert(`鍵生成に失敗しました: ${error}`);
            return false;
        } finally {
            setIsGeneratingKeys(false);
        }
    };

    const startServer = async () => {
        if (!serverConfig.privateKey) {
            alert("サーバーを起動する前に鍵を生成してください。");
            return;
        }
        try {
            await invoke("start_server", {
                config: {
                    port: serverConfig.listenPort,
                    allowedPorts: serverConfig.allowedPorts.map(p => [p.port, p.protocol]),
                    privateKeyB64: serverConfig.privateKey,
                    encryptionType: serverConfig.encryptionType
                }
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
            public_key: serverConfig.publicKey,
            encryption_type: serverConfig.encryptionType
        };
        return JSON.stringify(config, null, 2);
    };

    return {
        settings,
        setSettings,
        serverConfig,
        setServerConfig,
        isGeneratingKeys,
        generateKeys,
        startServer,
        stopServer,
        exportConfig
    };
};
