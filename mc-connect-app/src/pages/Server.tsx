import { useState } from "react";
import { motion } from "framer-motion";
import { Server, Play, Square, Key, Share2, Plus, Trash2, ShieldCheck, Globe, Settings as SettingsIcon, RefreshCw, Zap } from "lucide-react";
import { ServerConfig } from "../types";
import { PortModal } from "../components/Modals/PortModal";
import { ConfirmModal } from "../components/Modals/ConfirmModal";

interface ServerPageProps {
    config: ServerConfig;
    isGeneratingKeys: boolean;
    onConfigChange: (config: ServerConfig) => void;
    onStart: () => void;
    onStop: () => void;
    onGenerateKeys: () => void;
}

export const ServerPage = ({ config, isGeneratingKeys, onConfigChange, onStart, onStop, onGenerateKeys }: ServerPageProps) => {
    const [isPortModalOpen, setIsPortModalOpen] = useState(false);
    const [isConfirmModalOpen, setIsConfirmModalOpen] = useState(false);

    const handleAddPort = (port: number, protocol: "TCP" | "UDP") => {
        const newPorts = [...config.allowedPorts, { port, protocol }];
        onConfigChange({ ...config, allowedPorts: newPorts });
    };

    const handleRemovePort = (index: number) => {
        const newPorts = config.allowedPorts.filter((_, i) => i !== index);
        onConfigChange({ ...config, allowedPorts: newPorts });
    };

    const handleExport = async () => {
        if (!config.publicKey) {
            alert("公開鍵がありません。先に鍵を生成してください。");
            return;
        }
        const exportData = {
            name: "Server Connection",
            ws_url: `ws://YOUR_IP:${config.listenPort}/ws`,
            mappings: config.allowedPorts,
            public_key: config.publicKey,
            encryption_type: config.encryptionType
        };

        const jsonString = JSON.stringify(exportData, null, 2);

        // File System Access API を試行 (救済策・デスクトップブラウザ向け)
        if ('showSaveFilePicker' in window) {
            try {
                const handle = await (window as any).showSaveFilePicker({
                    suggestedName: 'mc-connect-config.json',
                    types: [{
                        description: 'JSON Files',
                        accept: { 'application/json': ['.json'] },
                    }],
                });
                const writable = await handle.createWritable();
                await writable.write(jsonString);
                await writable.close();
                return;
            } catch (err: any) {
                if (err.name === 'AbortError') return;
                console.error("showSaveFilePicker failed", err);
            }
        }

        // 従来のダウンロード方法 (フォールバック)
        const blob = new Blob([jsonString], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "mc-connect-config.json";
        a.click();
        URL.revokeObjectURL(url);
    };

    return (
        <motion.div
            key="server"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex-1 flex flex-col w-full"
        >
            <header className="sticky top-0 z-30 w-full bg-white/90 backdrop-blur-md border-b border-slate-200 px-6 sm:px-12 h-20 shrink-0 flex items-center shadow-sm">
                <div className="w-full flex justify-between items-center">
                    <h2 className="text-xl font-black text-slate-800">サーバー管理</h2>
                    <div className={`p-2.5 rounded-xl border-2 transition-all ${config.isRunning ? 'bg-green-50 border-green-200 text-[#16a34a] animate-pulse' : 'bg-slate-50 border-slate-100 text-slate-400'}`}>
                        <Server size={20} />
                    </div>
                </div>
            </header>

            <div className="max-w-4xl mx-auto w-full px-6 py-10 space-y-8 pb-32">
                {/* サーバー操作パネル */}
                <section className="bg-white rounded-[2.5rem] border border-slate-200 p-8 shadow-sm overflow-hidden relative">
                    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8">
                        <div className="flex items-center gap-6">
                            <div className={`p-5 rounded-3xl ${config.isRunning ? 'bg-[#16a34a] text-white shadow-lg shadow-green-100' : 'bg-slate-100 text-slate-400'}`}>
                                <Globe size={40} className={config.isRunning ? 'animate-spin-slow' : ''} />
                            </div>
                            <div>
                                <h3 className="text-2xl font-black text-slate-900 tracking-tight italic uppercase">
                                    {config.isRunning ? 'サーバー稼働中' : 'サーバー停止中'}
                                </h3>
                                <p className="text-sm text-slate-400 font-bold">
                                    {config.isRunning ? '外部からの接続を待ち受けています' : '設定を確認して起動してください'}
                                </p>
                            </div>
                        </div>

                        <button
                            onClick={config.isRunning ? onStop : onStart}
                            className={`
                                h-16 px-10 rounded-2xl font-black text-lg flex items-center justify-center gap-3 transition-all shadow-xl outline-none border-b-4
                                ${config.isRunning
                                    ? 'bg-slate-800 hover:bg-slate-900 text-white border-slate-950 active:border-b-0 active:translate-y-1'
                                    : 'bg-[#16a34a] hover:bg-[#15803d] text-white border-green-800 shadow-green-100 active:border-b-0 active:translate-y-1'}
                            `}
                        >
                            {config.isRunning ? <><Square size={20} fill="currentColor" /> 停止</> : <><Play size={20} fill="currentColor" /> 起動</>}
                        </button>
                    </div>
                </section>

                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    {/* 基本設定 */}
                    <section className="bg-white rounded-[2rem] border border-slate-200 p-8 shadow-sm space-y-6">
                        <h4 className="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1 flex items-center gap-2">
                            <SettingsIcon size={12} /> 基本構成
                        </h4>

                        <div className="space-y-4">
                            <div>
                                <label className="text-xs font-black text-slate-500 block mb-2 px-1">待ち受けポート (WebSocket)</label>
                                <input
                                    type="number"
                                    value={config.listenPort}
                                    onChange={e => onConfigChange({ ...config, listenPort: Number(e.target.value) })}
                                    disabled={config.isRunning}
                                    className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-mono font-bold focus:border-[#16a34a] outline-none disabled:opacity-50"
                                    placeholder="8080"
                                />
                            </div>

                            <div>
                                <label className="text-xs font-black text-slate-500 block mb-2 px-1">暗号化プロトコル</label>
                                <select
                                    value={config.encryptionType}
                                    onChange={e => onConfigChange({ ...config, encryptionType: e.target.value as any })}
                                    disabled={config.isRunning}
                                    className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-black outline-none focus:border-[#16a34a] disabled:opacity-50 cursor-pointer appearance-none"
                                    style={{ backgroundImage: 'url("data:image/svg+xml,%3Csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 24 24\' stroke=\'%2316a34a\'%3E%3Cpath stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'3\' d=\'M19 9l-7 7-7-7\'%3E%3C/path%3E%3C/svg%3E")', backgroundPosition: 'right 1rem center', backgroundSize: '1.2em', backgroundRepeat: 'no-repeat' }}
                                >
                                    <option value="RSA">RSA (標準的・高可用性)</option>
                                    <option value="ED25519">ED25519 (高速・モダン)</option>
                                </select>
                            </div>
                        </div>

                        <div>
                            <label className="text-xs font-black text-slate-500 block mb-2 px-1 border-b border-slate-100 pb-2">許可するポート一覧</label>
                            <div className="space-y-2 mt-4 max-h-48 overflow-y-auto pr-2">
                                {config.allowedPorts.map((p, i) => (
                                    <div key={i} className="flex items-center justify-between p-3 bg-slate-50 rounded-xl border border-slate-100 group">
                                        <div className="flex items-center gap-3">
                                            <span className="px-2 py-0.5 bg-slate-900 text-white text-[9px] font-black rounded uppercase tracking-wider">{p.protocol}</span>
                                            <span className="font-mono font-bold text-slate-700">{p.port}</span>
                                        </div>
                                        <button
                                            onClick={() => handleRemovePort(i)}
                                            disabled={config.isRunning}
                                            className="text-slate-300 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-all disabled:hidden"
                                        >
                                            <Trash2 size={16} />
                                        </button>
                                    </div>
                                ))}
                                <button
                                    onClick={() => setIsPortModalOpen(true)}
                                    disabled={config.isRunning}
                                    className="w-full py-3 border-2 border-dashed border-slate-200 rounded-xl text-slate-400 hover:border-slate-300 hover:text-slate-500 transition-all flex items-center justify-center gap-2 text-sm font-bold disabled:hidden"
                                >
                                    <Plus size={16} /> ポートを追加
                                </button>
                            </div>
                        </div>
                    </section>

                    {/* セキュリティ・エクスポート */}
                    <section className="bg-white rounded-[2rem] border border-slate-200 p-8 shadow-sm space-y-6 flex flex-col">
                        <h4 className="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1 flex items-center gap-2">
                            <ShieldCheck size={12} /> セキュリティ & エクスポート
                        </h4>

                        <div className="flex-1 space-y-4">
                            <div className="p-4 bg-amber-50 rounded-2xl border border-amber-100 flex gap-3">
                                <Key className="text-amber-500 shrink-0" size={20} />
                                <div>
                                    <p className="text-xs text-amber-900 font-bold mb-1">暗号化キー ({config.encryptionType})</p>
                                    <p className="text-[10px] text-amber-700 font-bold leading-tight">
                                        クライアントとの安全な通信のために鍵が必要です。
                                        最初に一度だけ生成してください。
                                    </p>
                                </div>
                            </div>

                            {config.publicKey ? (
                                <div className="p-4 bg-slate-50 rounded-2xl border border-slate-100 group relative">
                                    <p className="text-[9px] font-black text-slate-400 uppercase mb-2">公開鍵（配布用）</p>
                                    <div className="font-mono text-[9px] text-slate-500 break-all line-clamp-3 bg-white p-2 rounded-lg border border-slate-100">
                                        {config.publicKey}
                                    </div>
                                    <div className="absolute top-2 right-2 p-1 bg-green-100 text-[#16a34a] rounded-lg opacity-0 group-hover:opacity-100 transition-opacity">
                                        <Zap size={10} />
                                    </div>
                                </div>
                            ) : (
                                <div className="flex-1 flex flex-col items-center justify-center p-6 border-2 border-dashed border-slate-100 rounded-2xl text-slate-300">
                                    <p className="text-xs font-bold italic">No keys generated</p>
                                </div>
                            )}
                        </div>

                        <div className="grid grid-cols-2 gap-3 pt-2">
                            <button
                                onClick={() => setIsConfirmModalOpen(true)}
                                disabled={config.isRunning || isGeneratingKeys}
                                className={`
                                    py-4 rounded-2xl font-black transition-all text-sm flex items-center justify-center gap-2 active:scale-95 disabled:opacity-50
                                    ${isGeneratingKeys ? 'bg-slate-100 text-slate-400 cursor-wait' : 'bg-white border-2 border-slate-200 text-slate-600 hover:border-slate-400'}
                                `}
                            >
                                {isGeneratingKeys ? (
                                    <><RefreshCw size={16} className="animate-spin" /> 生成中...</>
                                ) : (
                                    <><Key size={16} /> 鍵を生成</>
                                )}
                            </button>
                            <button
                                onClick={handleExport}
                                className="py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all text-sm flex items-center justify-center gap-2 shadow-lg shadow-slate-200 active:scale-95 active:translate-y-1"
                            >
                                <Share2 size={16} /> 設定書き出し
                            </button>
                        </div>
                    </section>
                </div>
            </div>

            <PortModal
                isOpen={isPortModalOpen}
                onClose={() => setIsPortModalOpen(false)}
                onAdd={handleAddPort}
            />

            <ConfirmModal
                isOpen={isConfirmModalOpen}
                title="鍵ペアを生成しますか？"
                message={config.publicKey
                    ? "新しい鍵を生成すると、現在の鍵は上書きされます。\n既にクライアントへ配布済みの設定ファイルがある場合、クライアントは接続できなくなります。"
                    : "サーバー用のRSAキーペアを新規に生成します。\n生成完了後、クライアントへ配布するための設定ファイルを書き出せるようになります。"
                }
                confirmLabel="生成を開始"
                onConfirm={onGenerateKeys}
                onClose={() => setIsConfirmModalOpen(false)}
            />
        </motion.div>
    );
};
