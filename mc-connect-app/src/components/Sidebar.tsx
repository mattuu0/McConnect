import { useState } from "react";
import { Shield, Terminal, Info, Menu } from "lucide-react";
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
    const [isSidebarOpen, setSidebarOpen] = useState(true);

    const SidebarItem = ({ id, icon: Icon, label }: { id: View, icon: any, label: string }) => (
        <button
            onClick={() => setCurrentView(id)}
            className={cn(
                "w-full flex items-center space-x-3 px-6 py-4 border-r-4 transition-all outline-none group relative",
                currentView === id
                    ? "text-[#16a34a] bg-green-50 border-[#16a34a]"
                    : "text-slate-400 border-transparent hover:bg-slate-50",
                !isSidebarOpen && "px-2 justify-center"
            )}
        >
            <Icon size={20} className="shrink-0" />
            <span
                className={cn(
                    "font-bold transition-all duration-300 origin-left whitespace-nowrap overflow-hidden",
                    !isSidebarOpen ? "w-0 opacity-0" : "w-auto opacity-100"
                )}
            >
                {label}
            </span>

            {!isSidebarOpen && (
                <div className="absolute left-full ml-4 px-3 py-2 bg-[#1e293b] text-white text-[11px] rounded-lg opacity-0 group-hover:opacity-100 invisible group-hover:visible pointer-events-none transition-all duration-200 -translate-x-2 group-hover:translate-x-0 whitespace-nowrap z-50 shadow-xl hidden sm:block">
                    {label}
                </div>
            )}
        </button>
    );

    return (
        <>
            {/* Desktop Sidebar */}
            <motion.aside
                animate={{ width: isSidebarOpen ? 256 : 80 }}
                transition={{ type: "spring", stiffness: 300, damping: 30 }}
                className="hidden lg:flex flex-col bg-white border-r border-slate-200 h-screen transition-all duration-300 shadow-sm z-40 relative"
            >
                <div className="p-4 h-16 flex items-center justify-between border-b border-slate-100 px-6 overflow-hidden">
                    {isSidebarOpen && (
                        <motion.h1
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            className="text-xl font-black italic text-[#16a34a] tracking-tighter truncate leading-none mt-1"
                        >
                            MC CONNECT
                        </motion.h1>
                    )}
                    <button
                        onClick={() => setSidebarOpen(!isSidebarOpen)}
                        className="p-2 hover:bg-slate-50 rounded-lg text-slate-400 mx-auto outline-none shrink-0"
                    >
                        <Menu size={20} />
                    </button>
                </div>

                <nav className="flex-1 py-6 space-y-0.5">
                    <SidebarItem id="dashboard" icon={Shield} label="トンネル管理" />
                    <SidebarItem id="console" icon={Terminal} label="システムログ" />
                    <SidebarItem id="about" icon={Info} label="アプリケーション情報" />
                </nav>

                <div className="p-4 border-t border-slate-100 overflow-hidden">
                    <div className={cn("flex items-center space-x-2 px-2 text-[11px] font-bold text-[#16a34a]", !isSidebarOpen && "justify-center px-0")}>
                        <div className="w-2.5 h-2.5 bg-[#16a34a] rounded-full animate-pulse shrink-0" />
                        {isSidebarOpen && <span className="truncate">RUNNING</span>}
                    </div>
                </div>
            </motion.aside>

            {/* Mobile Bottom Navigation */}
            <nav className="lg:hidden fixed bottom-0 left-0 right-0 bg-white border-t border-slate-200 px-2 py-1 flex items-center justify-around z-50 shadow-[0_-4px_12px_rgba(0,0,0,0.05)] h-16">
                <button
                    onClick={() => setCurrentView("dashboard")}
                    className={cn(
                        "flex flex-col items-center justify-center flex-1 py-1 gap-1",
                        currentView === "dashboard" ? "text-[#16a34a]" : "text-slate-400"
                    )}
                >
                    <Shield size={20} />
                    <span className="text-[10px] font-bold">管理</span>
                </button>
                <button
                    onClick={() => setCurrentView("console")}
                    className={cn(
                        "flex flex-col items-center justify-center flex-1 py-1 gap-1",
                        currentView === "console" ? "text-[#16a34a]" : "text-slate-400"
                    )}
                >
                    <Terminal size={20} />
                    <span className="text-[10px] font-bold">ログ</span>
                </button>
                <button
                    onClick={() => setCurrentView("about")}
                    className={cn(
                        "flex flex-col items-center justify-center flex-1 py-1 gap-1",
                        currentView === "about" ? "text-[#16a34a]" : "text-slate-400"
                    )}
                >
                    <Info size={20} />
                    <span className="text-[10px] font-bold">情報</span>
                </button>
            </nav>
        </>
    );
};
