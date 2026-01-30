import { useState } from "react";
import { Server, LayoutDashboard, Terminal, Info, ChevronLeft, ChevronRight } from "lucide-react";
import { View, Mapping } from "../types";
import { twMerge } from "tailwind-merge";
import { clsx, type ClassValue } from "clsx";
import { motion } from "framer-motion";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

interface SidebarProps {
    currentView: View;
    setCurrentView: (view: View) => void;
    mappings: Mapping[];
}

export const Sidebar = ({ currentView, setCurrentView, mappings }: SidebarProps) => {
    const [isCollapsed, setIsCollapsed] = useState(false);

    const SidebarItem = ({ id, icon: Icon, label }: { id: View, icon: any, label: string }) => (
        <button
            onClick={() => setCurrentView(id)}
            className={cn(
                "w-full flex items-center px-4 py-3 rounded-xl text-sm font-bold transition-all gap-3 relative group",
                currentView === id
                    ? "bg-[#E8F0FE] text-[#1967D2]"
                    : "text-[#5F6368] hover:bg-[#F1F3F4]",
                isCollapsed && "justify-center px-0"
            )}
        >
            <Icon className="w-5 h-5 shrink-0" />
            <span
                className={cn(
                    "truncate transition-all duration-300 origin-left",
                    isCollapsed ? "w-0 opacity-0 scale-95 pointer-events-none" : "w-auto opacity-100 scale-100"
                )}
            >
                {label}
            </span>
            {isCollapsed && (
                <div className="absolute left-full ml-4 px-3 py-2 bg-[#202124] text-white text-[10px] rounded-lg opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity whitespace-nowrap z-50 shadow-xl">
                    {label}
                </div>
            )}
        </button>
    );

    return (
        <motion.aside
            animate={{ width: isCollapsed ? 80 : 256 }}
            transition={{ type: "spring", stiffness: 300, damping: 30 }}
            className="bg-white border-r border-[#DADCE0] flex flex-col relative shrink-0 overflow-visible"
        >
            <button
                onClick={() => setIsCollapsed(!isCollapsed)}
                className="absolute -right-3 top-24 w-6 h-6 bg-white border border-[#DADCE0] rounded-full flex items-center justify-center text-[#5F6368] hover:text-[#1967D2] hover:border-[#1967D2] shadow-sm z-50 transition-colors"
                title={isCollapsed ? "展開" : "折り畳む"}
            >
                {isCollapsed ? <ChevronRight className="w-4 h-4" /> : <ChevronLeft className="w-4 h-4" />}
            </button>

            <div className="p-4 pt-12 flex flex-col h-full overflow-hidden">
                <div className={cn("flex items-center gap-3 px-2 mb-10 transition-all", isCollapsed && "justify-center px-0")}>
                    <div className="w-10 h-10 bg-[#4285F4] rounded-xl flex items-center justify-center shadow-lg shadow-blue-100 shrink-0">
                        <Server className="text-white w-6 h-6" />
                    </div>
                    <div
                        className={cn(
                            "min-w-0 transition-all duration-300 origin-left",
                            isCollapsed ? "w-0 opacity-0 invisible" : "w-auto opacity-100"
                        )}
                    >
                        <h1 className="text-lg font-bold tracking-tight text-[#3C4043] truncate">McConnect</h1>
                        <p className="text-[10px] font-bold text-[#70757A] uppercase tracking-wider truncate">Proxy</p>
                    </div>
                </div>

                <nav className="flex-1 space-y-1">
                    <SidebarItem id="dashboard" icon={LayoutDashboard} label="ダッシュボード" />
                    <SidebarItem id="console" icon={Terminal} label="コンソール" />
                    <SidebarItem id="about" icon={Info} label="情報" />
                </nav>

                <div className={cn("mt-auto border-t border-[#DADCE0] pt-4 px-2 overflow-hidden", isCollapsed && "border-t-0")}>
                    <div className={cn("flex items-center gap-2 mb-2", isCollapsed && "justify-center")}>
                        <div className={cn("w-2 h-2 rounded-full shrink-0", mappings.some(m => m.isRunning) ? "bg-[#34A853] animate-pulse" : "bg-[#EA4335]")} />
                        <span
                            className={cn(
                                "text-[11px] font-bold text-[#70757A] uppercase tracking-wider truncate transition-all duration-300",
                                isCollapsed ? "w-0 opacity-0" : "w-auto opacity-100"
                            )}
                        >
                            {mappings.some(m => m.isRunning) ? "実行中" : "待機中"}
                        </span>
                    </div>
                    {!isCollapsed && (
                        <p className="text-[10px] text-[#9AA0A6] truncate px-2">
                            {mappings.filter(m => m.isRunning).length} 個のトンネル有効
                        </p>
                    )}
                </div>
            </div>
        </motion.aside>
    );
};
