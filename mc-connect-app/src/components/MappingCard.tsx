import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Globe, Play, Square, Trash2, ArrowUpCircle, ArrowDownCircle, Activity, RefreshCw, Clock } from "lucide-react";
import { Mapping } from "../types";

interface MappingCardProps {
    mapping: Mapping;
    isDeleteMode: boolean;
    isSelected: boolean;
    onSelect: (id: string) => void;
    onTriggerPing: (id: string) => void;
    onEdit: (mapping: Mapping) => void;
    onToggleConnect: (e: React.MouseEvent, mapping: Mapping) => void;
}

export const MappingCard = ({
    mapping: m,
    isDeleteMode,
    isSelected,
    onSelect,
    onEdit,
    onToggleConnect
}: MappingCardProps) => {
    const [uptime, setUptime] = useState<string>("00:00:00");

    useEffect(() => {
        if (!m.isRunning || !m.startedAt) {
            setUptime("00:00:00");
            return;
        }

        const update = () => {
            const now = Date.now();
            const start = m.startedAt || now;
            const diff = Math.floor(Math.max(0, now - start) / 1000);
            const h = Math.floor(diff / 3600).toString().padStart(2, '0');
            const min = Math.floor((diff % 3600) / 60).toString().padStart(2, '0');
            const s = (diff % 60).toString().padStart(2, '0');
            setUptime(`${h}:${min}:${s}`);
        };

        update();
        const interval = setInterval(update, 1000);
        return () => clearInterval(interval);
    }, [m.isRunning, m.startedAt]);

    const formatBytes = (bytes: number) => {
        if (!bytes || bytes === 0) return "0.0 MB";
        const mb = bytes / (1024 * 1024);
        return mb.toFixed(1) + " MB";
    };

    return (
        <motion.div
            layout
            onClick={() => {
                if (isDeleteMode) onSelect(m.id);
                else onEdit(m);
            }}
            className={`
                bg-white rounded-3xl border transition-all overflow-hidden shadow-[0_4px_20px_rgba(0,0,0,0.03)] select-none cursor-pointer
                ${m.isRunning ? 'border-green-400 ring-4 ring-green-50' : 'border-slate-200 hover:border-slate-300'}
                ${isDeleteMode ? 'border-red-400 animate-pulse ring-4 ring-red-50' : ''}
                ${isSelected ? 'bg-green-50/50' : ''}
            `}
        >
            <div className="p-5 md:p-7 space-y-5">
                {/* Upper Section */}
                <div className="flex flex-wrap items-center justify-between gap-4">
                    <div className="flex items-center gap-5 flex-1 min-w-[240px]">
                        <div className={`p-4 rounded-2xl shadow-inner shrink-0 ${m.isRunning ? 'bg-[#16a34a] text-white' : 'bg-slate-100 text-slate-400'}`}>
                            {m.loading ? <RefreshCw size={28} className="animate-spin" /> : <Globe size={28} />}
                        </div>
                        <div className="truncate">
                            <div className="flex items-center gap-2 mb-1.5">
                                <h3 className="font-black text-slate-900 text-lg md:text-xl truncate tracking-tight uppercase leading-none">{m.name}</h3>
                                {m.isRunning && (
                                    <span className="flex items-center gap-1 px-2 py-0.5 rounded-full bg-green-100 text-[#16a34a] text-[10px] font-black border border-green-200 shrink-0">
                                        <Clock size={10} /> {uptime}
                                    </span>
                                )}
                            </div>
                            <div className="flex items-center flex-wrap gap-2 text-xs font-bold leading-none">
                                <span className="px-2 py-0.5 rounded-md bg-slate-900 text-white tracking-widest text-[9px] uppercase">{m.protocol}</span>
                                <span className="text-slate-400 font-mono">Port: <span className="text-slate-700">{m.remotePort}</span></span>
                                <span className="text-slate-300 hidden md:inline">|</span>
                                <span className="text-slate-400 font-mono hidden md:inline">{m.bindAddr}:{m.localPort}</span>
                            </div>
                        </div>
                    </div>

                    <div className="w-full md:w-auto">
                        {isDeleteMode ? (
                            <button
                                onClick={(e) => { e.stopPropagation(); onSelect(m.id); }}
                                className="w-full md:w-44 h-14 bg-red-500 hover:bg-red-600 text-white rounded-2xl font-black shadow-lg shadow-red-100 border-b-4 border-red-800 transition-all active:border-b-0 active:translate-y-1"
                            >
                                削除する
                            </button>
                        ) : (
                            <button
                                onClick={(e) => onToggleConnect(e, m)}
                                disabled={m.loading || (m.hasFailed && !m.isRunning)}
                                className={`
                                    w-full md:w-44 h-14 rounded-2xl font-black text-sm flex items-center justify-center gap-2 transition-all shadow-lg outline-none border-b-4 h-[56px]
                                    ${m.isRunning
                                        ? 'bg-slate-800 hover:bg-slate-900 text-white border-slate-950 shadow-slate-200 active:border-b-0 active:translate-y-1'
                                        : m.loading
                                            ? 'bg-slate-100 text-slate-300 border-slate-200 cursor-wait shadow-none translate-y-1 border-b-0'
                                            : m.hasFailed
                                                ? 'bg-red-500 text-white border-red-800 active:border-b-0 active:translate-y-1'
                                                : 'bg-[#16a34a] hover:bg-[#15803d] text-white border-green-800 shadow-green-100 active:border-b-0 active:translate-y-1'}
                                `}
                            >
                                {m.loading ? (
                                    <span className="animate-pulse">処理中...</span>
                                ) : m.isRunning ? (
                                    <><Square size={16} fill="currentColor" className="mr-1" /> 切断する</>
                                ) : m.hasFailed ? (
                                    "失敗"
                                ) : (
                                    <><Play size={16} fill="currentColor" className="mr-1" /> 接続開始</>
                                )}
                            </button>
                        )}
                    </div>
                </div>

                {/* Statistics Panel */}
                {m.isRunning && m.stats && (
                    <div className="grid grid-cols-2 md:grid-cols-3 gap-4 bg-slate-50/80 p-5 rounded-2xl border border-slate-100 animate-in slide-in-from-top-2 duration-300">
                        <div className="flex flex-col">
                            <div className="flex items-center gap-2 mb-1.5">
                                <ArrowUpCircle size={14} className="text-green-500" />
                                <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.15em]">送信合計</span>
                            </div>
                            <span className="text-base font-black font-mono text-slate-800 leading-none">{formatBytes(m.stats.upload_total)}</span>
                        </div>
                        <div className="flex flex-col">
                            <div className="flex items-center gap-2 mb-1.5">
                                <ArrowDownCircle size={14} className="text-blue-500" />
                                <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.15em]">受信合計</span>
                            </div>
                            <span className="text-base font-black font-mono text-slate-800 leading-none">{formatBytes(m.stats.download_total)}</span>
                        </div>
                        <div className="flex flex-col col-span-2 md:col-span-1 border-t md:border-t-0 md:border-l border-slate-200 pt-3 md:pt-0 md:pl-6 flex justify-center">
                            <div className="flex items-center gap-2 mb-1.5">
                                <Activity size={14} className="text-amber-500" />
                                <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.15em]">現在のPING</span>
                            </div>
                            <span className="text-base font-black font-mono text-amber-600 leading-none">{m.stats.rtt_ms !== undefined ? `${m.stats.rtt_ms}ms` : "--"}</span>
                        </div>
                    </div>
                )}
            </div>
        </motion.div>
    );
};
