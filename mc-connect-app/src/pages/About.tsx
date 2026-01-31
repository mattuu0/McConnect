import { motion } from "framer-motion";
import { Server, Shield, Heart } from "lucide-react";

/**
 * アプリケーションの詳細情報やバージョン情報を表示する「バージョン情報」ページ
 */
export const About = () => {
    return (
        <motion.div
            key="about"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex-1 flex flex-col w-full min-h-screen bg-[#f8fafc]"
        >
            {/* ヘッダーセクション */}
            <header className="sticky top-0 z-30 w-full bg-white/90 backdrop-blur-md border-b border-slate-200 px-6 sm:px-12 h-20 shrink-0 flex items-center shadow-sm">
                <div className="w-full flex justify-between items-center">
                    <h2 className="text-xl font-black text-slate-800">アプリケーション情報</h2>
                    <div className="p-2.5 bg-white border-2 border-slate-100 rounded-xl text-[#16a34a] shadow-sm">
                        <Shield size={20} />
                    </div>
                </div>
            </header>

            {/* メインコンテンツセクション */}
            <div className="max-w-5xl mx-auto w-full px-6 py-10 flex-1 flex flex-col pb-32">
                <div className="flex-1 bg-white rounded-[2.5rem] border border-slate-200 p-8 lg:p-16 shadow-[0_4px_20px_rgba(0,0,0,0.03)] text-center flex flex-col items-center justify-center space-y-10">

                    {/* アプリケーションアイコン（Serverアイコンを使用） */}
                    <div className="w-28 h-28 bg-[#16a34a] rounded-[2.5rem] flex items-center justify-center shadow-2xl shadow-green-100 border-b-8 border-green-800 relative group shrink-0">
                        <Server className="text-white w-14 h-14 group-hover:scale-110 transition-transform" />
                    </div>

                    {/* アプリ名と説明文 */}
                    <div className="space-y-4">
                        <h2 className="text-4xl lg:text-5xl font-black tracking-tighter text-slate-900 italic uppercase leading-tight">
                            MC CONNECT <span className="text-[#16a34a]">v0.1.0</span>
                        </h2>
                        <p className="text-slate-500 max-w-lg mx-auto leading-relaxed font-bold text-lg">
                            Minecraft の TCP 通信を WebSocket にカプセル化し、ファイアウォールを超えて自由に接続するための次世代プロキシツール。
                        </p>
                    </div>

                    {/* 技術スタックとライセンス情報 */}
                    <div className="flex flex-wrap justify-center gap-4">
                        <div className="px-8 py-4 bg-slate-50 border-2 border-slate-100 rounded-2xl text-xs font-black text-slate-400 uppercase tracking-widest">
                            Powered by Rust & Tauri
                        </div>
                        <div className="px-8 py-4 bg-slate-50 border-2 border-slate-100 rounded-2xl text-xs font-black text-slate-400 uppercase tracking-widest">
                            MIT License
                        </div>
                    </div>

                    {/* フッターメッセージ */}
                    <div className="pt-10 text-slate-200 flex items-center gap-2">
                        <Heart size={16} className="fill-current text-red-400" />
                        <span className="text-[10px] font-black uppercase tracking-[0.3em]">Build with Passion</span>
                    </div>
                </div>
            </div>
        </motion.div>
    );
};

