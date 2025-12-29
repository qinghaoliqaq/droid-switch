import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface ConfigFile {
  name: string;
  path: string;
}

function App() {
  const [configs, setConfigs] = useState<ConfigFile[]>([]);
  const [selected, setSelected] = useState<ConfigFile | null>(null);
  const [content, setContent] = useState("");
  const [showEditor, setShowEditor] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [status, setStatus] = useState("");
  const [currentConfig, setCurrentConfig] = useState<string | null>(null);

  const loadConfigs = async () => {
    const list = await invoke<ConfigFile[]>("list_configs");
    setConfigs(list);
    const current = await invoke<string | null>("get_current_config");
    setCurrentConfig(current);
  };

  useEffect(() => { loadConfigs(); }, []);

  const showStatus = (msg: string) => {
    setStatus(msg);
    setTimeout(() => setStatus(""), 2000);
  };

  const selectConfig = async (cfg: ConfigFile) => {
    const data = await invoke<string>("read_config", { path: cfg.path });
    setSelected(cfg);
    setContent(data);
    setShowEditor(true);
  };

  const save = async () => {
    if (!selected) return;
    await invoke("save_config", { path: selected.path, content });
    if (currentConfig === selected.path) {
      await invoke("apply_config", { path: selected.path });
    }
    showStatus(`已保存: ${selected.name}`);
    setShowEditor(false);
  };

  const apply = async (cfg: ConfigFile) => {
    await invoke("apply_config", { path: cfg.path });
    setCurrentConfig(cfg.path);
    showStatus(`已启用: ${cfg.name}`);
  };

  const create = async () => {
    if (!newName.trim()) return;
    await invoke<string>("create_config", { name: newName.trim() });
    setNewName("");
    setShowCreate(false);
    loadConfigs();
    showStatus(`已创建: ${newName}`);
  };

  const del = async (cfg: ConfigFile) => {
    await invoke("delete_config", { path: cfg.path });
    if (selected?.path === cfg.path) {
      setSelected(null);
      setShowEditor(false);
    }
    loadConfigs();
    showStatus(`已删除: ${cfg.name}`);
  };

  const importCurrent = async () => {
    await invoke<string>("import_current");
    loadConfigs();
    showStatus("已导入当前配置");
  };

  const duplicate = async (cfg: ConfigFile) => {
    const content = await invoke<string>("read_config", { path: cfg.path });
    const newName = `${cfg.name}-copy`;
    const newPath = await invoke<string>("create_config", { name: newName });
    await invoke("save_config", { path: newPath, content });
    loadConfigs();
    showStatus(`已复制: ${newName}`);
  };

  const getIcon = (name: string) => {
    const n = name.toLowerCase();
    if (n.includes("claude") || n.includes("anthropic")) return "🅒";
    if (n.includes("gpt") || n.includes("openai")) return "⬡";
    if (n.includes("gemini") || n.includes("google")) return "◆";
    if (n.includes("aws") || n.includes("amazon")) return "▣";
    return "◎";
  };

  return (
    <div className="app">
      <header className="header">
        <div className="header-left">
          <span className="logo">DD Switch</span>
          <button className="settings-btn">⚙</button>
        </div>
        <div className="tabs">
          <button className="tab active">全部</button>
        </div>
        <div className="header-right">
          <button className="icon-btn" onClick={importCurrent} title="导入当前配置">↓</button>
          <button className="add-btn" onClick={() => setShowCreate(true)}>+</button>
        </div>
      </header>

      <div className="content">
        {configs.length === 0 ? (
          <div className="empty-state">
            <p>暂无配置，点击右上角 + 创建新配置</p>
            <p>或点击 ↓ 导入当前 settings.json</p>
          </div>
        ) : (
          <div className="config-list">
            {configs.map(cfg => (
              <div
                key={cfg.path}
                className={`config-card ${selected?.path === cfg.path ? "active" : ""} ${currentConfig === cfg.path ? "current" : ""}`}
              >
                <span className="drag-handle">⋮⋮</span>
                <div className="config-icon">{getIcon(cfg.name)}</div>
                <div className="config-info">
                  <div className="config-name">
                    {cfg.name}
                    {currentConfig === cfg.path && <span className="current-tag">当前使用</span>}
                  </div>
                  <div className="config-url">~/.factory/configs/{cfg.name}.json</div>
                </div>
                <div className="config-actions">
                  <button
                    className={`action-btn apply-btn ${currentConfig === cfg.path ? "applied" : ""}`}
                    onClick={() => apply(cfg)}
                    disabled={currentConfig === cfg.path}
                  >
                    {currentConfig === cfg.path ? "✓ 已启用" : "▶ 启用"}
                  </button>
                  <button className="action-icon" onClick={() => selectConfig(cfg)} title="编辑">✎</button>
                  <button className="action-icon" onClick={() => duplicate(cfg)} title="复制">⧉</button>
                  <button className="action-icon danger" onClick={() => del(cfg)} title="删除">🗑</button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {showEditor && selected && (
        <div className="modal-overlay" onClick={() => setShowEditor(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h3>编辑: {selected.name}</h3>
              <button className="close-btn" onClick={() => setShowEditor(false)}>×</button>
            </div>
            <div className="modal-body">
              <textarea value={content} onChange={e => setContent(e.target.value)} spellCheck={false} />
            </div>
            <div className="modal-footer">
              <button className="btn btn-default" onClick={() => setShowEditor(false)}>取消</button>
              <button className="btn btn-primary" onClick={save}>保存</button>
            </div>
          </div>
        </div>
      )}

      {showCreate && (
        <div className="modal-overlay" onClick={() => setShowCreate(false)}>
          <div className="modal create-modal" onClick={e => e.stopPropagation()} style={{maxWidth: 400}}>
            <div className="modal-header">
              <h3>新建配置</h3>
              <button className="close-btn" onClick={() => setShowCreate(false)}>×</button>
            </div>
            <div className="modal-body">
              <input
                value={newName}
                onChange={e => setNewName(e.target.value)}
                placeholder="输入配置名称"
                autoFocus
                onKeyDown={e => e.key === "Enter" && create()}
              />
            </div>
            <div className="modal-footer">
              <button className="btn btn-default" onClick={() => setShowCreate(false)}>取消</button>
              <button className="btn btn-primary" onClick={create}>创建</button>
            </div>
          </div>
        </div>
      )}

      {status && <div className="status-toast">{status}</div>}
    </div>
  );
}

export default App;
