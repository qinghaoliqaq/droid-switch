# Droid Switch

一个用于快速切换 Factory Droid 自定义模型配置的桌面工具。

## 功能特性

- 🔄 **快速切换** - 一键切换不同的模型配置方案
- 📝 **配置编辑** - 内置编辑器，直接修改配置内容
- 📋 **配置复制** - 快速复制现有配置创建新方案
- 📥 **导入配置** - 从当前 settings.json 导入配置
- 🔄 **自动转换** - 支持多种配置格式自动转换为 Factory 格式

## 截图

![DD Switch](./screenshots/main.png)

## 安装

### 下载安装包

前往 [Releases](https://github.com/qinghaoliqaq/droid-switch/releases) 下载对应平台的安装包：

- **Windows**: `dd-switch_x.x.x_x64-setup.exe` 或 `.msi`
- **macOS (Intel)**: `dd-switch_x.x.x_x64.dmg`
- **macOS (Apple Silicon)**: `dd-switch_x.x.x_aarch64.dmg`

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/qinghaoliqaq/droid-switch.git
cd droid-switch

# 安装依赖
npm install

# 开发模式运行
npm run tauri dev

# 构建安装包
npm run tauri build
```

## 使用说明

### 配置文件位置

配置文件存放在 `~/.factory/configs/` 目录下。

### 配置文件格式

支持两种格式，会自动转换：

**Factory 标准格式（推荐）：**
```json
{
  "customModels": [
    {
      "model": "claude-opus-4-5-20251101",
      "id": "custom:Claude-Opus-4.5-0",
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

### 字段映射

| 简化格式 | Factory 格式 |
|---------|-------------|
| `custom_models` | `customModels` |
| `model_display_name` | `displayName` |
| `base_url` | `baseUrl` |
| `api_key` | `apiKey` |
| `max_tokens` | `maxOutputTokens` |
| `supports_images` | `noImageSupport` (逻辑取反) |

## 技术栈

- [Tauri 2.0](https://tauri.app/) - 跨平台桌面应用框架
- [React 19](https://react.dev/) - 前端框架
- [Rust](https://www.rust-lang.org/) - 后端逻辑
- [TypeScript](https://www.typescriptlang.org/) - 类型安全

## 开发

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev

# 类型检查
npm run build

# 构建发布版本
npm run tauri build
```

## License

MIT
