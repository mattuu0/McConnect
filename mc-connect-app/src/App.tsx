import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ChevronRight,
  Link2,
  Server,
  AlertCircle,
  LayoutDashboard,
  Info,
  Terminal,
  Trash2,
  Plus,
  X,
  Globe,
  CheckCircle2,
  Cloud,
  CloudOff,
  RefreshCw
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface Mapping {
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

interface TunnelStatusEvent {
  id: string;
  running: boolean;
  message: string;
}

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

type View = "dashboard" | "console" | "about";

export default function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const [mappings, setMappings] = useState<Mapping[]>(() => {
    const saved = localStorage.getItem("mc-connect-mappings");
    if (saved) {
      const parsed = JSON.parse(saved);
      return parsed.map((m: any) => ({ ...m, isRunning: false, statusMessage: "待機中", loading: false, error: undefined }));
    }
    return [{
      id: "default",
      wsUrl: "ws://localhost:8080/ws",
      bindAddr: "127.0.0.1",
      localPort: 25565,
      remotePort: 25565,
      protocol: "TCP",
      isRunning: false,
      statusMessage: "待機中"
    }];
  });

  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [showAddModal, setShowAddModal] = useState(false);
  const [isDeleteMode, setIsDeleteMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);

  const [newMapping, setNewMapping] = useState<Partial<Mapping>>({
    wsUrl: "ws://localhost:8080/ws",
    bindAddr: "127.0.0.1",
    localPort: 25565,
    remotePort: 25565,
    protocol: "TCP"
  });

  const logEndRef = useRef<HTMLDivElement>(null);

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
            hasFailed: isError ? true : m.hasFailed
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
          protocol: mapping.protocol
        }
      });
    } catch (e) {
      setMappings(prev => prev.map(m => m.id === id ? { ...m, loading: false, error: `起動失敗: ${e}`, hasFailed: true } : m));

      // 3秒後に失敗状態を解除（エラーメッセージは残す）
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
      setMappings(prev => prev.map(m => m.id === id ? { ...m, loading: false, error: `停止失敗: ${e} ` } : m));
    }
  };

  const deleteSelected = () => {
    const runningSelected = mappings.filter(m => selectedIds.includes(m.id) && m.isRunning);
    if (runningSelected.length > 0) {
      alert("実行中のマッピングは削除できません。先に停止してください。");
      return;
    }
    setMappings(prev => prev.filter(m => !selectedIds.includes(m.id)));
    setSelectedIds([]);
    setIsDeleteMode(false);
  };

  const toggleSelect = (id: string) => {
    setSelectedIds(prev =>
      prev.includes(id) ? prev.filter(i => i !== id) : [...prev, id]
    );
  };

  const addNewMapping = () => {
    const id = Math.random().toString(36).substr(2, 9);
    setMappings(prev => [...prev, {
      ...newMapping as Mapping,
      id,
      isRunning: false,
      statusMessage: "待機中"
    }]);
    setShowAddModal(false);
  };

  const SidebarItem = ({ id, icon: Icon, label }: { id: View, icon: any, label: string }) => (
    <button
      onClick={() => setCurrentView(id)}
      className={cn(
        "w-full flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-all",
        currentView === id
          ? "bg-[#E8F0FE] text-[#1967D2]"
          : "text-[#5F6368] hover:bg-[#F1F3F4]"
      )}
    >
      <Icon className="w-5 h-5" />
      {label}
    </button>
  );

  return (
    <div className="flex h-screen bg-[#F8F9FA] text-[#202124] font-sans overflow-hidden">

      {/* サイドバー */}
      <aside className="w-64 bg-white border-r border-[#DADCE0] flex flex-col p-4 pt-12">
        <div className="flex items-center gap-3 px-2 mb-10">
          <div className="w-10 h-10 bg-[#4285F4] rounded-xl flex items-center justify-center shadow-lg shadow-blue-100">
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
          <SidebarItem id="about" icon={Info} label="情報" />
        </nav>

        <div className="mt-auto border-t border-[#DADCE0] pt-4 px-2">
          <div className="flex items-center gap-2 mb-2">
            <div className={cn("w-2 h-2 rounded-full", mappings.some(m => m.isRunning) ? "bg-[#34A853] animate-pulse" : "bg-[#EA4335]")} />
            <span className="text-[11px] font-bold text-[#70757A] uppercase tracking-wider">
              {mappings.some(m => m.isRunning) ? "実行中" : "待機中"}
            </span>
          </div>
          <p className="text-[10px] text-[#9AA0A6]">{mappings.filter(m => m.isRunning).length} 個のトンネルが有効</p>
        </div>
      </aside>

      {/* メインタブコンテンツ */}
      <main className="flex-1 overflow-y-auto pt-8 p-8">
        <div className="max-w-4xl mx-auto h-full flex flex-col">

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
                  <div>
                    <h2 className="text-2xl font-bold text-[#3C4043]">ダッシュボード</h2>
                    <p className="text-sm text-[#5F6368]">ポートマッピングの管理</p>
                  </div>
                  <div className="flex items-center gap-2">
                    {isDeleteMode ? (
                      <>
                        <button
                          onClick={() => { setIsDeleteMode(false); setSelectedIds([]); }}
                          className="px-4 py-2 text-[#5F6368] font-bold text-sm hover:bg-[#F1F3F4] rounded-lg transition-all"
                        >
                          キャンセル
                        </button>
                        <button
                          onClick={deleteSelected}
                          disabled={selectedIds.length === 0}
                          className="flex items-center gap-2 px-4 py-2 bg-[#EA4335] text-white rounded-lg font-bold text-sm hover:bg-[#D93025] transition-all shadow-md shadow-red-100 disabled:opacity-50 disabled:shadow-none"
                        >
                          選択した {selectedIds.length} 件を削除
                        </button>
                      </>
                    ) : (
                      <>
                        <button
                          onClick={() => setIsDeleteMode(true)}
                          className="p-2 text-[#5F6368] hover:bg-[#F1F3F4] rounded-lg transition-all"
                          title="削除モード"
                        >
                          <Trash2 className="w-5 h-5" />
                        </button>
                        <button
                          onClick={() => setShowAddModal(true)}
                          className="flex items-center gap-2 px-4 py-2 bg-[#4285F4] text-white rounded-lg font-bold text-sm hover:bg-[#1A73E8] transition-all shadow-md shadow-blue-100"
                        >
                          <Plus className="w-4 h-4" />
                          追加
                        </button>
                      </>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-1 gap-4">
                  {mappings.map((m) => (
                    <motion.div
                      key={m.id}
                      layout
                      onClick={() => isDeleteMode && toggleSelect(m.id)}
                      className={cn(
                        "group relative border-2 rounded-2xl overflow-hidden transition-all duration-300 select-none",
                        isDeleteMode && "cursor-pointer",
                        selectedIds.includes(m.id) ? "border-[#4285F4] bg-[#E8F0FE]" :
                          m.isRunning ? "border-[#AECBFA] bg-[#F8FAFF]" : "border-[#DADCE0] bg-white hover:border-[#BDC1C6]"
                      )}
                    >
                      <div className="py-3.5 px-6 flex items-center gap-5">
                        {isDeleteMode ? (
                          <div className={cn(
                            "w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all",
                            selectedIds.includes(m.id) ? "bg-[#4285F4] border-[#4285F4]" : "border-[#BDC1C6]"
                          )}>
                            {selectedIds.includes(m.id) && <CheckCircle2 className="w-3.5 h-3.5 text-white" />}
                          </div>
                        ) : (
                          <div className={cn(
                            "w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition-colors",
                            m.loading ? "bg-[#F1F3F4] text-[#4285F4]" :
                              m.isRunning ? "bg-[#4285F4] text-white shadow-md shadow-blue-50" : "bg-[#F1F3F4] text-[#5F6368]"
                          )}>
                            {m.loading ? <RefreshCw className="w-5 h-5 animate-spin" /> :
                              m.isRunning ? <Cloud className="w-5 h-5" /> : <CloudOff className="w-5 h-5 opacity-40" />}
                          </div>
                        )}

                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-3 mb-1">
                            <span className="text-sm font-bold text-[#3C4043] truncate">{m.wsUrl.replace(/^ws?s:\/\//, '').split('/')[0]}</span>
                            <span className="h-5 px-1.5 flex items-center bg-[#F1F3F4] rounded text-[9px] font-bold text-[#5F6368] uppercase tracking-wider">
                              {m.protocol}
                            </span>
                          </div>

                          <div className="flex items-center gap-6">
                            <div className="flex flex-col">
                              <span className="text-[9px] font-bold text-[#9AA0A6] uppercase tracking-tighter leading-none mb-1">Local</span>
                              <div className="flex items-center gap-1.5 text-[10px] font-medium text-[#5F6368]">
                                <Link2 className="w-3.5 h-3.5 text-[#4285F4]" />
                                <span className="font-mono bg-[#F1F3F4]/50 px-1 py-0.5 rounded">{m.bindAddr}:{m.localPort}</span>
                              </div>
                            </div>

                            <ChevronRight className="w-3 h-3 text-[#DADCE0] mt-3" />

                            <div className="flex flex-col">
                              <div className="flex items-center gap-2 mb-1">
                                <span className="text-[9px] font-bold text-[#9AA0A6] uppercase tracking-tighter leading-none">Remote</span>
                                <span className={cn(
                                  "text-[9px] font-bold px-1.5 py-0.5 rounded leading-none",
                                  m.isRunning ? "bg-[#E6F4EA] text-[#188038]" : m.loading ? "bg-[#E8F0FE] text-[#1967D2]" : "bg-[#F1F3F4] text-[#70757A]"
                                )}>
                                  {m.isRunning ? "接続済み" : m.loading ? "接続中..." : "未接続"}
                                </span>
                              </div>
                              <div className="flex items-center gap-1.5 text-[10px] font-medium text-[#5F6368]">
                                <Globe className="w-3.5 h-3.5 text-[#34A853]" />
                                <span className="font-mono bg-[#F1F3F4]/50 px-1 py-0.5 rounded">Port:{m.remotePort}</span>
                              </div>
                            </div>
                          </div>
                        </div>

                        <div className="flex items-center">
                          {!isDeleteMode && (
                            <button
                              onClick={(e) => { e.stopPropagation(); m.isRunning ? stopMapping(m.id) : startMapping(m.id); }}
                              disabled={m.loading || m.hasFailed}
                              className={cn(
                                "px-10 py-4.5 rounded-xl transition-all shadow-md active:scale-95 text-sm font-bold min-w-[130px] flex items-center justify-center gap-2",
                                m.isRunning
                                  ? "bg-white border-2 border-[#EA4335] text-[#EA4335] hover:bg-[#FEEBEE]"
                                  : m.hasFailed
                                    ? "bg-[#EA4335] text-white shadow-red-100"
                                    : m.loading
                                      ? "bg-[#E8F0FE] text-[#1967D2] cursor-not-allowed shadow-none"
                                      : "bg-[#4285F4] text-white hover:bg-[#1A73E8] shadow-blue-100"
                              )}
                            >
                              {m.loading ? (
                                <>
                                  <RefreshCw className="w-4 h-4 animate-spin text-[#1967D2]" />
                                  <span>接続中...</span>
                                </>
                              ) : m.hasFailed ? (
                                "失敗しました"
                              ) : m.isRunning ? (
                                "切断"
                              ) : (
                                "接続"
                              )}
                            </button>
                          )}
                        </div>
                      </div>

                      {/* カード下の赤枠エラー表示 */}
                      {m.error && (
                        <div className="px-5 py-2.5 bg-[#FEEBEE] text-[#C5221F] text-[11px] font-bold border-t border-[#FAD2D8] flex items-center gap-2">
                          <AlertCircle className="w-4 h-4 shrink-0" />
                          <span>接続に失敗しました。詳細はコンソールを確認してください。</span>
                        </div>
                      )}
                    </motion.div>
                  ))}
                </div>

                {mappings.length === 0 && !isDeleteMode && (
                  <div className="text-center py-20 border-2 border-dashed border-[#DADCE0] rounded-3xl bg-white/50">
                    <p className="text-[#9AA0A6] text-sm">マッピングが登録されていません。<br />「追加」から新しい接続を作成してください。</p>
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
                  <h2 className="text-2xl font-bold text-[#3C4043]">コンソールログ</h2>
                  <button
                    onClick={() => setLogs([])}
                    className="flex items-center gap-2 px-3 py-1.5 text-xs font-bold text-[#EA4335] hover:bg-[#FEEBEE] rounded-lg transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                    クリア
                  </button>
                </div>

                <div className="flex-1 bg-[#202124] rounded-2xl shadow-2xl p-6 font-mono text-[13px] overflow-y-auto custom-scrollbar border border-white/5">
                  <div className="space-y-1">
                    {logs.length === 0 ? (
                      <div className="text-white/20 italic p-10 text-center">ログはまだありません。</div>
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

      {/* モーダル: マッピング追加 */}
      <AnimatePresence>
        {showAddModal && (
          <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setShowAddModal(false)}
              className="absolute inset-0 bg-[#202124]/40 backdrop-blur-sm"
            />
            <motion.div
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative w-full max-w-lg bg-white rounded-3xl shadow-2xl overflow-hidden"
            >
              <div className="p-6 border-b border-[#DADCE0] flex items-center justify-between">
                <h3 className="text-lg font-bold text-[#3C4043]">新しいマッピングを追加</h3>
                <button onClick={() => setShowAddModal(false)} className="p-2 hover:bg-[#F1F3F4] rounded-full text-[#5F6368]">
                  <X className="w-5 h-5" />
                </button>
              </div>

              <div className="p-8 space-y-6">
                <div className="space-y-4">
                  <div className="space-y-2">
                    <label className="text-xs font-bold text-[#5F6368] ml-1">WebSocket URL</label>
                    <input
                      type="text"
                      value={newMapping.wsUrl}
                      onChange={(e) => setNewMapping({ ...newMapping, wsUrl: e.target.value })}
                      className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all"
                      placeholder="ws://example.com/ws"
                    />
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label className="text-xs font-bold text-[#5F6368] ml-1">バインドアドレス</label>
                      <input
                        type="text"
                        value={newMapping.bindAddr}
                        onChange={(e) => setNewMapping({ ...newMapping, bindAddr: e.target.value })}
                        className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all font-mono"
                        placeholder="127.0.0.1"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-xs font-bold text-[#5F6368] ml-1">プロトコル</label>
                      <select
                        value={newMapping.protocol}
                        onChange={(e) => setNewMapping({ ...newMapping, protocol: e.target.value })}
                        className="w-full bg-[#F1F3F4] rounded-xl px-4 h-[46px] text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all appearance-none cursor-pointer"
                        style={{ backgroundImage: 'url("data:image/svg+xml,%3Csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 24 24\' stroke=\'%235F6368\'%3E%3Cpath stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'2\' d=\'M19 9l-7 7-7-7\'%3E%3C/path%3E%3C/svg%3E")', backgroundPosition: 'right 1rem center', backgroundSize: '1em', backgroundRepeat: 'no-repeat' }}
                      >
                        <option value="TCP">TCP</option>
                        <option value="UDP">UDP</option>
                      </select>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label className="text-xs font-bold text-[#5F6368] ml-1">ローカルポート</label>
                      <input
                        type="number"
                        value={newMapping.localPort}
                        onChange={(e) => setNewMapping({ ...newMapping, localPort: Number(e.target.value) })}
                        className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all font-bold"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="text-xs font-bold text-[#5F6368] ml-1">リモートポート</label>
                      <input
                        type="number"
                        value={newMapping.remotePort}
                        onChange={(e) => setNewMapping({ ...newMapping, remotePort: Number(e.target.value) })}
                        className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all font-bold"
                      />
                    </div>
                  </div>
                </div>

                <div className="pt-4 flex gap-3">
                  <button
                    onClick={() => setShowAddModal(false)}
                    className="flex-1 py-3 border border-[#DADCE0] text-[#5F6368] rounded-xl font-bold text-sm hover:bg-[#F8F9FA] transition-all"
                  >
                    キャンセル
                  </button>
                  <button
                    onClick={addNewMapping}
                    className="flex-2 py-3 bg-[#4285F4] text-white rounded-xl font-bold text-sm hover:bg-[#1A73E8] transition-all shadow-md shadow-blue-100"
                  >
                    保存して追加
                  </button>
                </div>
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>
    </div>
  );
}
