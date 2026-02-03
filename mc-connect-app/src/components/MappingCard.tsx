import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Globe, Play, Square, ArrowUpCircle, ArrowDownCircle, Activity, RefreshCw, Clock } from "lucide-react";
import { Mapping } from "../types";

/**
 * マッピングカードコンポーネントのプロパティ定義
 */
interface MappingCardProps {
    /** 表示対象のマッピングデータ */
    mapping: Mapping;
    /** 削除モードが有効かどうか */
    isDeleteMode: boolean;
    /** このカードが選択されているかどうか */
    isSelected: boolean;
    /** マッピングが選択された時のコールバック */
    onSelect: (id: string) => void;
    /** PING計測をトリガーする時のコールバック */
    onTriggerPing: (id: string) => void;
    /** 編集ボタンが押された時のコールバック */
    onEdit: (mapping: Mapping) => void;
    /** 接続状態の切り替え（開始/停止）が押された時のコールバック */
    onToggleConnect: (e: React.MouseEvent, mapping: Mapping) => void;
}

/**
 * 接続設定（マッピング）をカード形式で表示するコンポーネント
 */
export const MappingCard = ({
    mapping,
    isDeleteMode,
    isSelected,
    onSelect,
    onEdit,
    onToggleConnect
}: MappingCardProps) => {
    // 接続が開始されてからの経過時間を保持するステート
    const [uptime, setUptime] = useState<string>("00:00:00");

    /**
     * 稼働時間を計算し、フォーマットされた文字列（HH:MM:SS）を更新するエフェクト
     */
    useEffect(() => {
        // 実行中でない、または開始時刻が不明な場合はリセット
        if (!mapping.isRunning || !mapping.startedAt) {
            setUptime("00:00:00");
            return;
        }

        const updateUptime = () => {
            const now = Date.now();
            const startTime = mapping.startedAt || now;
            // 経過秒数を計算
            const totalSeconds = Math.floor(Math.max(0, now - startTime) / 1000);

            // 時間、分、秒に変換
            const hours = Math.floor(totalSeconds / 3600).toString().padStart(2, '0');
            const minutes = Math.floor((totalSeconds % 3600) / 60).toString().padStart(2, '0');
            const seconds = (totalSeconds % 60).toString().padStart(2, '0');

            setUptime(`${hours}:${minutes}:${seconds}`);
        };

        // 初回実行と1秒ごとの定期更新
        updateUptime();
        const intervalId = setInterval(updateUptime, 1000);

        // クリーンアップ時にインターバルを解除
        return () => clearInterval(intervalId);
    }, [mapping.isRunning, mapping.startedAt]);

    /**
     * バイト数を読みやすい形式にフォーマットする関数
     * @param bytes フォーマット対象のバイト数
     */
    const formatBytes = (bytes: number) => {
        if (!bytes || bytes === 0) return "0.00 MB";
        const mb = bytes / (1024 * 1024);
        if (mb < 0.1) {
            return (bytes / 1024).toFixed(2) + " KB";
        }
        return mb.toFixed(2) + " MB";
    };

    return (
        <motion.div
            layout
            onClick={() => {
                // 削除モード時は選択切り替え、通常時は編集
                if (isDeleteMode) onSelect(mapping.id);
                else onEdit(mapping);
            }}
            className={`
                bg-white rounded-3xl border transition-all overflow-hidden shadow-[0_4px_20px_rgba(0,0,0,0.03)] select-none cursor-pointer
                ${mapping.isRunning ? 'border-green-400 ring-4 ring-green-50' : 'border-slate-200 hover:border-slate-300'}
                ${isDeleteMode ? 'border-red-400 animate-pulse ring-4 ring-red-50' : ''}
                ${isSelected ? 'bg-green-50/50' : ''}
            `}
        >
            <div className="p-5 md:p-7 space-y-5">
                {/* 上部セクション：ステータスアイコンと基本情報 */}
                <div className="flex flex-wrap items-center justify-between gap-4">
                    <div className="flex items-center gap-5 flex-1 min-w-[240px]">
                        {/* 状態に応じたアイコン表示 */}
                        <div className={`p-4 rounded-2xl shadow-inner shrink-0 ${mapping.isRunning ? 'bg-[#16a34a] text-white' : 'bg-slate-100 text-slate-400'}`}>
                            {mapping.loading ? <RefreshCw size={28} className="animate-spin" /> : <Globe size={28} />}
                        </div>
                        <div className="truncate">
                            <div className="flex items-center gap-2 mb-1.5">
                                {/* マッピング名 */}
                                <h3 className="font-black text-slate-900 text-lg md:text-xl truncate tracking-tight uppercase leading-none">{mapping.name}</h3>
                                {/* 稼働中の場合は経過時間を表示 */}
                                {mapping.isRunning && (
                                    <span className="flex items-center gap-1 px-2 py-0.5 rounded-full bg-green-100 text-[#16a34a] text-[10px] font-black border border-green-200 shrink-0">
                                        <Clock size={10} /> {uptime}
                                    </span>
                                )}
                            </div>
                            {/* 接続設定の詳細情報（プロトコル、ポート等） */}
                            <div className="flex items-center flex-wrap gap-2 text-xs font-bold leading-none">
                                <span className="px-2 py-0.5 rounded-md bg-slate-900 text-white tracking-widest text-[9px] uppercase">{mapping.protocol}</span>
                                <span className="text-slate-400 font-mono">Port: <span className="text-slate-700">{mapping.remotePort}</span></span>
                                <span className="text-slate-300 hidden md:inline">|</span>
                                <span className="text-slate-400 font-mono hidden md:inline">{mapping.bindAddr}:{mapping.localPort}</span>
                            </div>
                        </div>
                    </div>

                    <div className="w-full md:w-auto">
                        {/* 削除モードか通常モードかに応じたアクションボタン */}
                        {isDeleteMode ? (
                            <button
                                onClick={(e) => { e.stopPropagation(); onSelect(mapping.id); }}
                                className="w-full md:w-44 h-14 bg-red-500 hover:bg-red-600 text-white rounded-2xl font-black shadow-lg shadow-red-100 border-b-4 border-red-800 transition-all active:border-b-0 active:translate-y-1"
                            >
                                削除する
                            </button>
                        ) : (
                            <button
                                onClick={(e) => onToggleConnect(e, mapping)}
                                disabled={mapping.loading || (mapping.hasFailed && !mapping.isRunning)}
                                className={`
                                    w-full md:w-44 h-14 rounded-2xl font-black text-sm flex items-center justify-center gap-2 transition-all shadow-lg outline-none border-b-4 h-[56px]
                                    ${mapping.isRunning
                                        ? 'bg-slate-800 hover:bg-slate-900 text-white border-slate-950 shadow-slate-200 active:border-b-0 active:translate-y-1'
                                        : mapping.loading
                                            ? 'bg-slate-100 text-slate-300 border-slate-200 cursor-wait shadow-none translate-y-1 border-b-0'
                                            : mapping.hasFailed
                                                ? 'bg-red-500 text-white border-red-800 active:border-b-0 active:translate-y-1'
                                                : 'bg-[#16a34a] hover:bg-[#15803d] text-white border-green-800 shadow-green-100 active:border-b-0 active:translate-y-1'}
                                `}
                            >
                                {mapping.loading ? (
                                    <span className="animate-pulse">処理中...</span>
                                ) : mapping.isRunning ? (
                                    <><Square size={16} fill="currentColor" className="mr-1" /> 切断する</>
                                ) : mapping.hasFailed ? (
                                    "失敗"
                                ) : (
                                    <><Play size={16} fill="currentColor" className="mr-1" /> 接続開始</>
                                )}
                            </button>
                        )}
                    </div>
                </div>

                {/* 統計情報パネル：実行中のみ表示 */}
                {mapping.isRunning && mapping.stats && (
                    <div className="grid grid-cols-2 md:grid-cols-3 gap-4 bg-slate-50/80 p-5 rounded-2xl border border-slate-100 animate-in slide-in-from-top-2 duration-300">
                        {/* 送信統計 */}
                        <div className="flex flex-col">
                            <div className="flex items-center gap-2 mb-1.5">
                                <ArrowUpCircle size={14} className="text-green-500" />
                                <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.15em]">送信合計</span>
                            </div>
                            <span className="text-base font-black font-mono text-slate-800 leading-none">{formatBytes(mapping.stats.upload_total)}</span>
                        </div>
                        {/* 受信統計 */}
                        <div className="flex flex-col">
                            <div className="flex items-center gap-2 mb-1.5">
                                <ArrowDownCircle size={14} className="text-blue-500" />
                                <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.15em]">受信合計</span>
                            </div>
                            <span className="text-base font-black font-mono text-slate-800 leading-none">{formatBytes(mapping.stats.download_total)}</span>
                        </div>
                        {/* 遅延（PING）統計 */}
                        <div className="flex flex-col col-span-2 md:col-span-1 border-t md:border-t-0 md:border-l border-slate-200 pt-3 md:pt-0 md:pl-6 flex justify-center">
                            <div className="flex items-center gap-2 mb-1.5">
                                <Activity size={14} className="text-amber-500" />
                                <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.15em]">現在のPING</span>
                            </div>
                            <span className="text-base font-black font-mono text-amber-600 leading-none">{mapping.stats.rtt_ms !== undefined ? `${mapping.stats.rtt_ms}ms` : "--"}</span>
                        </div>
                    </div>
                )}
            </div>
        </motion.div>
    );
};

