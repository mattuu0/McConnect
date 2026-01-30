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
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="flex-1 flex flex-col space-y-4 h-full min-h-0 max-w-4xl mx-auto w-full"
        >
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-2 sm:mb-4 px-1">
                <h2 className="text-xl sm:text-2xl font-bold text-[#3C4043]">ダッシュボード</h2>
                <div className="flex items-center gap-2 self-end sm:self-auto">
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
                                    className="flex items-center gap-2 px-4 sm:px-6 py-2 sm:py-2.5 bg-[#EA4335] text-white rounded-xl text-xs sm:text-sm font-bold shadow-lg shadow-red-100 disabled:opacity-50 disabled:shadow-none transition-all hover:bg-[#D93025]"
                                >
                                    <Trash2 className="w-4 h-4" />
                                    <span className="hidden min-[450px]:inline">選択項目を削除 ({selectedIds.length})</span>
                                    <span className="min-[450px]:hidden">削除 ({selectedIds.length})</span>
                                </button>
                                <button
                                    onClick={() => { setIsDeleteMode(false); setSelectedIds([]); }}
                                    className="p-2 sm:p-2.5 bg-white border border-[#DADCE0] rounded-xl text-[#5F6368] hover:bg-[#F1F3F4] transition-all"
                                >
                                    <X className="w-5 h-5" />
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
                                    className="p-2 sm:p-2.5 bg-white border border-[#DADCE0] rounded-xl text-[#5F6368] hover:border-[#EA4335] hover:text-[#EA4335] transition-all group"
                                    title="削除モード"
                                >
                                    <Trash2 className="w-5 h-5 group-hover:animate-pulse" />
                                </button>
                                <button
                                    onClick={() => setShowAddModal(true)}
                                    className="flex items-center gap-2 px-4 sm:px-6 py-2 sm:py-2.5 bg-[#4285F4] text-white rounded-xl text-xs sm:text-sm font-bold shadow-lg shadow-blue-100 hover:bg-[#1A73E8] transition-all"
                                >
                                    <Plus className="w-5 h-5" />
                                    <span className="hidden min-[450px]:inline">マッピングを追加</span>
                                    <span className="min-[450px]:hidden">追加</span>
                                </button>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>
            </div>

            <div className="grid grid-cols-1 gap-4 overflow-y-auto pb-4 no-scrollbar">
                <AnimatePresence>
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
            </div>

            {mappings.length === 0 && !isDeleteMode && (
                <div className="text-center py-20 border-2 border-dashed border-[#DADCE0] rounded-3xl bg-white/50 mx-1">
                    <p className="text-[#9AA0A6] text-sm px-4">マッピングが登録されていません。<br className="hidden sm:block" />「追加」ボタンから接続を作成してください。</p>
                </div>
            )}
        </motion.div>
    );
};
