import { Server, LayoutDashboard, Terminal, Info } from "lucide-react";
import { View, Mapping } from "../types";
import { twMerge } from "tailwind-merge";
import { clsx, type ClassValue } from "clsx";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

interface SidebarProps {
    currentView: View;
    setCurrentView: (view: View) => void;
    mappings: Mapping[];
}

export const Sidebar = ({ currentView, setCurrentView, mappings }: SidebarProps) => {
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
    );
};
