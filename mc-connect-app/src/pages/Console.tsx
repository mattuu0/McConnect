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

            {/* ログ表示エリア（ターミナル風デザイン） */}
            <div className="flex-1 flex flex-col p-4 sm:p-10 pb-32">
                <div className="flex-1 flex flex-col bg-[#1e293b] rounded-[2rem] border-4 border-slate-800 shadow-2xl overflow-hidden relative">
                    {/* ターミナルヘッダー（OSのウィンドウ風デザイン） */}
                    <div className="h-10 bg-slate-800 flex items-center px-4 space-x-2 shrink-0">
                        <div className="w-3 h-3 rounded-full bg-red-500" />
                        <div className="w-3 h-3 rounded-full bg-amber-500" />
                        <div className="w-3 h-3 rounded-full bg-green-500" />
                        <span className="ml-4 text-[10px] font-bold text-slate-400 font-mono tracking-widest uppercase">mc-connect-bridge.shell</span>
                    </div>

                    {/* ログ本文エリア */}
                    <div className="flex-1 overflow-y-auto p-4 sm:p-6 font-mono text-[12px] sidebar:text-[14px] leading-relaxed scrollbar-terminal">
                        {logs.length === 0 ? (
                            // ログがない場合の表示
                            <div className="h-full flex flex-col items-center justify-center text-slate-500 space-y-2 opacity-50">
                                <TerminalIcon size={48} />
                                <p className="font-black">NO ACTIVE LOGS</p>
                            </div>
                        ) : (
                            // ログリストのレンダリング
                            <div className="space-y-1.5">
                                {logs.map((log, index) => (
                                    <div key={index} className="flex gap-4 group hover:bg-slate-800/50 rounded px-2 -mx-2 transition-colors">
                                        {/* タイムスタンプ */}
                                        <span className="text-slate-500 font-bold select-none shrink-0 min-w-[85px]">
                                            {log.timestamp}
                                        </span>
                                        {/* ログレベルと本文 */}
                                        <span className="flex-1">
                                            {/* ログレベルに応じた色分け表示 */}
                                            <span className={`
                                                font-black mr-3 px-1.5 rounded-[4px] text-[10px] uppercase tracking-tighter
                                                ${log.level === "ERROR" ? "text-red-400 bg-red-900/30" :
                                                    log.level === "SUCCESS" ? "text-green-400 bg-green-900/30" :
                                                        log.level === "INFO" ? "text-cyan-400 bg-cyan-900/30" : "text-slate-400 bg-slate-700/50"}
                                            `}>
                                                {log.level}
                                            </span>
                                            {/* ログ内容 */}
                                            <span className={
                                                log.level === "ERROR" ? "text-red-200" :
                                                    log.level === "SUCCESS" ? "text-green-200" :
                                                        log.level === "INFO" ? "text-cyan-200" : "text-slate-200"
                                            }>
                                                {log.message}
                                            </span>
                                        </span>
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

