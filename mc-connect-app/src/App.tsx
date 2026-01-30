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
    wsUrl: "ws://localhost:8080/ws",
    bindAddr: "127.0.0.1",
    localPort: 25565,
    remotePort: 25565,
    protocol: "TCP",
    pingInterval: 5
  });

  const handleToggleConnect = (e: React.MouseEvent, m: Mapping) => {
    e.stopPropagation();
    m.isRunning ? stopMapping(m.id) : startMapping(m.id);
  };

  const handleEdit = (m: Mapping) => {
    if (m.isRunning) return alert("実行中のマッピングは編集できません。先に停止してください。");
    setEditingMapping({ ...m });
    setShowEditModal(true);
  };

  const handleDeleteSelected = () => {
    if (mappings.filter(m => selectedIds.includes(m.id) && m.isRunning).length > 0) {
      return alert("実行中のマッピングは削除できません。先に停止してください。");
    }
    deleteMappings(selectedIds);
    setSelectedIds([]);
    setIsDeleteMode(false);
  };

  return (
    <div className="flex h-screen bg-[#F8F9FA] text-[#202124] font-sans overflow-hidden">
      <Sidebar currentView={currentView} setCurrentView={setCurrentView} mappings={mappings} />

      <main className="flex-1 overflow-y-auto pt-8 p-8">
        <div className="max-w-4xl mx-auto h-full flex flex-col">
          <AnimatePresence mode="wait">
            {currentView === "dashboard" && (
              <Dashboard
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
                onToggleSelect={(id) => setSelectedIds(prev => prev.includes(id) ? prev.filter(i => i !== id) : [...prev, id])}
              />
            )}
            {currentView === "console" && <Console logs={logs} logEndRef={logEndRef} />}
            {currentView === "about" && <About />}
          </AnimatePresence>
        </div>
      </main>

      <MappingModal
        isOpen={showAddModal}
        title="新しいマッピングを追加"
        mapping={newMapping}
        onClose={() => setShowAddModal(false)}
        onSave={() => { addMapping(newMapping); setShowAddModal(false); }}
        onChange={setNewMapping}
        submitLabel="保存して追加"
      />

      <MappingModal
        isOpen={showEditModal}
        title="マッピングを編集"
        mapping={editingMapping || {}}
        onClose={() => setShowEditModal(false)}
        onSave={() => { if (editingMapping) updateMapping(editingMapping); setShowEditModal(false); }}
        onChange={(m) => setEditingMapping(m as Mapping)}
        submitLabel="変更を保存"
      />
    </div>
  );
}
