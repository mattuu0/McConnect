import { motion } from "framer-motion";
import { Link2, Globe, Cloud, CloudOff, RefreshCw, AlertCircle, CheckCircle2, ArrowUp, ArrowDown } from "lucide-react";
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

const Sparkline = ({ data, color, height = 30 }: { data: number[], color: string, height?: number }) => {
    if (!data || data.length < 2) return null;
    const max = Math.max(...data, 1);
    const width = 100;
    const points = data.map((d, i) => {
        const x = (i / (data.length - 1)) * width;
        const y = height - (d / max) * height;
        return `${x},${y}`;
    }).join(" ");

    return (
        <svg viewBox={`0 0 ${width} ${height}`} className="w-16 h-8 opacity-50 overflow-visible hidden sm:block">
            <motion.polyline
                fill="none"
                stroke={color}
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                initial={{ pathLength: 0 }}
                animate={{ pathLength: 1 }}
                points={points}
            />
        </svg>
    );
};

export const MappingCard = ({
    mapping: m,
    isDeleteMode,
    isSelected,
    onSelect,
    onEdit,
    onToggleConnect
}: MappingCardProps) => {
    const formatSpeed = (bytesPerSec: number) => {
        if (bytesPerSec === 0) return "0 B/s";
        const k = 1024;
        const sizes = ["B/s", "KB/s", "MB/s", "GB/s"];
        const i = Math.floor(Math.log(bytesPerSec) / Math.log(k));
        return parseFloat((bytesPerSec / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
    };

    return (
        <motion.div
            key={m.id}
            layout
            onClick={() => isDeleteMode ? onSelect(m.id) : onEdit(m)}
            className={`
                group relative border-2 rounded-2xl overflow-hidden transition-all duration-300 select-none
                ${(isDeleteMode || !m.isRunning) ? "cursor-pointer" : ""}
                ${isSelected ? "border-[#4285F4] bg-[#E8F0FE]" :
                    m.isRunning ? "border-[#AECBFA] bg-[#F8FAFF]" : "border-[#DADCE0] bg-white hover:border-[#BDC1C6]"}
            `}
        >
            <div className="py-4 px-6 flex flex-col sm:flex-row sm:items-center gap-6">
                <div className="flex items-center gap-5 flex-1 min-w-0">
                    {isDeleteMode ? (
                        <div className={`
                            w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all shrink-0
                            ${isSelected ? "bg-[#4285F4] border-[#4285F4]" : "border-[#BDC1C6]"}
                        `}>
                            {isSelected && <CheckCircle2 className="w-3.5 h-3.5 text-white" />}
                        </div>
                    ) : (
                        <div className={`
                            w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition-colors
                            ${m.loading ? "bg-[#F1F3F4] text-[#4285F4]" :
                                m.isRunning ? "bg-[#4285F4] text-white shadow-md shadow-blue-50" : "bg-[#F1F3F4] text-[#5F6368]"}
                        `}>
                            {m.loading ? <RefreshCw className="w-5 h-5 animate-spin" /> :
                                m.isRunning ? <Cloud className="w-5 h-5" /> : <CloudOff className="w-5 h-5 opacity-40" />}
                        </div>
                    )}

                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-3 mb-3">
                            <span className="text-base font-bold text-[#3C4043] truncate">{m.wsUrl.replace(/^ws?s:\/\//, '').split('/')[0]}</span>
                            <span className="h-6 px-2 flex items-center bg-[#F1F3F4] rounded text-[10px] sm:text-[11px] font-bold text-[#5F6368] uppercase tracking-wider shrink-0">
                                {m.protocol}
                            </span>
                        </div>

                        <div className="flex items-center gap-4 sm:gap-8">
                            <div className="flex flex-col min-w-0">
                                <span className="text-[10px] font-bold text-[#9AA0A6] uppercase tracking-tighter leading-none mb-1.5 whitespace-nowrap">Local Source</span>
                                <div className="flex items-center gap-1.5 text-[11px] sm:text-[12px] font-medium text-[#5F6368] truncate">
                                    <Link2 className="w-4 h-4 text-[#4285F4] shrink-0" />
                                    <span className="font-mono bg-[#F1F3F4]/50 px-1.5 py-0.5 rounded truncate">{m.bindAddr}:{m.localPort}</span>
                                </div>
                            </div>

                            <div className="flex-1 flex items-center justify-center relative h-10 min-w-[40px] max-w-[200px]">
                                <div className="absolute inset-x-0 top-1/2 -translate-y-1/2 h-1.5 bg-[#F1F3F4] rounded-full overflow-hidden">
                                    {m.isRunning ? (
                                        <>
                                            <div className="absolute inset-0 bg-[#E8F0FE]" />
                                            {/* メインのパルスフロー */}
                                            <motion.div
                                                initial={{ left: "-100%" }}
                                                animate={{ left: "100%" }}
                                                transition={{
                                                    repeat: Infinity,
                                                    duration: 1.2,
                                                    ease: "linear",
                                                }}
                                                className="absolute inset-y-0 w-1/2 bg-gradient-to-r from-transparent via-[#4285F4] to-transparent shadow-[0_0_10px_#4285F4]"
                                            />
                                            {/* 高速なパルス */}
                                            <motion.div
                                                initial={{ left: "-100%" }}
                                                animate={{ left: "100%" }}
                                                transition={{
                                                    repeat: Infinity,
                                                    duration: 0.7,
                                                    ease: "linear",
                                                    delay: 0.4
                                                }}
                                                className="absolute inset-y-0 w-1/4 bg-white/60 blur-[1px]"
                                            />
                                        </>
                                    ) : (
                                        <div className="w-full h-full bg-[#DADCE0] opacity-40" />
                                    )}
                                </div>
                            </div>

                            <div className="flex flex-col min-w-0 text-right sm:text-left">
                                <div className="flex items-center justify-end sm:justify-start gap-2 mb-1.5">
                                    <span className="text-[10px] font-bold text-[#9AA0A6] uppercase tracking-tighter leading-none whitespace-nowrap">Remote Target</span>
                                    <span className={`
                                        text-[9px] font-bold px-1.5 py-0.5 rounded leading-none shrink-0
                                        ${m.isRunning ? "bg-[#E6F4EA] text-[#188038]" : m.loading ? "bg-[#E8F0FE] text-[#1967D2]" : "bg-[#F1F3F4] text-[#70757A]"}
                                    `}>
                                        {m.isRunning ? "接続済み" : m.loading ? "接続中" : "未接続"}
                                    </span>
                                </div>
                                <div className="flex items-center justify-end sm:justify-start gap-1.5 text-[11px] sm:text-[12px] font-medium text-[#5F6368] truncate">
                                    <Globe className="w-4 h-4 text-[#34A853] shrink-0" />
                                    <span className="font-mono bg-[#F1F3F4]/50 px-1.5 py-0.5 rounded truncate">Port:{m.remotePort}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div className="flex items-center justify-end shrink-0">
                    {!isDeleteMode && (
                        <button
                            onClick={(e) => onToggleConnect(e, m)}
                            disabled={m.loading || m.hasFailed}
                            className={`
                                w-full sm:w-auto px-10 py-4.5 rounded-xl transition-all shadow-md active:scale-95 text-sm font-bold min-w-[130px] flex items-center justify-center gap-2
                                ${m.isRunning
                                    ? "bg-white border-2 border-[#EA4335] text-[#EA4335] hover:bg-[#FEEBEE]"
                                    : m.hasFailed
                                        ? "bg-[#EA4335] text-white shadow-red-100"
                                        : m.loading
                                            ? "bg-[#E8F0FE] text-[#1967D2] cursor-not-allowed shadow-none"
                                            : "bg-[#4285F4] text-white hover:bg-[#1A73E8] shadow-blue-100"}
                            `}
                        >
                            {m.loading ? (
                                <RefreshCw className="w-4 h-4 animate-spin" />
                            ) : m.hasFailed ? (
                                "失敗"
                            ) : m.isRunning ? (
                                "切断"
                            ) : (
                                "接続"
                            )}
                        </button>
                    )}
                </div>
            </div>

            {m.isRunning && m.stats && (
                <div className="px-6 py-3 bg-[#F8F9FA] border-t border-[#DADCE0] flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                    <div className="flex items-center gap-8 grow">
                        <div className="flex items-center gap-4 flex-1">
                            <div className="flex flex-col min-w-0">
                                <div className="flex items-center gap-1.5 text-[10px] font-bold text-[#137333] uppercase">
                                    <ArrowUp className="w-3 h-3" />
                                    <span className="truncate">Up</span>
                                </div>
                                <span className="text-[13px] font-bold text-[#3C4043] tabular-nums font-mono truncate">{formatSpeed(m.stats.upload_speed || 0)}</span>
                            </div>
                            <Sparkline data={m.speedHistory?.up || []} color="#34A853" />
                        </div>

                        <div className="flex items-center gap-4 flex-1">
                            <div className="flex flex-col min-w-0">
                                <div className="flex items-center gap-1.5 text-[10px] font-bold text-[#EA4335] uppercase">
                                    <ArrowDown className="w-3 h-3" />
                                    <span className="truncate">Down</span>
                                </div>
                                <span className="text-[13px] font-bold text-[#3C4043] tabular-nums font-mono truncate">{formatSpeed(m.stats.download_speed || 0)}</span>
                            </div>
                            <Sparkline data={m.speedHistory?.down || []} color="#EA4335" />
                        </div>
                    </div>

                    <div className="flex items-center gap-4 shrink-0">
                        <Sparkline data={m.latencyHistory || []} color="#1967D2" />
                        <div className="flex flex-col items-end shrink-0">
                            <span className="text-[9px] font-bold text-[#9AA0A6] uppercase mb-0.5">Latency</span>
                            <div className="flex items-center gap-2 bg-white px-3 py-1 rounded-full border border-[#DADCE0] shadow-sm">
                                <span className="text-[11px] font-bold text-[#1967D2] tabular-nums">{m.stats.rtt_ms !== undefined ? `${m.stats.rtt_ms}ms` : "--"}</span>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {m.error && (
                <div className="px-5 py-2.5 bg-[#FEEBEE] text-[#C5221F] text-[11px] font-bold border-t border-[#FAD2D8] flex items-center gap-2">
                    <AlertCircle className="w-4 h-4 shrink-0" />
                    <span>エラーが発生しました。詳細はコンソールを確認してください。</span>
                </div>
            )}
        </motion.div>
    );
};
