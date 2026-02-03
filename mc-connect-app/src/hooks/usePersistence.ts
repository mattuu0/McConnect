import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Mapping, ServerConfig, AppSettings } from "../types";

export const usePersistence = (
    mappings: Mapping[],
    serverConfig: ServerConfig,
    appSettings: AppSettings,
    initialized: boolean
) => {
    useEffect(() => {
        if (!initialized) return;

        const save = async () => {
            const config = {
                mappings: mappings.map(m => ({
                    id: m.id,
                    name: m.name,
                    wsUrl: m.wsUrl,
                    bindAddr: m.bindAddr,
                    localPort: m.localPort,
                    remotePort: m.remotePort,
                    protocol: m.protocol,
                    publicKey: m.publicKey,
                    pingInterval: m.pingInterval
                })),
                serverConfig: {
                    listenPort: serverConfig.listenPort,
                    publicHost: serverConfig.publicHost,
                    publicPort: serverConfig.publicPort,
                    privateKey: serverConfig.privateKey,
                    publicKey: serverConfig.publicKey,
                    encryptionType: serverConfig.encryptionType,
                    allowedPorts: serverConfig.allowedPorts.map(p => [p.port, p.protocol])
                },
                appSettings: {
                    serverModeEnabled: appSettings.serverModeEnabled
                }
            };

            try {
                await invoke("save_config", { config });
            } catch (error) {
                console.error("Failed to save config:", error);
            }
        };

        const timer = setTimeout(save, 500); // Debounce
        return () => clearTimeout(timer);
    }, [mappings, serverConfig, appSettings, initialized]);
};
