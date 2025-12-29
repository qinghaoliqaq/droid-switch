import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { restrictToVerticalAxis, restrictToParentElement } from "@dnd-kit/modifiers";
import "./App.css";

interface ConfigFile {
  name: string;
  path: string;
}

interface AppSettings {
  factory_path: string | null;
}

interface SortableItemProps {
  cfg: ConfigFile;
  currentConfig: string | null;
  getIcon: (name: string) => string;
  apply: (cfg: ConfigFile) => void;
  selectConfig: (cfg: ConfigFile) => void;
  duplicate: (cfg: ConfigFile) => void;
  del: (cfg: ConfigFile) => void;
}

function SortableItem({ cfg, currentConfig, getIcon, apply, selectConfig, duplicate, del }: SortableItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
  } = useSortable({ id: cfg.name });

  const style = {
    transform: transform ? `translateY(${transform.y}px)` : undefined,
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`config-card ${currentConfig === cfg.path ? "current" : ""}`}
    >
      <span className="drag-handle" {...attributes} {...listeners}>⋮⋮</span>
      <div className="config-icon">{getIcon(cfg.name)}</div>
      <div className="config-info">
        <div className="config-name">
          {cfg.name}
          {currentConfig === cfg.path && <span className="current-tag">当前使用</span>}
        </div>
        <div className="config-url">{cfg.path}</div>
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
  );
}

