import { motion } from "framer-motion";
import { ChevronRight, Link2, Globe, Cloud, CloudOff, RefreshCw, AlertCircle, CheckCircle2 } from "lucide-react";
import { Mapping } from "../types";
import { twMerge } from "tailwind-merge";
import { clsx, type ClassValue } from "clsx";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

interface MappingCardProps {
    mapping: Mapping;
    isDeleteMode: boolean;
    isSelected: boolean;
    onSelect: (id: string) => void;
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
    return (
        <motion.div
            key={m.id}
            layout
            onClick={() => isDeleteMode ? onSelect(m.id) : onEdit(m)}
            className={cn(
                "group relative border-2 rounded-2xl overflow-hidden transition-all duration-300 select-none",
                (isDeleteMode || !m.isRunning) && "cursor-pointer",
                isSelected ? "border-[#4285F4] bg-[#E8F0FE]" :
                    m.isRunning ? "border-[#AECBFA] bg-[#F8FAFF]" : "border-[#DADCE0] bg-white hover:border-[#BDC1C6]"
            )}
        >
            <div className="py-3.5 px-6 flex items-center gap-5">
                {isDeleteMode ? (
                    <div className={cn(
                        "w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all",
                        isSelected ? "bg-[#4285F4] border-[#4285F4]" : "border-[#BDC1C6]"
                    )}>
                        {isSelected && <CheckCircle2 className="w-3.5 h-3.5 text-white" />}
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
                            onClick={(e) => onToggleConnect(e, m)}
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

            {m.error && (
                <div className="px-5 py-2.5 bg-[#FEEBEE] text-[#C5221F] text-[11px] font-bold border-t border-[#FAD2D8] flex items-center gap-2">
                    <AlertCircle className="w-4 h-4 shrink-0" />
                    <span>接続に失敗しました。詳細はコンソールを確認してください。</span>
                </div>
            )}
        </motion.div>
    );
};
