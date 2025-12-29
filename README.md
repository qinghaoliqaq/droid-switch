# Droid Switch

[English](#english) | [中文](#中文) | [日本語](#日本語)

---

## English

A desktop tool for quickly switching Factory Droid custom model configurations.

### Features

- 🔄 **Quick Switch** - One-click switching between different model configurations
- 📝 **Config Editor** - Built-in editor for direct configuration editing
- 📋 **Duplicate Config** - Quickly copy existing configurations
- 📥 **Import Config** - Import from current settings.json
- 🔄 **Auto Convert** - Automatically converts various formats to Factory format
- 🖥️ **System Tray** - Runs in system tray, switch configs without opening the app
- 🔃 **Drag & Drop** - Reorder configurations by dragging
- 🤖 **Droid Installer** - One-click Droid CLI installation

### Installation

Download from [Releases](https://github.com/qinghaoliqaq/droid-switch/releases):

- **Windows**: `Droid Switch_x.x.x_x64-setup.exe` or `.msi`
- **macOS (Intel)**: `Droid Switch_x.x.x_x64.dmg`
- **macOS (Apple Silicon)**: `Droid Switch_x.x.x_aarch64.dmg`

### Build from Source

```bash
git clone https://github.com/qinghaoliqaq/droid-switch.git
cd droid-switch
npm install
npm run tauri dev      # Development
npm run tauri build    # Production build
```

### Usage

#### Config Location
Configurations are stored in `~/.factory/configs/`

#### Supported Formats

**Factory Standard Format (Recommended):**
```json
{
  "customModels": [
    {
      "model": "claude-opus-4-5-20251101",
      "id": "custom:Claude-Opus-0",
      "index": 0,
      "baseUrl": "https://api.example.com",
      "apiKey": "your-api-key",
      "displayName": "Claude Opus 4.5",
      "maxOutputTokens": 8192,
      "noImageSupport": false,
      "provider": "anthropic"
    }
  ]
}
```

**Simplified Format (Auto-converted):**
```json
{
  "custom_models": [
    {
      "model_display_name": "Claude Opus 4.5",
      "model": "claude-opus-4-5-20251101",
      "base_url": "https://api.example.com",
      "api_key": "your-api-key",
      "provider": "anthropic",
      "max_tokens": 8192
    }
  ]
}
```

#### Field Mapping

| Simplified | Factory |
|------------|---------|
| `custom_models` | `customModels` |
| `model_display_name` | `displayName` |
| `base_url` | `baseUrl` |
| `api_key` | `apiKey` |
| `max_tokens` | `maxOutputTokens` |
| `supports_images` | `noImageSupport` (inverted) |

---

## 中文

一个用于快速切换 Factory Droid 自定义模型配置的桌面工具。

### 功能特性

- 🔄 **快速切换** - 一键切换不同的模型配置方案
- 📝 **配置编辑** - 内置编辑器，直接修改配置内容
- 📋 **配置复制** - 快速复制现有配置创建新方案
- 📥 **导入配置** - 从当前 settings.json 导入配置
- 🔄 **自动转换** - 支持多种配置格式自动转换为 Factory 格式
- 🖥️ **系统托盘** - 在系统托盘运行，无需打开应用即可切换配置
- 🔃 **拖拽排序** - 通过拖拽重新排列配置顺序
- 🤖 **Droid 安装器** - 一键安装 Droid CLI

### 安装

前往 [Releases](https://github.com/qinghaoliqaq/droid-switch/releases) 下载：

- **Windows**: `Droid Switch_x.x.x_x64-setup.exe` 或 `.msi`
- **macOS (Intel)**: `Droid Switch_x.x.x_x64.dmg`
- **macOS (Apple Silicon)**: `Droid Switch_x.x.x_aarch64.dmg`

### 从源码构建

```bash
git clone https://github.com/qinghaoliqaq/droid-switch.git
cd droid-switch
npm install
npm run tauri dev      # 开发模式
npm run tauri build    # 构建发布版本
```

### 使用说明

#### 配置文件位置
配置文件存放在 `~/.factory/configs/` 目录下。

#### 支持的格式

**Factory 标准格式（推荐）：**
```json
{
  "customModels": [
    {
      "model": "claude-opus-4-5-20251101",
      "id": "custom:Claude-Opus-0",
      "index": 0,
      "baseUrl": "https://api.example.com",
      "apiKey": "your-api-key",
      "displayName": "Claude Opus 4.5",
      "maxOutputTokens": 8192,
      "noImageSupport": false,
      "provider": "anthropic"
    }
  ]
}
```

**简化格式（自动转换）：**
```json
{
  "custom_models": [
    {
      "model_display_name": "Claude Opus 4.5",
      "model": "claude-opus-4-5-20251101",
      "base_url": "https://api.example.com",
      "api_key": "your-api-key",
      "provider": "anthropic",
      "max_tokens": 8192
    }
  ]
}
```

#### 字段映射

| 简化格式 | Factory 格式 |
|---------|-------------|
| `custom_models` | `customModels` |
| `model_display_name` | `displayName` |
| `base_url` | `baseUrl` |
| `api_key` | `apiKey` |
| `max_tokens` | `maxOutputTokens` |
| `supports_images` | `noImageSupport` (逻辑取反) |

---

## 日本語

Factory Droid のカスタムモデル設定を素早く切り替えるためのデスクトップツールです。

### 機能

- 🔄 **クイック切り替え** - ワンクリックで異なるモデル設定を切り替え
- 📝 **設定エディタ** - 内蔵エディタで直接設定を編集
- 📋 **設定の複製** - 既存の設定を素早くコピー
- 📥 **設定のインポート** - 現在の settings.json からインポート
- 🔄 **自動変換** - 様々な形式を Factory 形式に自動変換
- 🖥️ **システムトレイ** - システムトレイで動作、アプリを開かずに設定を切り替え
- 🔃 **ドラッグ＆ドロップ** - ドラッグで設定の順序を変更
- 🤖 **Droid インストーラー** - ワンクリックで Droid CLI をインストール

### インストール

[Releases](https://github.com/qinghaoliqaq/droid-switch/releases) からダウンロード：

- **Windows**: `Droid Switch_x.x.x_x64-setup.exe` または `.msi`
- **macOS (Intel)**: `Droid Switch_x.x.x_x64.dmg`
- **macOS (Apple Silicon)**: `Droid Switch_x.x.x_aarch64.dmg`

### ソースからビルド

```bash
git clone https://github.com/qinghaoliqaq/droid-switch.git
cd droid-switch
npm install
npm run tauri dev      # 開発モード
npm run tauri build    # 本番ビルド
```

### 使い方

#### 設定ファイルの場所
設定ファイルは `~/.factory/configs/` に保存されます。

#### サポートされる形式

**Factory 標準形式（推奨）：**
```json
{
  "customModels": [
    {
      "model": "claude-opus-4-5-20251101",
      "id": "custom:Claude-Opus-0",
      "index": 0,
      "baseUrl": "https://api.example.com",
      "apiKey": "your-api-key",
      "displayName": "Claude Opus 4.5",
      "maxOutputTokens": 8192,
      "noImageSupport": false,
      "provider": "anthropic"
    }
  ]
}
```

**簡略形式（自動変換）：**
```json
{
  "custom_models": [
    {
      "model_display_name": "Claude Opus 4.5",
      "model": "claude-opus-4-5-20251101",
      "base_url": "https://api.example.com",
      "api_key": "your-api-key",
      "provider": "anthropic",
      "max_tokens": 8192
    }
  ]
}
```

#### フィールドマッピング

| 簡略形式 | Factory 形式 |
|---------|-------------|
| `custom_models` | `customModels` |
| `model_display_name` | `displayName` |
| `base_url` | `baseUrl` |
| `api_key` | `apiKey` |
| `max_tokens` | `maxOutputTokens` |
| `supports_images` | `noImageSupport` (反転) |

---

## Tech Stack

- [Tauri 2.0](https://tauri.app/) - Cross-platform desktop framework
- [React 19](https://react.dev/) - Frontend framework
- [Rust](https://www.rust-lang.org/) - Backend logic
- [TypeScript](https://www.typescriptlang.org/) - Type safety

## License

MIT
