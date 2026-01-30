import { motion, AnimatePresence } from "framer-motion";
import { X } from "lucide-react";
import { Mapping } from "../../types";

interface MappingModalProps {
    isOpen: boolean;
    title: string;
    mapping: Partial<Mapping>;
    onClose: () => void;
    onSave: () => void;
    onChange: (mapping: Partial<Mapping>) => void;
    submitLabel: string;
}

export const MappingModal = ({
    isOpen,
    title,
    mapping,
    onClose,
    onSave,
    onChange,
    submitLabel
}: MappingModalProps) => {
    return (
        <AnimatePresence>
            {isOpen && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-[#202124]/40 backdrop-blur-sm"
                    />
                    <motion.div
                        initial={{ opacity: 0, scale: 0.9, y: 20 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.9, y: 20 }}
                        className="relative w-full max-w-lg bg-white rounded-3xl shadow-2xl overflow-hidden"
                    >
                        <div className="p-6 border-b border-[#DADCE0] flex items-center justify-between">
                            <h3 className="text-lg font-bold text-[#3C4043]">{title}</h3>
                            <button onClick={onClose} className="p-2 hover:bg-[#F1F3F4] rounded-full text-[#5F6368]">
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        <div className="p-8 space-y-6">
                            <div className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-xs font-bold text-[#5F6368] ml-1">WebSocket URL</label>
                                    <input
                                        type="text"
                                        value={mapping.wsUrl}
                                        onChange={(e) => onChange({ ...mapping, wsUrl: e.target.value })}
                                        className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all"
                                        placeholder="ws://example.com/ws"
                                    />
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div className="space-y-2">
                                        <label className="text-xs font-bold text-[#5F6368] ml-1">バインドアドレス</label>
                                        <input
                                            type="text"
                                            value={mapping.bindAddr}
                                            onChange={(e) => onChange({ ...mapping, bindAddr: e.target.value })}
                                            className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all font-mono"
                                            placeholder="127.0.0.1"
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <label className="text-xs font-bold text-[#5F6368] ml-1">プロトコル</label>
                                        <div className="relative">
                                            <select
                                                value={mapping.protocol}
                                                onChange={(e) => onChange({ ...mapping, protocol: e.target.value })}
                                                className="w-full bg-[#F1F3F4] rounded-xl px-4 h-[46px] text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all appearance-none cursor-pointer"
                                                style={{ backgroundImage: 'url("data:image/svg+xml,%3Csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 24 24\' stroke=\'%235F6368\'%3E%3Cpath stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'2\' d=\'M19 9l-7 7-7-7\'%3E%3C/path%3E%3C/svg%3E")', backgroundPosition: 'right 1rem center', backgroundSize: '1em', backgroundRepeat: 'no-repeat' }}
                                            >
                                                <option value="TCP">TCP</option>
                                                <option value="UDP">UDP</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div className="space-y-2">
                                        <label className="text-xs font-bold text-[#5F6368] ml-1">ローカルポート</label>
                                        <input
                                            type="number"
                                            value={mapping.localPort}
                                            onChange={(e) => onChange({ ...mapping, localPort: Number(e.target.value) })}
                                            className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all font-bold"
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <label className="text-xs font-bold text-[#5F6368] ml-1">リモートポート</label>
                                        <input
                                            type="number"
                                            value={mapping.remotePort}
                                            onChange={(e) => onChange({ ...mapping, remotePort: Number(e.target.value) })}
                                            className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all font-bold"
                                        />
                                    </div>
                                </div>

                                <div className="space-y-2">
                                    <label className="text-xs font-bold text-[#5F6368] ml-1">Ping 間隔 (秒)</label>
                                    <input
                                        type="number"
                                        value={mapping.pingInterval}
                                        onChange={(e) => onChange({ ...mapping, pingInterval: Number(e.target.value) })}
                                        className="w-full bg-[#F1F3F4] rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-[#4285F4]/20 border border-transparent focus:border-[#4285F4] transition-all font-bold"
                                        min={1}
                                        max={60}
                                    />
                                </div>
                            </div>

                            <div className="pt-4 flex gap-3">
                                <button
                                    onClick={onClose}
                                    className="flex-1 py-3 border border-[#DADCE0] text-[#5F6368] rounded-xl font-bold text-sm hover:bg-[#F8F9FA] transition-all"
                                >
                                    キャンセル
                                </button>
                                <button
                                    onClick={onSave}
                                    className="flex-2 py-3 bg-[#4285F4] text-white rounded-xl font-bold text-sm hover:bg-[#1A73E8] transition-all shadow-md shadow-blue-100"
                                >
                                    {submitLabel}
                                </button>
                            </div>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};
