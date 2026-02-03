import { useState } from "react";
import { AnimatePresence } from "framer-motion";

// 型定義とカスタムフックのインポート
import { Mapping, View } from "./types";
import { useMappings } from "./hooks/useMappings";
import { useLogs } from "./hooks/useLogs";
import { useServer } from "./hooks/useServer";

// 各コンポーネントのインポート
import { Sidebar } from "./components/Sidebar";
import { MappingModal } from "./components/Modals/MappingModal";
import { Dashboard } from "./pages/Dashboard";
import { Console } from "./pages/Console";
import { About } from "./pages/About";
import { SettingsPage } from "./pages/Settings";
import { ServerPage } from "./pages/Server";

/**
 * アプリケーションのルートコンポーネント
 * 画面遷移（ビュー管理）やモーダルの開閉、マッピング操作の橋渡しを行う
 */
export default function App() {
  // 現在表示している画面（ダッシュボード、コンソール、アバウト）の状態
  const [currentView, setCurrentView] = useState<View>("dashboard");

  // マッピングデータの操作用フック
  const { mappings, startMapping, stopMapping, triggerPing, addMapping, updateMapping, deleteMappings, importConfig } = useMappings();

  // サーバー操作用フック
  const { settings, setSettings, serverConfig, setServerConfig, generateKeys, startServer, stopServer } = useServer();

  // ログデータの操作用フック
  const { logs, logEndRef } = useLogs(currentView);

  // モーダルや削除モードの状態管理
  const [showAddModal, setShowAddModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [isDeleteMode, setIsDeleteMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [editingMapping, setEditingMapping] = useState<Mapping | null>(null);

  // 新規作成時の初期値
  const [newMapping, setNewMapping] = useState<Partial<Mapping>>({
    name: "サバイバルサーバー",
    wsUrl: "ws://localhost:8080/ws",
    bindAddr: "127.0.0.1",
    localPort: 25565,
    remotePort: 25565,
    protocol: "TCP",
    pingInterval: 5
  });

  /**
   * 接続開始/停止の切り替え処理
   * @param event マウスイベント
   * @param mapping 対象のマッピング
   */
  const handleToggleConnect = (event: React.MouseEvent, mapping: Mapping) => {
    event.stopPropagation();
    if (mapping.isRunning) {
      stopMapping(mapping.id);
    } else {
      startMapping(mapping.id);
    }
  };

  /**
   * 編集モードへの切り替え処理
   * @param mapping 編集対象のマッピング
   */
  const handleEdit = (mapping: Mapping) => {
    // 実行中の場合は編集不可（設定変更が反映されないため）
    if (mapping.isRunning) {
      alert("実行中のマッピングは編集できません。先に停止してください。");
      return;
    }
    setEditingMapping({ ...mapping });
    setShowEditModal(true);
  };

  /**
   * 選択されたマッピングの一括削除処理
   */
  const handleDeleteSelected = () => {
    // 実行中のマッピングが含まれている場合は削除不可
    if (mappings.filter(mapping => selectedIds.includes(mapping.id) && mapping.isRunning).length > 0) {
      alert("実行中のマッピングは削除できません。先に停止してください。");
      return;
    }
    deleteMappings(selectedIds);
    setSelectedIds([]);
    setIsDeleteMode(false);
  };

  /**
   * 削除モード時の選択切り替え処理
   * @param id 対象のID
   */
  const handleToggleSelect = (id: string) => {
    setSelectedIds(prevIds =>
      prevIds.includes(id) ? prevIds.filter(prevId => prevId !== id) : [...prevIds, id]
    );
  };

  return (
    <div
      className="flex flex-col sidebar:flex-row h-screen bg-[#f8fafc] text-[#1e293b] font-sans overflow-hidden select-none"
      style={{ fontFamily: '"BIZ UDPGothic", sans-serif' }}
    >
      {/* 共通サイドバー */}
      <Sidebar
        currentView={currentView}
        setCurrentView={setCurrentView}
        settings={settings}
      />

      {/* メインビューエリア */}
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
                onImportConfig={importConfig}
              />
            }
            {currentView === "server" && settings.serverModeEnabled &&
              <ServerPage
                key="server"
                config={serverConfig}
                onConfigChange={setServerConfig}
                onStart={startServer}
                onStop={stopServer}
                onGenerateKeys={generateKeys}
              />
            }
            {currentView === "console" &&
              <Console key="console" logs={logs} logEndRef={logEndRef} />
            }
            {currentView === "settings" &&
              <SettingsPage
                key="settings"
                settings={settings}
                onSettingsChange={setSettings}
              />
            }
            {currentView === "about" &&
              <About key="about" />
            }
          </AnimatePresence>
        </div>
      </main>

      {/* 各種モーダル */}
      {/* 新規作成モーダル */}
      <MappingModal
        isOpen={showAddModal}
        title="新規トンネル作成"
        mapping={newMapping}
        onClose={() => setShowAddModal(false)}
        onSave={() => { addMapping(newMapping); setShowAddModal(false); }}
        onChange={setNewMapping}
        submitLabel="設定を保存する"
      />

      {/* 編集モーダル */}
      <MappingModal
        isOpen={showEditModal}
        title="設定を編集"
        mapping={editingMapping || ({} as any)}
        onClose={() => setShowEditModal(false)}
        onSave={() => { if (editingMapping) updateMapping(editingMapping); setShowEditModal(false); }}
        onChange={(mapping) => setEditingMapping(mapping as Mapping)}
        submitLabel="設定を保存する"
      />
    </div>
  );
}

