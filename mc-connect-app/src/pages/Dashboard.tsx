import { motion, AnimatePresence } from "framer-motion";
import { Plus, Trash2, X, ArrowUpCircle } from "lucide-react";
import { Mapping } from "../types";
import { MappingCard } from "../components/MappingCard";

/**
 * ダッシュボードプロパティのインターフェース
 */
interface DashboardProps {
    /** 登録されているマッピングデータのリスト */
    mappings: Mapping[];
    /** 削除モードが有効かどうか */
    isDeleteMode: boolean;
    /** 削除モードの状態を切り替える関数 */
    setIsDeleteMode: (mode: boolean) => void;
    /** 現在選択されているマッピングIDのリスト（削除用） */
    selectedIds: string[];
    /** 選択状態を更新する関数 */
    setSelectedIds: (ids: string[]) => void;
    /** 追加用モーダルの表示状態を制御する関数 */
    setShowAddModal: (show: boolean) => void;
    /** 接続状態の切り替えを実行するコールバック */
    onToggleConnect: (event: React.MouseEvent, mapping: Mapping) => void;
    /** 導通確認（PING）を実行するコールバック */
    onTriggerPing: (id: string) => void;
    /** 編集画面を開くコールバック */
    onEdit: (mapping: Mapping) => void;
    /** 選択されたマッピングを一括削除するコールバック */
    onDeleteSelected: () => void;
    /** 単一マッピングの選択状態を反転させるコールバック */
    onToggleSelect: (id: string) => void;
    /** 設定ファイルをインポートする関数 */
    onImportConfig: (configJson: string) => boolean;
}

/**
 * トンネル管理のメイン画面コンポーネント
 */
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
    onToggleSelect,
    onImportConfig
}: DashboardProps) => {
    /**
     * ファイル選択時の処理
     */
    const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        const file = event.target.files?.[0];
        if (!file) return;

        const reader = new FileReader();
        reader.onload = (e) => {
            const content = e.target?.result as string;
            if (onImportConfig(content)) {
                alert("設定をインポートしました。");
            }
        };
        reader.readAsText(file);
        // 同じファイルを再度選択できるように値をリセット
        event.target.value = "";
    };

    return (
        <motion.div
            key="dashboard"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex-1 flex flex-col w-full"
        >
            {/* 上部ヘッダー：タイトルと操作ボタン */}
            <header className="sticky top-0 z-30 w-full bg-white/90 backdrop-blur-md border-b border-slate-200 px-6 sm:px-12 h-20 shrink-0 flex items-center shadow-sm">
                <div className="w-full flex justify-between items-center">
                    <h2 className="text-xl font-black text-slate-800">トンネル管理</h2>
                    <div className="flex items-center gap-2">
                        {/* モードに応じたアクションボタンの切り替えアニメーション */}
                        <AnimatePresence mode="wait">
                            {isDeleteMode ? (
                                <motion.div
                                    key="delete-actions"
                                    initial={{ opacity: 0, scale: 0.9 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    exit={{ opacity: 0, scale: 0.9 }}
                                    className="flex items-center gap-2"
                                >
                                    {/* 削除実行ボタン：選択がある時のみ有効 */}
                                    <button
                                        onClick={onDeleteSelected}
                                        disabled={selectedIds.length === 0}
                                        className="bg-red-500 hover:bg-red-600 text-white px-5 py-2.5 rounded-xl font-black flex items-center space-x-2 transition-all active:scale-95 shadow-lg shadow-red-100 disabled:opacity-50 disabled:shadow-none text-sm border-b-4 border-red-800 active:border-b-0 active:translate-y-1 h-[46px]"
                                    >
                                        <Trash2 size={18} />
                                        <span>削除 ({selectedIds.length})</span>
                                    </button>
                                    {/* キャンセルボタン */}
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
                                    {/* 削除モード切り替えボタン */}
                                    <button
                                        onClick={() => setIsDeleteMode(true)}
                                        className="p-2.5 bg-white border-2 border-slate-100 text-slate-400 hover:border-slate-300 rounded-xl transition-all outline-none shadow-sm h-[46px] w-[46px] flex items-center justify-center"
                                        title="削除モード"
                                    >
                                        <Trash2 size={20} />
                                    </button>
                                    {/* インポートボタン */}
                                    <label className="cursor-pointer bg-slate-100 hover:bg-slate-200 text-slate-600 px-5 py-2.5 rounded-xl font-black flex items-center gap-2 shadow-sm active:scale-95 transition-all text-sm h-[46px] border-b-4 border-slate-300 active:border-b-0 active:translate-y-1">
                                        <ArrowUpCircle size={18} />
                                        <span>インポート</span>
                                        <input
                                            type="file"
                                            accept=".json"
                                            className="hidden"
                                            onChange={handleFileChange}
                                        />
                                    </label>
                                    {/* 新規作成ボタン */}
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

            {/* メインコンテンツ：マッピングカードの一覧 */}
            <div className="max-w-5xl mx-auto w-full px-6 py-10 space-y-6 pb-32">
                <AnimatePresence initial={false}>
                    {mappings.map((mapping) => (
                        <MappingCard
                            key={mapping.id}
                            mapping={mapping}
                            isDeleteMode={isDeleteMode}
                            isSelected={selectedIds.includes(mapping.id)}
                            onSelect={onToggleSelect}
                            onTriggerPing={onTriggerPing}
                            onEdit={onEdit}
                            onToggleConnect={onToggleConnect}
                        />
                    ))}
                </AnimatePresence>

                {/* データが空の場合のプレースホルダー */}
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

