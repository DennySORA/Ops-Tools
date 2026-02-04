# Tools - DevOps CLI Toolset

A Rust-based CLI toolset designed for DevOps tasks, adhering to SOLID principles and Clean Code practices.

## Project Overview

This project provides a suite of tools to assist with development and operations workflows:
*   **Terraform/Terragrunt Cache Cleaner:** Intelligently cleans `.terragrunt-cache`, `.terraform` directories, and lock files.
*   **AI Tool Upgrader:** Batch upgrades for AI coding assistants like Claude Code, OpenAI Codex, and Google Gemini CLI.
*   **Package Security Scanner:** Scans dependencies for high-risk packages with performant parallel processing.
*   **MCP Manager:** Manages Model Context Protocol (MCP) servers for Claude and Codex.

## Building and Running

### Prerequisites
*   Rust (latest stable)
*   A `.env` file is required for MCP management features (see `.env.example`).

### Commands

*   **Build Release:**
    ```bash
    cargo build --release
    ```

*   **Run (Development):**
    ```bash
    cargo run
    ```

*   **Run (Release):**
    ```bash
    ./target/release/tools
    ```

*   **Run Tests:**
    ```bash
    cargo test
    ```
    *   Specific feature test: `cargo test terraform_cleaner`

*   **Linting (Clippy):**
    ```bash
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    ```

*   **Formatting:**
    ```bash
    cargo fmt --all -- --check
    ```

## Architecture

The project follows a strict layered architecture:

*   **`src/main.rs`**: Entry point and interactive menu loop.
*   **`src/core/`**: Core abstractions, error handling, result types, and traits (`FileScanner`, `FileCleaner`).
*   **`src/features/`**: Independent modules for each tool.
    *   `terraform_cleaner/`: Logic for cleaning Terraform caches.
    *   `tool_upgrader/`: Logic for upgrading CLI tools.
    *   `package_scanner/`: Logic for scanning package dependencies.
    *   `mcp_manager/`: Logic for managing MCP servers.
*   **`src/ui/`**: UI components like console output, progress bars, and prompts.

## Development Conventions

*   **SOLID Principles:**
    *   **SRP:** Each module/struct has a single responsibility.
    *   **OCP:** Features are extended via traits without modifying core logic.
    *   **DIP:** Services depend on abstractions (traits), not concrete implementations.
*   **Clean Code:**
    *   Clear, specific naming.
    *   Small functions.
    *   Comments explain "why", not "what".
*   **Error Handling:**
    *   Unified `OperationError` enum for all error types.
    *   Errors must be traceable and categorized.

## Skill Installer Development（必須參考文件）

**⚠️ 重要：** 開發 Skill Installer 擴充功能前，**必須先閱讀並遵循** [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md)。

該文件包含完整的技術細節，包括：
- Extension 定義格式與所有欄位說明
- Marketplace-based 插件安裝架構
- `${CLAUDE_PLUGIN_ROOT}` 變數轉換機制
- Hooks 轉換流程
- 依賴安裝流程
- CLI 相容性與限制

### Gemini 特有的轉換機制

#### 變數轉換

Gemini 安裝時會自動將 `${CLAUDE_PLUGIN_ROOT}` 轉換為絕對路徑：

```javascript
// Claude 原始格式
const pluginRoot = '${CLAUDE_PLUGIN_ROOT}';

// Gemini 轉換後
const pluginRoot = '/Users/username/.gemini/extensions/plugin-name';
```

#### Hooks 轉換

Gemini 支援 hooks，安裝器會自動：
1. 複製 hooks 目錄結構
2. 轉換所有 `${CLAUDE_PLUGIN_ROOT}` 引用
3. 安裝 Node.js 依賴
4. 註冊至 `extension-enablement.json`

#### Extension 格式

Gemini 使用與 Claude/Codex 不同的格式：

```
~/.gemini/extensions/<name>/
├── gemini-extension.json    # 必要的 manifest
├── GEMINI.md                # Context 檔案
├── hooks/                   # 轉換後的 hooks（如有）
└── commands/
    └── <name>/
        └── invoke.toml      # TOML 格式的命令
```

### 使用擴充功能

```bash
# Gemini CLI 中使用
> /frontend-design:invoke
> /code-review:invoke
```

### 限制說明

| 功能 | Gemini 支援 |
|-----|------------|
| Hooks | ✅ 完整支援（轉換後） |
| Marketplace plugins | ✅ 支援（變數轉換） |
| `${CLAUDE_PLUGIN_ROOT}` | ⚠️ 轉為絕對路徑 |
| `allowed-tools` | ❌ 移除（Gemini 無此功能） |
| Sub-agent orchestration | ❌ Claude 專有 |

詳細說明請參閱 [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md)。