function App() {
  const [configs, setConfigs] = useState<ConfigFile[]>([]);
  const [selected, setSelected] = useState<ConfigFile | null>(null);
  const [content, setContent] = useState("");
  const [showEditor, setShowEditor] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [renameName, setRenameName] = useState("");
  const [newName, setNewName] = useState("");
  const [newContent, setNewContent] = useState('{\n  "customModels": []\n}');
  const [status, setStatus] = useState("");
  const [currentConfig, setCurrentConfig] = useState<string | null>(null);
  const [factoryPath, setFactoryPath] = useState("");
  const [defaultPath, setDefaultPath] = useState("");
  const [factoryFound, setFactoryFound] = useState(true);
  const [platform, setPlatform] = useState("");
  const [installing, setInstalling] = useState(false);
  const [droidVersion, setDroidVersion] = useState<string | null>(null);
  const [proxyUrl, setProxyUrl] = useState("");
  const [loading, setLoading] = useState(true);

  const loadConfigs = async () => {
    const list = await invoke<ConfigFile[]>("list_configs");
    setConfigs(list);
    const current = await invoke<string | null>("get_current_config");
    setCurrentConfig(current);
  };

  const loadSettings = async () => {
    const settings = await invoke<AppSettings>("get_app_settings");
    const defPath = await invoke<string>("get_default_factory_path");
    setDefaultPath(defPath);
    setFactoryPath(settings.factory_path || "");
    const found = await invoke<boolean>("check_factory_path");
    setFactoryFound(found);
    const p = await invoke<string>("get_platform");
    setPlatform(p);
  };

  const checkDroid = async () => {
    const version = await invoke<string | null>("check_droid_installed");
    setDroidVersion(version);
  };

  const installDroid = async () => {
    setInstalling(true);
    try {
      const result = await invoke<string>("install_droid", { proxy: proxyUrl || null });
      showStatus(result);
      const version = await invoke<string | null>("check_droid_installed");
      setDroidVersion(version);
    } catch (e) {
      showStatus(`安装失败: ${e}`);
    }
    setInstalling(false);
  };

  useEffect(() => {
    loadSettings().then(() => loadConfigs()).finally(() => setLoading(false));
    checkDroid(); // 异步检测，不阻塞主界面
  }, []);

  const showStatus = (msg: string) => {
    setStatus(msg);
    setTimeout(() => setStatus(""), 2000);
  };

  const selectConfig = async (cfg: ConfigFile) => {
    const data = await invoke<string>("read_config", { path: cfg.path });
    setSelected(cfg);
    setContent(data);
    setRenameName(cfg.name);
    setShowEditor(true);
  };

  const save = async () => {
    if (!selected) return;
    let finalPath = selected.path;
    
    // Handle rename if name changed
    if (renameName.trim() && renameName.trim() !== selected.name) {
      try {
        finalPath = await invoke<string>("rename_config", {
          oldPath: selected.path,
          newName: renameName.trim()
        });
      } catch (e) {
        showStatus(`重命名失败: ${e}`);
        return;
      }
    }
    
    await invoke("save_config", { path: finalPath, content });
    if (currentConfig === selected.path || currentConfig === finalPath) {
      await invoke("apply_config", { path: finalPath });
      setCurrentConfig(finalPath);
    }
    loadConfigs();
    showStatus(`已保存: ${renameName.trim() || selected.name}`);
    setShowEditor(false);
  };

  const apply = async (cfg: ConfigFile) => {
    await invoke("apply_config", { path: cfg.path });
    setCurrentConfig(cfg.path);
    showStatus(`已启用: ${cfg.name}`);
  };

  const create = async () => {
    if (!newName.trim()) return;
    const path = await invoke<string>("create_config", { name: newName.trim() });
    if (newContent.trim()) {
      await invoke("save_config", { path, content: newContent });
    }
    setNewName("");
    setNewContent('{\n  "customModels": []\n}');
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

  const saveSettings = async () => {
    await invoke("set_factory_path", { path: factoryPath });
    await loadSettings();
    await loadConfigs();
    setShowSettings(false);
    showStatus("设置已保存");
  };

  const getIcon = (name: string) => {
    const n = name.toLowerCase();
    if (n.includes("claude") || n.includes("anthropic")) return "🅒";
    if (n.includes("gpt") || n.includes("openai")) return "⬡";
    if (n.includes("gemini") || n.includes("google")) return "◆";
    if (n.includes("aws") || n.includes("amazon")) return "▣";
    return "◎";
  };

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5,
      },
    })
  );

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;
    if (over && active.id !== over.id) {
      const oldIndex = configs.findIndex(c => c.name === active.id);
      const newIndex = configs.findIndex(c => c.name === over.id);
      const newConfigs = arrayMove(configs, oldIndex, newIndex);
      setConfigs(newConfigs);
      await invoke("save_config_order", { order: newConfigs.map(c => c.name) });
    }
  };

  if (loading) {
    return (
      <div className="app loading-screen">
        <div className="loading-spinner"></div>
        <p>加载中...</p>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="header">
        <div className="header-left">
          <span className="logo">DD Switch</span>
          <button className="settings-btn" onClick={() => setShowSettings(true)}>⚙</button>
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
          <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd} modifiers={[restrictToVerticalAxis, restrictToParentElement]}>
            <SortableContext items={configs.map(c => c.name)} strategy={verticalListSortingStrategy}>
              <div className="config-list">
                {configs.map(cfg => (
                  <SortableItem
                    key={cfg.name}
                    cfg={cfg}
                    currentConfig={currentConfig}
                    getIcon={getIcon}
                    apply={apply}
                    selectConfig={selectConfig}
                    duplicate={duplicate}
                    del={del}
                  />
                ))}
              </div>
            </SortableContext>
          </DndContext>
        )}
      </div>

      {showEditor && selected && (
        <div className="modal-overlay" onClick={() => setShowEditor(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h3>编辑配置</h3>
              <button className="close-btn" onClick={() => setShowEditor(false)}>×</button>
            </div>
            <div className="modal-body">
              <div className="editor-name-row">
                <label>配置名称</label>
                <input
                  className="editor-name-input"
                  value={renameName}
                  onChange={e => setRenameName(e.target.value)}
                  placeholder="配置名称"
                />
              </div>
              <label>配置内容</label>
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
          <div className="modal" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h3>新建配置</h3>
              <button className="close-btn" onClick={() => setShowCreate(false)}>×</button>
            </div>
            <div className="modal-body">
              <div className="editor-name-row">
                <label>配置名称</label>
                <input
                  className="editor-name-input"
                  value={newName}
                  onChange={e => setNewName(e.target.value)}
                  placeholder="输入配置名称"
                  autoFocus
                />
              </div>
              <label>配置内容</label>
              <textarea
                value={newContent}
                onChange={e => setNewContent(e.target.value)}
                placeholder='{"customModels": []}'
                spellCheck={false}
              />
            </div>
            <div className="modal-footer">
              <button className="btn btn-default" onClick={() => setShowCreate(false)}>取消</button>
              <button className="btn btn-primary" onClick={create}>创建</button>
            </div>
          </div>
        </div>
      )}

      {showSettings && (
        <div className="modal-overlay" onClick={() => setShowSettings(false)}>
          <div className="modal settings-modal" onClick={e => e.stopPropagation()} style={{maxWidth: 500}}>
            <div className="modal-header">
              <h3>设置</h3>
              <button className="close-btn" onClick={() => setShowSettings(false)}>×</button>
            </div>
            <div className="modal-body">
              <div className="setting-item">
                <label>Factory 目录路径</label>
                <input
                  value={factoryPath}
                  onChange={e => setFactoryPath(e.target.value)}
                  placeholder={defaultPath}
                />
                <div className="setting-hint">
                  留空使用默认路径: {defaultPath}
                </div>
                {!factoryFound && (
                  <div className="setting-warning">
                    ⚠️ 未找到 Factory 配置文件，请确认路径正确
                  </div>
                )}
              </div>

              <div className="setting-item">
                <label>安装 Droid</label>
                {droidVersion ? (
                  <div className="droid-installed">
                    <span className="droid-status">✓ 已安装</span>
                    <span className="droid-version">版本: {droidVersion}</span>
                  </div>
                ) : (
                  <>
                    <input
                      value={proxyUrl}
                      onChange={e => setProxyUrl(e.target.value)}
                      placeholder="代理地址（可选）如: http://127.0.0.1:7890"
                      style={{ marginBottom: 8 }}
                    />
                    <button 
                      className="btn btn-primary install-btn" 
                      onClick={installDroid}
                      disabled={installing}
                    >
                      {installing ? "安装中..." : "一键安装 Droid"}
                    </button>
                    <div className="setting-hint">
                      {platform === "macos" || platform === "darwin" 
                        ? "将执行: curl -fsSL https://app.factory.ai/cli | sh"
                        : "将执行: irm https://app.factory.ai/cli/windows | iex"
                      }
                    </div>
                  </>
                )}
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn btn-default" onClick={() => setShowSettings(false)}>取消</button>
              <button className="btn btn-primary" onClick={saveSettings}>保存</button>
            </div>
          </div>
        </div>
      )}

      {status && <div className="status-toast">{status}</div>}
    </div>
  );
}

export default App;
