import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Play,
  Square,
  RefreshCw,
  ChevronRight,
  Settings,
  Link2,
  Shield,
  Activity,
  Server,
  AlertCircle,
  LayoutDashboard,
  Compass,
  Info,
  Terminal,
  Trash2
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface AllowedPort {
  port: number;
  protocol: string;
}

interface ServerInfo {
  server_version: string;
  allowed_ports: AllowedPort[];
}

interface TunnelStatus {
  running: boolean;
  message: string;
}

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

type View = "dashboard" | "console" | "settings" | "about";

export default function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const [wsUrl, setWsUrl] = useState("ws://localhost:8080/ws");
  const [localPort, setLocalPort] = useState(25565);
  const [remotePort, setRemotePort] = useState(25565);
  const [allowedPorts, setAllowedPorts] = useState<AllowedPort[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string>("接続準備完了");
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    checkStatus();

    // Listen for status updates
    const unlistenStatus = listen<TunnelStatus>("tunnel-status", (event) => {
      setIsRunning(event.payload.running);
      setStatusMessage(event.payload.message);
      if (!event.payload.running && event.payload.message.startsWith("Error")) {
        setError(event.payload.message);
      }
    });

    // Listen for log events
    const unlistenLogs = listen<LogEntry>("log-event", (event) => {
      setLogs(prev => [...prev.slice(-199), event.payload]);
    });

    return () => {
      unlistenStatus.then(f => f());
      unlistenLogs.then(f => f());
    };
  }, []);

  useEffect(() => {
    if (logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, currentView]);

  const checkStatus = async () => {
    try {
      const running: boolean = await invoke("is_tunnel_running");
      setIsRunning(running);
      if (running) setStatusMessage("実行中");
    } catch (e) {
      console.error("ステータス確認に失敗しました", e);
    }
  };

  const fetchServerInfo = async () => {
    setLoading(true);
    setError(null);
    try {
      const info: ServerInfo = await invoke("get_server_info", { wsUrl });
      setAllowedPorts(info.allowed_ports);
      if (info.allowed_ports.length > 0) {
        setRemotePort(info.allowed_ports[0].port);
      }
    } catch (e) {
      setError(`サーバー情報の取得に失敗しました。URLが正しいか確認してください。`);
      if (currentView === "settings") {
        // Keep in settings if explicit action
      } else {
        setCurrentView("dashboard");
      }
    } finally {
      setLoading(false);
    }
  };

  const startTunnel = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("start_tunnel", {
        info: {
          ws_url: wsUrl,
          local_port: localPort,
          remote_port: remotePort,
        }
      });
    } catch (e) {
      setError(`トンネルの起動に失敗しました: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const stopTunnel = async () => {
    setLoading(true);
    try {
      await invoke("stop_tunnel");
    } catch (e) {
      setError(`トンネルの停止に失敗しました: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const clearLogs = () => setLogs([]);

  const SidebarItem = ({ id, icon: Icon, label, badge }: { id: View, icon: any, label: string, badge?: number }) => (
    <button
      onClick={() => setCurrentView(id)}
      className={cn(
        "w-full flex items-center justify-between px-4 py-3 rounded-lg text-sm font-bold transition-all",
        currentView === id
          ? "bg-[#E8F0FE] text-[#1967D2]"
          : "text-[#5F6368] hover:bg-[#F1F3F4]"
      )}
    >
      <div className="flex items-center gap-3">
        <Icon className="w-5 h-5" />
        {label}
      </div>
      {badge !== undefined && badge > 0 && (
        <span className="bg-[#EA4335] text-white text-[10px] px-1.5 py-0.5 rounded-full min-w-[18px]">
          {badge > 99 ? "99+" : badge}
        </span>
      )}
    </button>
  );

  return (
    <div className="flex h-screen bg-[#F8F9FA] text-[#202124] font-sans overflow-hidden">

      {/* サイドバー */}
      <aside className="w-64 bg-white border-r border-[#DADCE0] flex flex-col p-4 pt-12">
        <div className="flex items-center gap-3 px-2 mb-10">
          <div className="w-10 h-10 bg-[#4285F4] rounded-lg flex items-center justify-center shadow-md shadow-blue-200">
            <Server className="text-white w-6 h-6" />
          </div>
          <div>
            <h1 className="text-lg font-bold tracking-tight text-[#3C4043]">McConnect</h1>
            <p className="text-[10px] font-bold text-[#70757A] uppercase tracking-wider">Cloud Native Proxy</p>
          </div>
        </div>

        <nav className="flex-1 space-y-1">
          <SidebarItem id="dashboard" icon={LayoutDashboard} label="ダッシュボード" />
          <SidebarItem id="console" icon={Terminal} label="コンソール" />
          <SidebarItem id="settings" icon={Compass} label="詳細設定" />
          <SidebarItem id="about" icon={Info} label="情報" />
        </nav>

        <div className="mt-auto border-t border-[#DADCE0] pt-4 px-2">
          <div className="flex items-center gap-2 mb-2">
            <div className={cn("w-2 h-2 rounded-full", isRunning ? "bg-[#34A853] animate-pulse" : "bg-[#EA4335]")} />
            <span className="text-[11px] font-bold text-[#70757A] uppercase tracking-wider">
              {isRunning ? "実行中" : "待機中"}
            </span>
          </div>
          <p className="text-[10px] text-[#9AA0A6]">{statusMessage}</p>
        </div>
      </aside>

      {/* メインタブコンテンツ */}
      <main className="flex-1 overflow-y-auto pt-12 p-8">
        <div className="max-w-4xl mx-auto h-full flex flex-col space-y-8">

          <AnimatePresence mode="wait">
            {currentView === "dashboard" && (
              <motion.div
                key="dashboard"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="space-y-6"
              >
                <div className="flex items-center justify-between">
                  <h2 className="text-2xl font-bold text-[#3C4043]">ダッシュボード</h2>
                  <div className="flex gap-2">
                    <button
                      onClick={() => setCurrentView("console")}
                      className="p-2 hover:bg-white rounded-full transition-colors text-[#5F6368] border border-transparent hover:border-[#DADCE0]"
                      title="コンソールを見る"
                    >
                      <Terminal className="w-5 h-5" />
                    </button>
                    <button
                      onClick={() => setCurrentView("settings")}
                      className="p-2 hover:bg-white rounded-full transition-colors text-[#5F6368] border border-transparent hover:border-[#DADCE0]"
                      title="設定"
                    >
                      <Settings className="w-5 h-5" />
                    </button>
                  </div>
                </div>

                {/* ステータス概要 */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="bg-white border border-[#DADCE0] rounded-xl p-5 flex items-center gap-4 hover:shadow-sm transition-shadow">
                    <div className="p-3 bg-[#E8F0FE] rounded-lg text-[#1967D2]">
                      <Link2 className="w-5 h-5" />
                    </div>
                    <div>
                      <p className="text-[10px] font-bold text-[#70757A] tracking-wider mb-0.5">接続方式</p>
                      <p className="text-sm font-bold">WebSocket (TCP)</p>
                    </div>
                  </div>
                  <div className="bg-white border border-[#DADCE0] rounded-xl p-5 flex items-center gap-4 hover:shadow-sm transition-shadow">
                    <div className="p-3 bg-[#E6F4EA] rounded-lg text-[#137333]">
                      <Activity className="w-5 h-5" />
                    </div>
                    <div>
                      <p className="text-[10px] font-bold text-[#70757A] tracking-wider mb-0.5">通信状態</p>
                      <p className="text-sm font-bold">{isRunning ? "接続済み" : "未接続"}</p>
                    </div>
                  </div>
                </div>

                <div className="bg-white border border-[#DADCE0] rounded-2xl shadow-sm overflow-hidden">
                  <div className="p-8 space-y-8">
                    <div className="space-y-6">
                      <div className="flex items-center gap-2">
                        <div className="w-1 h-4 bg-[#4285F4] rounded-full" />
                        <h2 className="text-base font-bold text-[#3C4043]">クイック接続</h2>
                      </div>

                      <div className="grid grid-cols-2 gap-8">
                        <div className="space-y-2">
                          <label className="text-xs font-bold text-[#5F6368] ml-1">ローカル待受ポート</label>
                          <div className="text-2xl font-bold p-3 bg-[#F1F3F4] rounded-xl text-[#202124]">
                            {localPort}
                          </div>
                          <p className="text-[10px] text-[#70757A] ml-1 italic">Client Binding</p>
                        </div>
                        <div className="space-y-2">
                          <label className="text-xs font-bold text-[#5F6368] ml-1">リモート転送先</label>
                          <div className="text-2xl font-bold p-3 bg-[#F1F3F4] rounded-xl text-[#202124]">
                            {remotePort}
                          </div>
                          <p className="text-[10px] text-[#70757A] ml-1 italic">Target Endpoint</p>
                        </div>
                      </div>
                    </div>

                    <AnimatePresence>
                      {error && (
                        <motion.div
                          initial={{ opacity: 0, scale: 0.95 }}
                          animate={{ opacity: 1, scale: 1 }}
                          exit={{ opacity: 0, scale: 0.95 }}
                          className="p-4 bg-[#FEEBEE] border border-[#FAD2D8] rounded-xl text-[#C5221F] text-xs font-bold flex items-center gap-3"
                        >
                          <AlertCircle className="w-4 h-4 shrink-0" />
                          {error}
                        </motion.div>
                      )}
                    </AnimatePresence>

                    <button
                      onClick={isRunning ? stopTunnel : startTunnel}
                      disabled={loading}
                      className={cn(
                        "w-full py-5 rounded-2xl font-bold text-lg tracking-wide transition-all shadow-md active:shadow-sm active:scale-[0.99] flex items-center justify-center gap-4",
                        isRunning
                          ? "bg-white border-2 border-[#EA4335] text-[#EA4335] hover:bg-[#FEEBEE]"
                          : "bg-[#4285F4] text-white hover:bg-[#1A73E8] shadow-blue-200"
                      )}
                    >
                      {isRunning ? (
                        <><Square className="w-5 h-5 fill-current" /> トンネル通信を終了</>
                      ) : (
                        <><Play className="w-5 h-5 fill-current ml-1" /> トンネルを確立する</>
                      )}
                    </button>
                  </div>
                </div>

                {/* ミニコンソールプレビュー */}
                {logs.length > 0 && (
                  <div className="bg-[#202124] rounded-xl p-4 font-mono text-[11px] text-[#9AA0A6] shadow-inner">
                    <div className="flex justify-between items-center mb-2 border-b border-white/10 pb-1">
                      <span className="uppercase tracking-widest font-bold">Latest Log</span>
                      <button onClick={() => setCurrentView("console")} className="hover:text-white transition-colors">View All</button>
                    </div>
                    <div>
                      <span className="text-white/30 mr-2">[{logs[logs.length - 1].timestamp}]</span>
                      <span className={cn(
                        "font-bold mr-2",
                        logs[logs.length - 1].level === "ERROR" ? "text-red-400" :
                          logs[logs.length - 1].level === "SUCCESS" ? "text-green-400" : "text-blue-400"
                      )}>{logs[logs.length - 1].level}</span>
                      <span className="text-white">{logs[logs.length - 1].message}</span>
                    </div>
                  </div>
                )}
              </motion.div>
            )}

            {currentView === "console" && (
              <motion.div
                key="console"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="flex-1 flex flex-col space-y-4 h-full min-h-0"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <button onClick={() => setCurrentView("dashboard")} className="p-2 hover:bg-white rounded-full transition-colors">
                      <ChevronRight className="w-5 h-5 rotate-180" />
                    </button>
                    <h2 className="text-2xl font-bold text-[#3C4043]">コンソールログ</h2>
                  </div>
                  <button
                    onClick={clearLogs}
                    className="flex items-center gap-2 px-3 py-1.5 text-xs font-bold text-[#EA4335] hover:bg-[#FEEBEE] rounded-lg transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                    クリア
                  </button>
                </div>

                <div className="flex-1 bg-[#202124] rounded-2xl shadow-2xl p-6 font-mono text-[13px] overflow-y-auto custom-scrollbar border border-white/5">
                  <div className="space-y-1">
                    {logs.length === 0 ? (
                      <div className="text-white/20 italic p-10 text-center">ログはまだありません。トンネルを開始するとここに接続ログが表示されます。</div>
                    ) : (
                      logs.map((log, i) => (
                        <div key={i} className="flex gap-4 hover:bg-white/5 px-2 py-0.5 rounded transition-colors group">
                          <span className="text-white/20 shrink-0 select-none w-14">[{log.timestamp}]</span>
                          <span className={cn(
                            "font-bold shrink-0 w-16",
                            log.level === "ERROR" ? "text-[#F28B82]" :
                              log.level === "SUCCESS" ? "text-[#81C995]" :
                                log.level === "WARN" ? "text-[#FDD663]" : "text-[#8AB4F8]"
                          )}>{log.level}</span>
                          <span className="text-white/90 break-all">{log.message}</span>
                        </div>
                      ))
                    )}
                    <div ref={logEndRef} />
                  </div>
                </div>
              </motion.div>
            )}

            {currentView === "settings" && (
              <motion.div
                key="settings"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="space-y-6"
              >
                <div className="flex items-center gap-4 mb-2">
                  <button onClick={() => setCurrentView("dashboard")} className="p-2 hover:bg-white rounded-full transition-colors">
                    <ChevronRight className="w-5 h-5 rotate-180" />
                  </button>
                  <h2 className="text-2xl font-bold text-[#3C4043]">詳細設定</h2>
                </div>

                <div className="bg-white border border-[#DADCE0] rounded-2xl p-8 space-y-10">
                  <section className="space-y-6">
                    <div className="flex items-center gap-2">
                      <div className="w-1 h-4 bg-[#34A853] rounded-full" />
                      <h3 className="text-sm font-bold text-[#3C4043]">ネットワークゲートウェイ</h3>
                    </div>
                    <div className="space-y-3">
                      <label className="text-xs font-bold text-[#5F6368]">WebSocket エンドポイント</label>
                      <div className="flex gap-3">
                        <input
                          type="text"
                          value={wsUrl}
                          onChange={(e) => setWsUrl(e.target.value)}
                          disabled={isRunning}
                          className="flex-1 bg-[#F1F3F4] border border-transparent focus:bg-white focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 rounded-xl px-4 py-3 text-sm transition-all outline-none"
                          placeholder="ws://example.com/ws"
                        />
                        <button
                          onClick={fetchServerInfo}
                          disabled={isRunning || loading}
                          className="px-6 py-3 bg-[#F8F9FA] border border-[#DADCE0] hover:bg-white rounded-xl text-[#1A73E8] text-sm font-bold flex items-center gap-2 transition-all shadow-sm"
                        >
                          <RefreshCw className={cn("w-4 h-4", loading && "animate-spin")} />
                          情報取得
                        </button>
                      </div>
                    </div>
                  </section>

                  <section className="space-y-6">
                    <div className="flex items-center gap-2">
                      <div className="w-1 h-4 bg-[#FBBC05] rounded-full" />
                      <h3 className="text-sm font-bold text-[#3C4043]">ポートマッピング</h3>
                    </div>
                    <div className="grid grid-cols-2 gap-8">
                      <div className="space-y-3">
                        <label className="text-xs font-bold text-[#5F6368]">ローカル待受ポート</label>
                        <input
                          type="number"
                          value={localPort}
                          onChange={(e) => setLocalPort(Number(e.target.value))}
                          disabled={isRunning}
                          className="w-full bg-[#F1F3F4] border border-transparent focus:bg-white focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 rounded-xl px-4 py-3 text-sm transition-all outline-none font-bold"
                        />
                      </div>
                      <div className="space-y-3">
                        <label className="text-xs font-bold text-[#5F6368]">リモート転送先</label>
                        {allowedPorts.length > 0 ? (
                          <div className="relative">
                            <select
                              value={remotePort}
                              onChange={(e) => setRemotePort(Number(e.target.value))}
                              disabled={isRunning}
                              className="w-full bg-[#F1F3F4] border border-transparent focus:bg-white focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 rounded-xl px-4 py-3 text-sm transition-all outline-none font-bold appearance-none cursor-pointer"
                            >
                              {allowedPorts.map((p) => (
                                <option key={p.port} value={p.port}>{p.port} ({p.protocol})</option>
                              ))}
                            </select>
                            <ChevronRight className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-[#5F6368] pointer-events-none rotate-90" />
                          </div>
                        ) : (
                          <input
                            type="number"
                            value={remotePort}
                            onChange={(e) => setRemotePort(Number(e.target.value))}
                            disabled={isRunning}
                            className="w-full bg-[#F1F3F4] border border-transparent focus:bg-white focus:border-[#4285F4] focus:ring-4 focus:ring-[#4285F4]/10 rounded-xl px-4 py-3 text-sm transition-all outline-none font-bold"
                          />
                        )}
                      </div>
                    </div>
                  </section>
                </div>
              </motion.div>
            )}

            {currentView === "about" && (
              <motion.div
                key="about"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="space-y-6 text-center pt-20"
              >
                <div className="flex justify-center mb-6">
                  <div className="w-20 h-20 bg-[#4285F4] rounded-2xl flex items-center justify-center shadow-xl shadow-blue-200">
                    <Server className="text-white w-12 h-12" />
                  </div>
                </div>
                <h2 className="text-3xl font-bold text-[#3C4043]">McConnect v0.1.0</h2>
                <p className="text-[#5F6368] max-w-md mx-auto leading-relaxed">
                  Minecraft の TCP 通信を WebSocket にカプセル化し、ファイアウォールを超えて自由に接続するための次世代プロキシツール。
                </p>
                <div className="pt-10 flex justify-center gap-4">
                  <div className="px-6 py-2 bg-white border border-[#DADCE0] rounded-full text-xs font-bold text-[#70757A]">
                    Powered by Rust & Tauri
                  </div>
                  <div className="px-6 py-2 bg-white border border-[#DADCE0] rounded-full text-xs font-bold text-[#70757A]">
                    MIT License
                  </div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </main>

      {/* セキュリティバッジ */}
      <div className="fixed bottom-6 right-8 flex items-center gap-2 bg-white/80 backdrop-blur-md px-4 py-2 border border-[#DADCE0] rounded-full shadow-sm text-[10px] font-bold text-[#34A853] uppercase tracking-widest">
        <Shield className="w-3 h-3" />
        Secure Communication
      </div>
    </div>
  );
}
