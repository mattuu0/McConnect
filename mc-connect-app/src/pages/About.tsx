import { motion } from "framer-motion";
import { Server } from "lucide-react";

export const About = () => {
    return (
        <motion.div
            key="about"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="space-y-6 text-center pt-20"
        >
            <div className="flex justify-center mb-6">
                <div className="w-20 h-20 bg-[#4285F4] rounded-2xl flex items-center justify-center shadow-xl shadow-blue-200">
                    <Server className="text-white w-12 h-12" />
                </div>
            </div>
            <h2 className="text-3xl font-bold text-[#3C4043]">McConnect v0.1.0</h2>
            <p className="text-[#5F6368] max-w-md mx-auto leading-relaxed">
                Minecraft の TCP 通信を WebSocket にカプセル化し、ファイアウォールを超えて自由に接続するための次世代プロキシツール。
            </p>
            <div className="pt-10 flex justify-center gap-4">
                <div className="px-6 py-2 bg-white border border-[#DADCE0] rounded-full text-xs font-bold text-[#70757A]">
                    Powered by Rust & Tauri
                </div>
                <div className="px-6 py-2 bg-white border border-[#DADCE0] rounded-full text-xs font-bold text-[#70757A]">
                    MIT License
                </div>
            </div>
        </motion.div>
    );
};
