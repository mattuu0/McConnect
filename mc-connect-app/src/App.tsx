import { useState } from "react";
import { AnimatePresence } from "framer-motion";

// Types & Hooks
import { Mapping, View } from "./types";
import { useMappings } from "./hooks/useMappings";
import { useLogs } from "./hooks/useLogs";

// Components
import { Sidebar } from "./components/Sidebar";
import { MappingModal } from "./components/Modals/MappingModal";
import { Dashboard } from "./pages/Dashboard";
import { Console } from "./pages/Console";
import { About } from "./pages/About";

export default function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const { mappings, startMapping, stopMapping, triggerPing, addMapping, updateMapping, deleteMappings } = useMappings();
  const { logs, logEndRef } = useLogs(currentView);

  const [showAddModal, setShowAddModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [isDeleteMode, setIsDeleteMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [editingMapping, setEditingMapping] = useState<Mapping | null>(null);

  const [newMapping, setNewMapping] = useState<Partial<Mapping>>({
    name: "サバイバルサーバー",
    wsUrl: "ws://localhost:8080/ws",
    bindAddr: "127.0.0.1",
    localPort: 25565,
    remotePort: 25565,
    protocol: "TCP",
    pingInterval: 5
  });

  const handleToggleConnect = (e: React.MouseEvent, m: Mapping) => {
    e.stopPropagation();
    if (m.isRunning) {
      stopMapping(m.id);
    } else {
      startMapping(m.id);
    }
  };

  const handleEdit = (m: Mapping) => {
    if (m.isRunning) {
      alert("実行中のマッピングは編集できません。先に停止してください。");
      return;
    }
    setEditingMapping({ ...m });
    setShowEditModal(true);
  };

  const handleDeleteSelected = () => {
    if (mappings.filter(m => selectedIds.includes(m.id) && m.isRunning).length > 0) {
      alert("実行中のマッピングは削除できません。先に停止してください。");
      return;
    }
    deleteMappings(selectedIds);
    setSelectedIds([]);
    setIsDeleteMode(false);
  };

  const handleToggleSelect = (id: string) => {
    setSelectedIds(prev =>
      prev.includes(id) ? prev.filter(i => i !== id) : [...prev, id]
    );
  };

  return (
    <div
      className="flex flex-col sm:flex-row h-screen bg-[#f8fafc] text-[#1e293b] font-sans overflow-hidden select-none"
      style={{ fontFamily: '"BIZ UDPGothic", sans-serif' }}
    >
      <Sidebar currentView={currentView} setCurrentView={setCurrentView} mappings={mappings} />

      <main className="flex-1 flex flex-col overflow-hidden">
        <div className="flex-1 overflow-y-auto no-scrollbar">
          <AnimatePresence mode="wait">
            {currentView === "dashboard" &&
              <Dashboard
                key="dashboard"
                mappings={mappings}
                isDeleteMode={isDeleteMode}
                setIsDeleteMode={setIsDeleteMode}
                selectedIds={selectedIds}
                setSelectedIds={setSelectedIds}
                setShowAddModal={setShowAddModal}
                onToggleConnect={handleToggleConnect}
                onTriggerPing={triggerPing}
                onEdit={handleEdit}
                onDeleteSelected={handleDeleteSelected}
                onToggleSelect={handleToggleSelect}
              />
            }
            {currentView === "console" &&
              <Console key="console" logs={logs} logEndRef={logEndRef} />
            }
            {currentView === "about" &&
              <About key="about" />
            }
          </AnimatePresence>
        </div>
      </main>

      <MappingModal
        isOpen={showAddModal}
        title="新規トンネル作成"
        mapping={newMapping}
        onClose={() => setShowAddModal(false)}
        onSave={() => { addMapping(newMapping); setShowAddModal(false); }}
        onChange={setNewMapping}
        submitLabel="設定を保存する"
      />

      <MappingModal
        isOpen={showEditModal}
        title="設定を編集"
        mapping={editingMapping || ({} as any)}
        onClose={() => setShowEditModal(false)}
        onSave={() => { if (editingMapping) updateMapping(editingMapping); setShowEditModal(false); }}
        onChange={(m) => setEditingMapping(m as Mapping)}
        submitLabel="設定を保存する"
      />
    </div>
  );
}
