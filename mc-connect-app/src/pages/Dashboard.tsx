import { motion, AnimatePresence } from "framer-motion";
import { Plus, Trash2, X } from "lucide-react";
import { Mapping } from "../types";
import { MappingCard } from "../components/MappingCard";
import { twMerge } from "tailwind-merge";
import { clsx, type ClassValue } from "clsx";

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

interface DashboardProps {
    mappings: Mapping[];
    isDeleteMode: boolean;
    setIsDeleteMode: (mode: boolean) => void;
    selectedIds: string[];
    setSelectedIds: (ids: string[]) => void;
    setShowAddModal: (show: boolean) => void;
    onToggleConnect: (e: React.MouseEvent, mapping: Mapping) => void;
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
            className="flex-1 flex flex-col space-y-4 h-full min-h-0"
        >
            <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-[#3C4043]">ダッシュボード</h2>
                <div className="flex items-center gap-4">
                    <AnimatePresence mode="wait">
                        {isDeleteMode ? (
                            <motion.div
                                key="delete-actions"
                                initial={{ opacity: 0, x: 20 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: 20 }}
                                className="flex items-center gap-2"
                            >
                                <button
                                    onClick={onDeleteSelected}
                                    disabled={selectedIds.length === 0}
                                    className="flex items-center gap-2 px-6 py-2.5 bg-[#EA4335] text-white rounded-xl text-sm font-bold shadow-lg shadow-red-100 disabled:opacity-50 disabled:shadow-none transition-all hover:bg-[#D93025]"
                                >
                                    <Trash2 className="w-4 h-4" />
                                    選択した項目を削除 ({selectedIds.length})
                                </button>
                                <button
                                    onClick={() => { setIsDeleteMode(false); setSelectedIds([]); }}
                                    className="p-2.5 bg-white border border-[#DADCE0] rounded-xl text-[#5F6368] hover:bg-[#F1F3F4] transition-all"
                                >
                                    <X className="w-5 h-5" />
                                </button>
                            </motion.div>
                        ) : (
                            <motion.div
                                key="normal-actions"
                                initial={{ opacity: 0, x: -20 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: -20 }}
                                className="flex items-center gap-2"
                            >
                                <button
                                    onClick={() => setIsDeleteMode(true)}
                                    className="p-2.5 bg-white border border-[#DADCE0] rounded-xl text-[#5F6368] hover:border-[#EA4335] hover:text-[#EA4335] transition-all group"
                                    title="削除モード"
                                >
                                    <Trash2 className="w-5 h-5 group-hover:animate-pulse" />
                                </button>
                                <button
                                    onClick={() => setShowAddModal(true)}
                                    className="flex items-center gap-2 px-6 py-2.5 bg-[#4285F4] text-white rounded-xl text-sm font-bold shadow-lg shadow-blue-100 hover:bg-[#1A73E8] transition-all"
                                >
                                    <Plus className="w-5 h-5" />
                                    マッピングを追加
                                </button>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-1 lg:grid-cols-2 xl:grid-cols-2 gap-4 overflow-y-auto pb-4">
                <AnimatePresence>
                    {mappings.map((m) => (
                        <MappingCard
                            key={m.id}
                            mapping={m}
                            isDeleteMode={isDeleteMode}
                            isSelected={selectedIds.includes(m.id)}
                            onSelect={onToggleSelect}
                            onEdit={onEdit}
                            onToggleConnect={onToggleConnect}
                        />
                    ))}
                </AnimatePresence>
            </div>

            {mappings.length === 0 && !isDeleteMode && (
                <div className="text-center py-20 border-2 border-dashed border-[#DADCE0] rounded-3xl bg-white/50">
                    <p className="text-[#9AA0A6] text-sm">マッピングが登録されていません。<br />「追加」から新しい接続を作成してください。</p>
                </div>
            )}
        </motion.div>
    );
};
