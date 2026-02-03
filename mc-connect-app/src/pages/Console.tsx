import { motion } from "framer-motion";
import { Terminal as TerminalIcon } from "lucide-react";
import { LogEntry } from "../types";

/**
 * ログ表示（コンソール）画面のプロパティ定義
 */
interface ConsoleProps {
    /** 表示するログエントリの配列 */
    logs: LogEntry[];
    /** ログの末尾に自動スクロールするためのRef */
    logEndRef: React.RefObject<HTMLDivElement | null>;
}

/**
 * システムの動作ログをターミナル風のUIで表示するコンポーネント
 */
export const Console = ({ logs, logEndRef }: ConsoleProps) => {
    return (
        <motion.div
            key="console"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex-1 flex flex-col w-full min-h-screen bg-[#f8fafc]"
        >
            {/* 上部ヘッダー */}
            <header className="sticky top-0 z-30 w-full bg-white/90 backdrop-blur-md border-b border-slate-200 px-6 sm:px-12 h-20 shrink-0 flex items-center shadow-sm">
                <div className="w-full flex justify-between items-center">
                    <h2 className="text-xl font-black text-slate-800">システムログ</h2>
                    <div className="p-2.5 bg-slate-900 rounded-xl text-green-400 shadow-lg">
                        <TerminalIcon size={20} />
                    </div>
                </div>
            </header>

            {/* ログ表示エリア（コマンドプロンプト風デザイン） */}
            <div className="flex-1 flex flex-col p-4 sm:p-8 pb-32">
                <div className="flex-1 flex flex-col bg-[#0c0c0c] border border-slate-700 shadow-2xl overflow-hidden relative">
                    {/* シンプルなステータスバー */}
                    <div className="h-8 bg-[#1e1e1e] flex items-center px-4 shrink-0 border-b border-slate-800">
                        <TerminalIcon size={14} className="text-slate-400 mr-3" />
                        <span className="text-[10px] font-bold text-slate-500 font-mono tracking-widest uppercase">McConnect Console Session</span>
                    </div>

                    {/* ログ本文エリア */}
                    <div className="flex-1 overflow-y-auto p-4 sm:p-6 font-mono text-[11px] sidebar:text-[12px] leading-relaxed scrollbar-terminal">
                        {logs.length === 0 ? (
                            // ログがない場合の表示
                            <div className="h-full flex flex-col items-center justify-center text-slate-700 space-y-2 opacity-50">
                                <TerminalIcon size={48} />
                                <p className="font-black">Terminal Ready - No Input</p>
                            </div>
                        ) : (
                            // ログリストのレンダリング
                            <div className="space-y-1">
                                {logs.map((log, index) => (
                                    <div key={index} className="flex gap-3 group hover:bg-white/5 rounded px-1 -mx-1 transition-colors">
                                        {/* タイムスタンプ */}
                                        <span className="text-slate-600 font-bold select-none shrink-0 w-16">
                                            [{log.timestamp}]
                                        </span>
                                        {/* ログレベルと本文 */}
                                        <div className="flex-1 min-w-0">
                                            {/* ログレベルに応じた色分け表示 */}
                                            <span className={`
                                                inline-block font-black mr-2 px-1 rounded-[2px] text-[10px] uppercase tracking-tighter
                                                ${log.level === "ERROR" ? "text-red-500" :
                                                    log.level === "SUCCESS" ? "text-green-500" :
                                                        log.level === "INFO" ? "text-blue-500" : "text-slate-500"}
                                            `}>
                                                {log.level}
                                            </span>
                                            {/* ログ内容 */}
                                            <span className={`
                                                break-all whitespace-pre-wrap
                                                ${log.level === "ERROR" ? "text-red-400" :
                                                    log.level === "SUCCESS" ? "text-green-400" :
                                                        log.level === "INFO" ? "text-slate-200" : "text-slate-300"}
                                            `}>
                                                {log.message}
                                            </span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                        {/* 自動スクロールのためのアンカー要素 */}
                        <div ref={logEndRef} className="h-4" />
                    </div>
                </div>
            </div>
        </motion.div>
    );
};

