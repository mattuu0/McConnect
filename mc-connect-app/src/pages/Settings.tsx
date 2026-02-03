import { motion } from "framer-motion";
import { Settings as SettingsIcon, Server, ShieldCheck } from "lucide-react";
import { AppSettings } from "../types";

interface SettingsPageProps {
    settings: AppSettings;
    onSettingsChange: (settings: AppSettings) => void;
}

export const SettingsPage = ({ settings, onSettingsChange }: SettingsPageProps) => {
    return (
        <motion.div
            key="settings"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex-1 flex flex-col w-full"
        >
            <header className="sticky top-0 z-30 w-full bg-white/90 backdrop-blur-md border-b border-slate-200 px-6 sm:px-12 h-20 shrink-0 flex items-center shadow-sm">
                <div className="w-full flex justify-between items-center">
                    <h2 className="text-xl font-black text-slate-800">設定</h2>
                    <div className="p-2.5 bg-slate-100 rounded-xl text-slate-600">
                        <SettingsIcon size={20} />
                    </div>
                </div>
            </header>

            <div className="max-w-3xl mx-auto w-full px-6 py-10 space-y-8">
                <section className="bg-white rounded-3xl border border-slate-200 p-8 shadow-sm">
                    <div className="flex items-center gap-4 mb-6">
                        <div className="p-3 bg-green-50 text-[#16a34a] rounded-2xl">
                            <Server size={24} />
                        </div>
                        <div>
                            <h3 className="text-lg font-black text-slate-900">サーバーモード</h3>
                            <p className="text-sm text-slate-400 font-bold">このアプリをプロキシサーバーとして動作させます</p>
                        </div>
                    </div>

                    <div className="flex items-center justify-between p-4 bg-slate-50 rounded-2xl border border-slate-100">
                        <span className="font-bold text-slate-700">サーバー機能を有効化</span>
                        <button
                            onClick={() => onSettingsChange({ ...settings, serverModeEnabled: !settings.serverModeEnabled })}
                            className={`
                                relative w-14 h-8 rounded-full transition-colors duration-300 outline-none
                                ${settings.serverModeEnabled ? 'bg-[#16a34a]' : 'bg-slate-300'}
                            `}
                        >
                            <motion.div
                                animate={{ x: settings.serverModeEnabled ? 26 : 4 }}
                                className="absolute top-1 w-6 h-6 bg-white rounded-full shadow-sm"
                            />
                        </button>
                    </div>

                    {settings.serverModeEnabled && (
                        <div className="mt-6 p-4 bg-blue-50 rounded-2xl border border-blue-100 flex gap-3">
                            <ShieldCheck className="text-blue-500 shrink-0" size={20} />
                            <p className="text-xs text-blue-700 font-bold leading-relaxed">
                                サーバーモードが有効になりました。サイドバーに「サーバー管理」タブが表示されます。
                                ここでは待ち受けポートの設定や、クライアント向けの接続設定（公開鍵含む）の生成が行えます。
                            </p>
                        </div>
                    )}
                </section>
            </div>
        </motion.div>
    );
};
