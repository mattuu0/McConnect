import { motion, AnimatePresence } from "framer-motion";
import { X } from "lucide-react";
import { Mapping } from "../../types";

/**
 * マップ設定（マッピング）編集・新規作成用モーダルのプロパティ定義
 */
interface MappingModalProps {
    /** モーダルが表示されているかどうか */
    isOpen: boolean;
    /** モーダルのタイトル（例：「新規作成」「編集」） */
    title: string;
    /** 編集中のマッピングデータ（Partialなので一部が欠けていても良い） */
    mapping: Partial<Mapping>;
    /** モーダルを閉じる時のコールバック */
    onClose: () => void;
    /** 保存ボタンが押された時のコールバック */
    onSave: () => void;
    /** 入力値が変更された時のコールバック */
    onChange: (mapping: Partial<Mapping>) => void;
    /** 送信ボタンのラベル（例：「作成」「更新」） */
    submitLabel: string;
}

/**
 * 接続設定の追加や編集を行うためのオーバーレイモーダルコンポーネント
 */
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
                <div className="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4">
                    {/* モーダル本体のコンテナ（フェード＆スケールアニメーション付き） */}
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.95 }}
                        className="bg-white w-full max-w-md rounded-[2.5rem] shadow-2xl p-8 border border-slate-200 relative overflow-hidden"
                    >
                        {/* ヘッダー：タイトルと閉じるボタン */}
                        <div className="flex justify-between items-center mb-8">
                            <h3 className="text-xl font-black text-slate-900 italic tracking-tight uppercase">{title}</h3>
                            <button
                                onClick={onClose}
                                className="p-2 bg-slate-100 rounded-full text-slate-400 hover:text-slate-900 transition-colors"
                            >
                                <X size={20} />
                            </button>
                        </div>

                        {/* 入力フォームエリア */}
                        <div className="space-y-5">
                            {/* 名称設定 */}
                            <div>
                                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">識別名称</label>
                                <input
                                    type="text"
                                    value={mapping.name || ""}
                                    onChange={event => onChange({ ...mapping, name: event.target.value })}
                                    className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-bold focus:border-[#16a34a] focus:bg-white outline-none transition-all text-slate-900"
                                    placeholder="例: サバイバルサーバー"
                                />
                            </div>

                            {/* WebSocket URLの設定（プロキシサーバーの接続先） */}
                            <div>
                                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">WebSocket URL</label>
                                <input
                                    type="text"
                                    value={mapping.wsUrl || ""}
                                    onChange={event => onChange({ ...mapping, wsUrl: event.target.value })}
                                    className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-bold focus:border-[#16a34a] focus:bg-white outline-none transition-all font-mono text-sm"
                                    placeholder="ws://example.com/ws"
                                />
                            </div>

                            {/* プロトコルと外部ポートの設定 */}
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">プロトコル</label>
                                    <select
                                        value={mapping.protocol || "TCP"}
                                        onChange={event => onChange({ ...mapping, protocol: event.target.value as "TCP" | "UDP" })}
                                        className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-black outline-none cursor-pointer appearance-none"
                                        style={{ backgroundImage: 'url("data:image/svg+xml,%3Csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 24 24\' stroke=\'%2316a34a\'%3E%3Cpath stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'3\' d=\'M19 9l-7 7-7-7\'%3E%3C/path%3E%3C/svg%3E")', backgroundPosition: 'right 1rem center', backgroundSize: '1.2em', backgroundRepeat: 'no-repeat' }}
                                    >
                                        <option>TCP</option>
                                        <option>UDP</option>
                                    </select>
                                </div>
                                <div>
                                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">外部ポート</label>
                                    <input
                                        type="number"
                                        value={mapping.remotePort || ""}
                                        onChange={event => onChange({ ...mapping, remotePort: Number(event.target.value) })}
                                        className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-mono font-black outline-none"
                                        placeholder="25565"
                                    />
                                </div>
                            </div>

                            {/* ローカル接続先の設定 */}
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">バインドアドレス</label>
                                    <input
                                        type="text"
                                        value={mapping.bindAddr || ""}
                                        onChange={event => onChange({ ...mapping, bindAddr: event.target.value })}
                                        className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-mono font-bold outline-none"
                                    />
                                </div>
                                <div>
                                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">ローカルポート</label>
                                    <input
                                        type="number"
                                        value={mapping.localPort || ""}
                                        onChange={event => onChange({ ...mapping, localPort: Number(event.target.value) })}
                                        className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-mono font-bold outline-none"
                                    />
                                </div>
                            </div>

                            {/* 公開鍵の設定 */}
                            <div>
                                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest block mb-2 px-1">プロキシ公開鍵 (Base64)</label>
                                <textarea
                                    value={mapping.publicKey || ""}
                                    onChange={event => onChange({ ...mapping, publicKey: event.target.value })}
                                    className="w-full bg-slate-50 border-2 border-slate-100 p-4 rounded-2xl font-mono text-[10px] h-24 focus:border-[#16a34a] focus:bg-white outline-none transition-all resize-none"
                                    placeholder="Base64 encoded public key..."
                                />
                            </div>

                            {/* フッター：アクションボタン */}
                            <div className="pt-4 flex flex-col sm:flex-row gap-3">
                                <button
                                    onClick={onClose}
                                    className="w-full sm:flex-1 bg-slate-100 py-4 font-black text-slate-500 rounded-2xl active:scale-95 transition-all"
                                >
                                    キャンセル
                                </button>
                                <button
                                    onClick={onSave}
                                    className="w-full sm:flex-1 bg-[#16a34a] text-white py-4 rounded-2xl font-black shadow-xl shadow-green-100 hover:bg-[#15803d] active:scale-95 transition-all border-b-4 border-green-800 active:border-b-0 h-[64px]"
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

