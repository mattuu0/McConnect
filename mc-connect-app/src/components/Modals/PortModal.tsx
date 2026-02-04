import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Plus } from "lucide-react";

interface PortModalProps {
    isOpen: boolean;
    onClose: () => void;
    onAdd: (port: number, protocol: "TCP" | "UDP") => void;
}

export const PortModal = ({ isOpen, onClose, onAdd }: PortModalProps) => {
    const [port, setPort] = useState<string>("");
    const [protocol, setProtocol] = useState<"TCP" | "UDP">("TCP");

    const handleAdd = () => {
        const portNum = Number(port);
        if (isNaN(portNum) || portNum <= 0 || portNum > 65535) {
            alert("有効なポート番号を入力してください (1-65535)");
            return;
        }
        onAdd(portNum, protocol);
        setPort("");
        onClose();
    };

    return (
        <AnimatePresence>
            {isOpen && (
                <div className="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex justify-center items-center p-4 overflow-y-auto">
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.95 }}
                        className="bg-white w-full max-w-sm rounded-[2rem] shadow-2xl p-8 border border-slate-200 relative my-auto"
                    >
                        <div className="flex justify-between items-center mb-6">
                            <h3 className="text-lg font-black text-slate-900 italic tracking-tight uppercase">ポート追加</h3>
                            <button
                                onClick={onClose}
                                className="p-2 bg-slate-100 rounded-full text-slate-400 hover:text-slate-900 transition-colors"
                            >
                                <X size={18} />
                            </button>
                        </div>

                        <div className="space-y-5">
                            <div>
                                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">プロトコル</label>
                                <div className="grid grid-cols-2 gap-2 p-1 bg-slate-100 rounded-2xl">
                                    <button
                                        onClick={() => setProtocol("TCP")}
                                        className={`py-2 rounded-xl font-black text-xs transition-all ${protocol === "TCP" ? "bg-white text-slate-900 shadow-sm" : "text-slate-400"}`}
                                    >
                                        TCP
                                    </button>
                                    <button
                                        onClick={() => setProtocol("UDP")}
                                        className={`py-2 rounded-xl font-black text-xs transition-all ${protocol === "UDP" ? "bg-white text-slate-900 shadow-sm" : "text-slate-400"}`}
                                    >
                                        UDP
                                    </button>
                                </div>
                            </div>

                            <div>
                                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">ポート番号</label>
                                <input
                                    type="number"
                                    value={port}
                                    onChange={e => setPort(e.target.value)}
                                    className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-mono font-bold focus:border-[#16a34a] focus:bg-white outline-none transition-all"
                                    placeholder="25565"
                                    autoFocus
                                    onKeyDown={e => e.key === 'Enter' && handleAdd()}
                                />
                            </div>

                            <button
                                onClick={handleAdd}
                                className="w-full bg-[#16a34a] text-white py-4 rounded-2xl font-black shadow-xl shadow-green-100 hover:bg-[#15803d] active:scale-95 transition-all border-b-4 border-green-800 active:border-b-0 flex items-center justify-center gap-2"
                            >
                                <Plus size={18} /> 追加する
                            </button>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};
