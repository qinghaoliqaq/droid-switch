use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    name: String,
    path: String,
}

fn configs_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".factory/configs")
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
    dirs::home_dir().unwrap().join(".factory/settings.json")
}

fn config_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".factory/config.json")
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
    configs.sort_by(|a, b| a.name.cmp(&b.name));
    configs
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
    Manager,
    tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent},
    menu::{Menu, MenuItem},
    WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_configs, read_config, save_config, create_config,
            delete_config, apply_config, import_current, get_current_config
        ])
        .setup(|app| {
            // Create tray menu
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            
            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
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
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
