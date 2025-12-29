use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct AppSettings {
    factory_path: Option<String>,
    #[serde(default)]
    config_order: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { 
            factory_path: None,
            config_order: vec![],
        }
    }
}

static APP_SETTINGS: Mutex<Option<AppSettings>> = Mutex::new(None);

fn app_settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap())
        .join("dd-switch")
        .join("settings.json")
}

fn load_app_settings() -> AppSettings {
    let mut settings = APP_SETTINGS.lock().unwrap();
    if settings.is_none() {
        let path = app_settings_path();
        *settings = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
        } else {
            Some(AppSettings::default())
        };
    }
    settings.clone().unwrap_or_default()
}

fn save_app_settings(new_settings: &AppSettings) -> Result<(), String> {
    let path = app_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(new_settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    let mut settings = APP_SETTINGS.lock().unwrap();
    *settings = Some(new_settings.clone());
    Ok(())
}

fn factory_base_dir() -> PathBuf {
    let settings = load_app_settings();
    if let Some(custom_path) = settings.factory_path {
        if !custom_path.is_empty() {
            return PathBuf::from(custom_path);
        }
    }
    dirs::home_dir().unwrap().join(".factory")
}

fn configs_dir() -> PathBuf {
    factory_base_dir().join("configs")
}

// Check if model object is in correct Factory format
fn is_factory_format(model: &Value) -> bool {
    model.get("id").is_some() && 
    model.get("index").is_some() && 
    model.get("displayName").is_some()
}

// Convert a single model from custom format to Factory format
fn convert_model(model: &Value, index: usize) -> Value {
    // If already in correct format, return as-is
    if is_factory_format(model) {
        return model.clone();
    }
    
    let display_name = model.get("model_display_name")
        .or_else(|| model.get("displayName"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let model_id = model.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let base_url = model.get("base_url")
        .or_else(|| model.get("baseUrl"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let api_key = model.get("api_key")
        .or_else(|| model.get("apiKey"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let provider = model.get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("anthropic")
        .to_string();
    
    let max_tokens = model.get("max_tokens")
        .or_else(|| model.get("maxOutputTokens"))
        .and_then(|v| v.as_i64())
        .unwrap_or(8192);
    
    // supports_images: true means noImageSupport: false
    let no_image_support = model.get("supports_images")
        .and_then(|v| v.as_bool())
        .map(|v| !v)
        .or_else(|| model.get("noImageSupport").and_then(|v| v.as_bool()))
        .unwrap_or(false);
    
    // Generate unique id
    let clean_name = display_name.replace(" ", "-");
    let id = format!("custom:{}-{}", clean_name, index);
    
    json!({
        "model": model_id,
        "id": id,
        "index": index,
        "baseUrl": base_url,
        "apiKey": api_key,
        "displayName": display_name,
        "maxOutputTokens": max_tokens,
        "noImageSupport": no_image_support,
        "provider": provider
    })
}

// Convert models array, handling both custom_models and customModels
fn convert_models(config: &Value) -> Value {
    let models = config.get("customModels")
        .or_else(|| config.get("custom_models"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    
    let converted: Vec<Value> = models.iter()
        .enumerate()
        .map(|(i, m)| convert_model(m, i))
        .collect();
    
    json!(converted)

}

fn settings_path() -> PathBuf {
    factory_base_dir().join("settings.json")
}

fn config_path() -> PathBuf {
    factory_base_dir().join("config.json")
}

fn target_path() -> PathBuf {
    let settings = settings_path();
    if settings.exists() {
        settings
    } else {
        config_path()
    }
}

#[tauri::command]
fn list_configs() -> Vec<ConfigFile> {
    let dir = configs_dir();
    fs::create_dir_all(&dir).ok();
    let mut configs = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                configs.push(ConfigFile {
                    name: path.file_stem().unwrap().to_string_lossy().to_string(),
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }
    
    // Sort by saved order, new configs go to end
    let settings = load_app_settings();
    if !settings.config_order.is_empty() {
        configs.sort_by(|a, b| {
            let pos_a = settings.config_order.iter().position(|x| x == &a.name).unwrap_or(usize::MAX);
            let pos_b = settings.config_order.iter().position(|x| x == &b.name).unwrap_or(usize::MAX);
            pos_a.cmp(&pos_b)
        });
    } else {
        configs.sort_by(|a, b| a.name.cmp(&b.name));
    }
    configs
}

#[tauri::command]
fn save_config_order(order: Vec<String>) -> Result<(), String> {
    let mut settings = load_app_settings();
    settings.config_order = order;
    save_app_settings(&settings)
}

#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[tauri::command]
fn check_droid_installed() -> Option<String> {
    // Try multiple possible paths for droid
    let paths = if cfg!(target_os = "windows") {
        vec![
            "droid".to_string(),
            format!("{}\\AppData\\Local\\Programs\\droid\\droid.exe", std::env::var("USERPROFILE").unwrap_or_default()),
        ]
    } else {
        vec![
            "droid".to_string(),
            format!("{}/bin/droid", std::env::var("HOME").unwrap_or_default()),
            "/usr/local/bin/droid".to_string(),
            format!("{}/.local/bin/droid", std::env::var("HOME").unwrap_or_default()),
        ]
    };
    
    for path in paths {
        let output = std::process::Command::new(&path)
            .arg("--version")
            .output();
        
        if let Ok(out) = output {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
    }
    None
}

#[tauri::command]
async fn install_droid(proxy: Option<String>) -> Result<String, String> {
    let os = std::env::consts::OS;
    
    let mut cmd = if os == "windows" {
        let mut c = std::process::Command::new("powershell");
        c.args(["-Command", "irm https://app.factory.ai/cli/windows | iex"]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", "curl -fsSL https://app.factory.ai/cli | sh"]);
        c
    };
    
    // Set proxy environment variables if provided
    if let Some(ref p) = proxy {
        if !p.is_empty() {
            cmd.env("http_proxy", p);
            cmd.env("https_proxy", p);
            cmd.env("all_proxy", p);
        }
    }
    
    let output = cmd.output();
    
    match output {
        Ok(out) => {
            if out.status.success() {
                // Check if droid is now installed
                if let Some(version) = check_droid_installed() {
                    Ok(format!("安装成功！版本: {}", version))
                } else {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                }
            } else {
                Err(String::from_utf8_lossy(&out.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
fn read_config(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(path: String, content: String) -> Result<(), String> {
    fs::write(&path, &content).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_config(name: String) -> Result<String, String> {
    let path = configs_dir().join(format!("{}.json", name));
    if path.exists() {
        return Err("Config already exists".to_string());
    }
    let template = json!({
        "customModels": []
    });
    let output = serde_json::to_string_pretty(&template).map_err(|e| e.to_string())?;
    fs::write(&path, output).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn delete_config(path: String) -> Result<(), String> {
    fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_config(old_path: String, new_name: String) -> Result<String, String> {
    let old = PathBuf::from(&old_path);
    let new_path = old.parent()
        .ok_or("Invalid path")?
        .join(format!("{}.json", new_name));
    
    if new_path.exists() {
        return Err("Config with this name already exists".to_string());
    }
    
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_app_settings() -> AppSettings {
    load_app_settings()
}

#[tauri::command]
fn set_factory_path(path: String) -> Result<(), String> {
    let mut settings = load_app_settings();
    settings.factory_path = if path.is_empty() { None } else { Some(path) };
    save_app_settings(&settings)
}

#[tauri::command]
fn check_factory_path() -> Result<bool, String> {
    let base = factory_base_dir();
    Ok(base.exists() && (base.join("settings.json").exists() || base.join("config.json").exists()))
}

#[tauri::command]
fn get_default_factory_path() -> String {
    dirs::home_dir()
        .unwrap()
        .join(".factory")
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
fn apply_config(path: String) -> Result<(), String> {
    let new_config: Value = serde_json::from_str(
        &fs::read_to_string(&path).map_err(|e| e.to_string())?
    ).map_err(|e| e.to_string())?;
    
    // Auto-convert models to Factory format
    let new_models = convert_models(&new_config);
    
    let target = target_path();
    if target.exists() {
        let content = fs::read_to_string(&target).map_err(|e| e.to_string())?;
        let mut settings: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        settings["customModels"] = new_models;
        let output = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
        fs::write(&target, output).map_err(|e| e.to_string())?;
    } else {
        let new_settings = json!({
            "customModels": new_models
        });
        let output = serde_json::to_string_pretty(&new_settings).map_err(|e| e.to_string())?;
        fs::write(&target, output).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_current_config() -> Option<String> {
    let current_content = fs::read_to_string(target_path()).ok()?;
    let current: Value = serde_json::from_str(&current_content).ok()?;
    let current_models = current.get("customModels")?.as_array()?;
    
    let dir = configs_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<Value>(&content) {
                        // Convert config models to Factory format for comparison
                        let converted_models = convert_models(&config);
                        if let Some(models) = converted_models.as_array() {
                            if models == current_models {
                                return Some(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[tauri::command]
fn import_current() -> Result<String, String> {
    let content = fs::read_to_string(target_path()).map_err(|e| e.to_string())?;
    let settings: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    let export_config = json!({
        "customModels": settings.get("customModels").cloned().unwrap_or(json!([]))
    });
    
    let name = format!("imported_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    let path = configs_dir().join(format!("{}.json", name));
    let output = serde_json::to_string_pretty(&export_config).map_err(|e| e.to_string())?;
    fs::write(&path, output).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

use tauri::{
    Emitter, Manager,
    tray::TrayIconBuilder,
    menu::{Menu, MenuItem, Submenu, PredefinedMenuItem},
    WindowEvent,
};

fn build_tray_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let configs = list_configs();
    let current = get_current_config();
    
    // Create config submenu items
    let mut config_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    for cfg in &configs {
        let label = if Some(cfg.path.clone()) == current {
            format!("✓ {}", cfg.name)
        } else {
            format!("  {}", cfg.name)
        };
        let item = MenuItem::with_id(app, &format!("config:{}", cfg.path), &label, true, None::<&str>)?;
        config_items.push(item);
    }
    
    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    if config_items.is_empty() {
        Ok(Menu::with_items(app, &[&show_item, &PredefinedMenuItem::separator(app)?, &quit_item])?)
    } else {
        let config_refs: Vec<&MenuItem<tauri::Wry>> = config_items.iter().collect();
        let configs_submenu = Submenu::with_items(app, "切换配置", true, &config_refs.iter().map(|i| *i as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect::<Vec<_>>())?;
        Ok(Menu::with_items(app, &[&configs_submenu, &PredefinedMenuItem::separator(app)?, &show_item, &quit_item])?)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_configs, read_config, save_config, create_config,
            delete_config, apply_config, import_current, get_current_config,
            rename_config, get_app_settings, set_factory_path, check_factory_path,
            get_default_factory_path, save_config_order, get_platform, install_droid, check_droid_installed
        ])
        .setup(|app| {
            // Use app icon for tray
            let icon = app.default_window_icon().cloned().unwrap();
            
            let menu = build_tray_menu(app)?;
            
            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    if id.starts_with("config:") {
                        let path = id.strip_prefix("config:").unwrap().to_string();
                        let _ = apply_config(path.clone());
                        // Rebuild menu to update checkmarks
                        if let Some(tray) = app.tray_by_id("main") {
                            if let Ok(new_menu) = build_tray_menu_runtime(app) {
                                let _ = tray.set_menu(Some(new_menu));
                            }
                        }
                        // Notify frontend to refresh
                        let _ = app.emit("config-changed", path);
                    } else {
                        match id {
                            "show" => {
                                #[cfg(target_os = "macos")]
                                let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    }
                })
                .build(app)?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                let _ = window.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray_menu_runtime(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let configs = list_configs();
    let current = get_current_config();
    
    let mut config_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    for cfg in &configs {
        let label = if Some(cfg.path.clone()) == current {
            format!("✓ {}", cfg.name)
        } else {
            format!("  {}", cfg.name)
        };
        let item = MenuItem::with_id(app, &format!("config:{}", cfg.path), &label, true, None::<&str>)?;
        config_items.push(item);
    }
    
    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    if config_items.is_empty() {
        Ok(Menu::with_items(app, &[&show_item, &PredefinedMenuItem::separator(app)?, &quit_item])?)
    } else {
        let config_refs: Vec<&MenuItem<tauri::Wry>> = config_items.iter().collect();
        let configs_submenu = Submenu::with_items(app, "切换配置", true, &config_refs.iter().map(|i| *i as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect::<Vec<_>>())?;
        Ok(Menu::with_items(app, &[&configs_submenu, &PredefinedMenuItem::separator(app)?, &show_item, &quit_item])?)
    }
}
