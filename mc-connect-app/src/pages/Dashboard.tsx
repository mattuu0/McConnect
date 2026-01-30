import { motion, AnimatePresence } from "framer-motion";
import { Plus, Trash2, X } from "lucide-react";
import { Mapping } from "../types";
import { MappingCard } from "../components/MappingCard";

interface DashboardProps {
    mappings: Mapping[];
    isDeleteMode: boolean;
    setIsDeleteMode: (mode: boolean) => void;
    selectedIds: string[];
    setSelectedIds: (ids: string[]) => void;
    setShowAddModal: (show: boolean) => void;
    onToggleConnect: (e: React.MouseEvent, mapping: Mapping) => void;
    onTriggerPing: (id: string) => void;
    onEdit: (mapping: Mapping) => void;
    onDeleteSelected: () => void;
    onToggleSelect: (id: string) => void;
}

export const Dashboard = ({
    mappings,
    isDeleteMode,
    setIsDeleteMode,
    selectedIds,
    setSelectedIds,
    setShowAddModal,
    onToggleConnect,
    onTriggerPing,
    onEdit,
    onDeleteSelected,
    onToggleSelect
}: DashboardProps) => {
    return (
        <motion.div
            key="dashboard"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex-1 flex flex-col w-full"
        >
            <header className="sticky top-0 z-30 w-full bg-white/90 backdrop-blur-md border-b border-slate-200 px-6 sm:px-12 h-20 shrink-0 flex items-center shadow-sm">
                <div className="w-full flex justify-between items-center">
                    <h2 className="text-xl font-black text-slate-800">トンネル管理</h2>
                    <div className="flex items-center gap-2">
                        <AnimatePresence mode="wait">
                            {isDeleteMode ? (
                                <motion.div
                                    key="delete-actions"
                                    initial={{ opacity: 0, scale: 0.9 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    exit={{ opacity: 0, scale: 0.9 }}
                                    className="flex items-center gap-2"
                                >
                                    <button
                                        onClick={onDeleteSelected}
                                        disabled={selectedIds.length === 0}
                                        className="bg-red-500 hover:bg-red-600 text-white px-5 py-2.5 rounded-xl font-black flex items-center space-x-2 transition-all active:scale-95 shadow-lg shadow-red-100 disabled:opacity-50 disabled:shadow-none text-sm border-b-4 border-red-800 active:border-b-0 active:translate-y-1 h-[46px]"
                                    >
                                        <Trash2 size={18} />
                                        <span>削除 ({selectedIds.length})</span>
                                    </button>
                                    <button
                                        onClick={() => { setIsDeleteMode(false); setSelectedIds([]); }}
                                        className="p-2.5 bg-white border-2 border-slate-200 rounded-xl text-slate-500 hover:border-slate-400 transition-all outline-none h-[46px] w-[46px] flex items-center justify-center"
                                    >
                                        <X size={20} />
                                    </button>
                                </motion.div>
                            ) : (
                                <motion.div
                                    key="normal-actions"
                                    initial={{ opacity: 0, scale: 0.9 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    exit={{ opacity: 0, scale: 0.9 }}
                                    className="flex items-center gap-2"
                                >
                                    <button
                                        onClick={() => setIsDeleteMode(true)}
                                        className="p-2.5 bg-white border-2 border-slate-100 text-slate-400 hover:border-slate-300 rounded-xl transition-all outline-none shadow-sm h-[46px] w-[46px] flex items-center justify-center"
                                        title="削除モード"
                                    >
                                        <Trash2 size={20} />
                                    </button>
                                    <button
                                        onClick={() => setShowAddModal(true)}
                                        className="bg-[#16a34a] hover:bg-[#15803d] text-white px-5 py-2.5 rounded-xl font-black flex items-center gap-2 shadow-lg shadow-green-100 active:scale-95 transition-all text-sm border-b-4 border-green-800 active:border-b-0 active:translate-y-1 h-[46px]"
                                    >
                                        <Plus size={18} strokeWidth={3} />
                                        <span>マッピングを追加</span>
                                    </button>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </div>
                </div>
            </header>

            <div className="max-w-5xl mx-auto w-full px-6 py-10 space-y-6 pb-32">
                <AnimatePresence initial={false}>
                    {mappings.map((m) => (
                        <MappingCard
                            key={m.id}
                            mapping={m}
                            isDeleteMode={isDeleteMode}
                            isSelected={selectedIds.includes(m.id)}
                            onSelect={onToggleSelect}
                            onTriggerPing={onTriggerPing}
                            onEdit={onEdit}
                            onToggleConnect={onToggleConnect}
                        />
                    ))}
                </AnimatePresence>
                {mappings.length === 0 && (
                    <div className="text-center py-20 border-2 border-dashed border-slate-200 rounded-[2.5rem] bg-white/50">
                        <p className="text-slate-400 font-black text-lg">マッピングが登録されていません。</p>
                        <p className="text-slate-300 text-sm mt-2">「マッピングを追加」ボタンから開始してください。</p>
                    </div>
                )}
            </div>
        </motion.div>
    );
};
