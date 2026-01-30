import { motion } from "framer-motion";
import { Terminal } from "lucide-react";
import { LogEntry } from "../types";
import { twMerge } from "tailwind-merge";
import { clsx, type ClassValue } from "clsx";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

interface ConsoleProps {
    logs: LogEntry[];
    logEndRef: React.RefObject<HTMLDivElement | null>;
}

export const Console = ({ logs, logEndRef }: ConsoleProps) => {
    return (
        <motion.div
            key="console"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="flex-1 flex flex-col min-h-0"
        >
            <div className="flex items-center gap-3 mb-4">
                <Terminal className="w-6 h-6 text-[#1A73E8]" />
                <h2 className="text-2xl font-bold text-[#3C4043]">システムログ</h2>
            </div>

            <div className="flex-1 bg-[#202124] rounded-2xl overflow-hidden shadow-2xl flex flex-col">
                <div className="px-4 py-2 bg-[#303134] border-b border-[#3C4043] flex items-center gap-2">
                    <div className="flex gap-1.5">
                        <div className="w-3 h-3 rounded-full bg-[#FF5F56]" />
                        <div className="w-3 h-3 rounded-full bg-[#FFBD2E]" />
                        <div className="w-3 h-3 rounded-full bg-[#27C93F]" />
                    </div>
                    <span className="text-[10px] font-mono text-[#9AA0A6] ml-2">mc-connect-session.log</span>
                </div>
                <div className="flex-1 overflow-y-auto p-4 font-mono text-sm space-y-1 custom-scrollbar">
                    {logs.map((log, i) => (
                        <div key={i} className="flex gap-3 py-0.5 group">
                            <span className="text-[#5F6368] shrink-0 select-none">[{log.timestamp}]</span>
                            <span className={cn(
                                "font-bold shrink-0 min-w-[60px]",
                                log.level === "ERROR" ? "text-[#F28B82]" :
                                    log.level === "SUCCESS" ? "text-[#81C995]" :
                                        log.level === "INFO" ? "text-[#8AB4F8]" : "text-[#BDC1C6]"
                            )}>
                                {log.level}
                            </span>
                            <span className="text-[#E8EAED] break-all group-hover:bg-white/5 transition-colors">{log.message}</span>
                        </div>
                    ))}
                    <div ref={logEndRef} />
                </div>
            </div>
        </motion.div>
    );
};
