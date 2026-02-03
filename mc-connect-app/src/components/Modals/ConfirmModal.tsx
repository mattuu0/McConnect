import { motion, AnimatePresence } from "framer-motion";
import { AlertCircle, X } from "lucide-react";

interface ConfirmModalProps {
    isOpen: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onClose: () => void;
}

export const ConfirmModal = ({
    isOpen,
    title,
    message,
    confirmLabel = "実行",
    cancelLabel = "キャンセル",
    onConfirm,
    onClose
}: ConfirmModalProps) => {
    return (
        <AnimatePresence>
            {isOpen && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-6">
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={onClose}
                        className="absolute inset-0 bg-slate-900/40 backdrop-blur-sm"
                    />
                    <motion.div
                        initial={{ opacity: 0, scale: 0.9, y: 20 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.9, y: 20 }}
                        className="relative w-full max-w-md bg-white rounded-[2.5rem] shadow-2xl shadow-slate-900/20 p-8 overflow-hidden"
                    >
                        <button
                            onClick={onClose}
                            className="absolute top-6 right-6 p-2 text-slate-300 hover:text-slate-500 rounded-xl transition-colors"
                        >
                            <X size={20} />
                        </button>

                        <div className="flex flex-col items-center text-center space-y-6">
                            <div className="p-5 bg-amber-50 text-amber-500 rounded-3xl">
                                <AlertCircle size={40} strokeWidth={2.5} />
                            </div>

                            <div className="space-y-2">
                                <h3 className="text-xl font-black text-slate-800 tracking-tight">{title}</h3>
                                <p className="text-sm text-slate-400 font-bold leading-relaxed whitespace-pre-wrap">
                                    {message}
                                </p>
                            </div>

                            <div className="flex w-full gap-3 pt-4">
                                <button
                                    onClick={onClose}
                                    className="flex-1 py-4 bg-slate-50 text-slate-500 rounded-2xl font-black hover:bg-slate-100 transition-all text-sm"
                                >
                                    {cancelLabel}
                                </button>
                                <button
                                    onClick={() => {
                                        onConfirm();
                                        onClose();
                                    }}
                                    className="flex-1 py-4 bg-amber-500 hover:bg-amber-600 text-white rounded-2xl font-black shadow-lg shadow-amber-100 transition-all text-sm border-b-4 border-amber-700 active:border-b-0 active:translate-y-1"
                                >
                                    {confirmLabel}
                                </button>
                            </div>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    );
};
