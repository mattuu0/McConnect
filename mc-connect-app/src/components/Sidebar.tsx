import { useState } from "react";
import { Shield, Terminal, Menu, Server, Settings } from "lucide-react";
import { View, AppSettings } from "../types";
import { twMerge } from "tailwind-merge";
import { clsx, type ClassValue } from "clsx";
import { motion } from "framer-motion";

/**
 * Tailwindのクラス名を条件に応じて結合し、マージするためのユーティリティ関数
 * @param inputs クラス名のリスト（文字列、オブジェクト、配列など）
 */
function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

/**
 * サイドバーコンポーネントのプロパティ定義
 */
interface SidebarProps {
    /** 現在表示中の表示モード */
    currentView: View;
    /** 表示モードを切り替えるための関数 */
    setCurrentView: (view: View) => void;
    /** アプリ設定 */
    settings: AppSettings;
}

/**
 * アプリケーションのナビゲーション（デスクトップ：サイドバー、モバイル：ボトムバー）を管理するコンポーネント
 */
export const Sidebar = ({ currentView, setCurrentView, settings }: SidebarProps) => {
    // デスクトップ版のサイドバーが開いているかどうか（折りたたみ状態）の管理
    const [isSidebarOpen, setSidebarOpen] = useState(true);

    /**
     * サイドバー内の各ナビゲーション項目を表示するインナーコンポーネント
     */
    const SidebarItem = ({ viewId, icon: Icon, label }: { viewId: View, icon: any, label: string }) => (
        <button
            onClick={() => setCurrentView(viewId)}
            className={cn(
                "w-full flex items-center space-x-3 px-6 py-4 border-r-4 transition-all outline-none group relative overflow-hidden",
                currentView === viewId
                    ? "text-[#16a34a] bg-green-50 border-[#16a34a]"
                    : "text-slate-400 border-transparent hover:bg-slate-50",
                !isSidebarOpen && "px-0 justify-center space-x-0"
            )}
        >
            {/* アイコンの表示 */}
            <Icon size={20} className="shrink-0" />

            {/* ラベルの表示：サイドバーが閉じている時は幅と透明度を制御 */}
            <span
                className={cn(
                    "font-bold transition-all duration-300 origin-left whitespace-nowrap",
                    !isSidebarOpen ? "w-0 opacity-0 pointer-events-none" : "w-auto opacity-100"
                )}
            >
                {label}
            </span>

            {/* サイドバーが閉じている時のみ、ホバー時にツールチップを表示 */}
            {!isSidebarOpen && (
                <div className="absolute left-full ml-4 px-3 py-2 bg-[#1e293b] text-white text-[11px] rounded-lg opacity-0 group-hover:opacity-100 invisible group-hover:visible pointer-events-none transition-all duration-200 -translate-x-2 group-hover:translate-x-0 whitespace-nowrap z-50 shadow-xl hidden sidebar:block">
                    {label}
                </div>
            )}
        </button>
    );

    return (
        <>
            {/* デスクトップ用サイドバー（画面幅がsidebar以上の場合に表示） */}
            <motion.aside
                initial={false}
                animate={{ width: isSidebarOpen ? 256 : 80 }}
                transition={{ type: "spring", stiffness: 200, damping: 25, mass: 0.5 }}
                className="hidden sidebar:flex flex-col bg-white border-r border-slate-200 h-screen shadow-sm z-40 relative"
            >
                {/* サイドバーヘッダー：ロゴと開閉ボタン */}
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

                {/* ナビゲーションメニュー */}
                <nav className="flex-1 py-6 space-y-0.5">
                    <SidebarItem viewId="dashboard" icon={Shield} label="トンネル管理" />
                    {settings.serverModeEnabled && (
                        <SidebarItem viewId="server" icon={Server} label="サーバー管理" />
                    )}
                    <SidebarItem viewId="console" icon={Terminal} label="システムログ" />
                    <SidebarItem viewId="settings" icon={Settings} label="設定" />
                </nav>

                {/* サイドバーフッター：稼働状態の表示 */}
                <div className="p-4 border-t border-slate-100 overflow-hidden">
                    <div className={cn("flex items-center space-x-2 px-2 text-[11px] font-bold text-[#16a34a]", !isSidebarOpen && "justify-center px-0")}>
                        <div className="w-2.5 h-2.5 bg-[#16a34a] rounded-full animate-pulse shrink-0" />
                        {isSidebarOpen && <span className="truncate">RUNNING</span>}
                    </div>
                </div>
            </motion.aside>

            {/* モバイル用ボトムナビゲーション（画面幅がsidebar未満の場合に表示） */}
            <nav className="sidebar:hidden fixed bottom-0 left-0 right-0 bg-white border-t border-slate-200 px-2 py-1 flex items-center justify-around z-50 shadow-[0_-4px_12px_rgba(0,0,0,0.05)] h-16">
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
                {settings.serverModeEnabled && (
                    <button
                        onClick={() => setCurrentView("server")}
                        className={cn(
                            "flex flex-col items-center justify-center flex-1 py-1 gap-1",
                            currentView === "server" ? "text-[#16a34a]" : "text-slate-400"
                        )}
                    >
                        <Server size={20} />
                        <span className="text-[10px] font-bold">サーバー</span>
                    </button>
                )}
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
                    onClick={() => setCurrentView("settings")}
                    className={cn(
                        "flex flex-col items-center justify-center flex-1 py-1 gap-1",
                        currentView === "settings" ? "text-[#16a34a]" : "text-slate-400"
                    )}
                >
                    <Settings size={20} />
                    <span className="text-[10px] font-bold">設定</span>
                </button>
            </nav>
        </>
    );
};
