# DevOps 工具集

基於 Rust 的高效能 CLI 工具集，專為 DevOps 工作流程設計。

[English](../README.md) | [简体中文](README_zh-CN.md) | [日本語](README_ja.md)

## 功能特色

### 1. Terraform/Terragrunt 快取清理
智能清理 Terraform 和 Terragrunt 產生的快取檔案：
- `.terragrunt-cache` 目錄
- `.terraform` 目錄
- `.terraform.lock.hcl` 檔案
- 自動過濾重複的子路徑，避免重複刪除。

### 2. AI 程式碼助手升級
批次升級熱門的 AI 程式碼助手工具：
- `Claude Code` (@anthropic-ai/claude-code)
- `OpenAI Codex` (@openai/codex)
- `Google Gemini CLI` (@google/gemini-cli)

### 3. MCP 工具管理
管理 Claude、Codex 和 Gemini CLI 的 MCP 伺服器：

| MCP 工具 | 說明 |
|----------|------|
| `sequential-thinking` | 循序思考 |
| `context7` | 文檔查詢 |
| `chrome-devtools` | 瀏覽器開發工具 |
| `kubernetes` | K8s 管理 |
| `github` | GitHub 整合 |
| `cloudflare-*` | Cloudflare MCP（Docs/Workers/Observability 等） |

**環境變數**（可選 - 僅 MCP 管理功能需要，編譯時設定於 `.env`）：
- `GITHUB_PERSONAL_ACCESS_TOKEN`
- `GITHUB_HOST`
- `CONTEXT7_API_KEY`
- `enable_cloudflare_mcp`（設為 `true` 啟用 Cloudflare MCP）

對 Codex MCP 安裝，`CONTEXT7_API_KEY`、`GITHUB_PERSONAL_ACCESS_TOKEN`、`GITHUB_HOST` 的編譯期值會寫入 `~/.codex/config.toml`，執行時不需環境變數。
Cloudflare MCP 會透過 OAuth 互動登入，安裝時請依 CLI 顯示的 URL 完成授權；WSL 可用 `wslview <URL>` 開啟。

### 4. Git 機密掃描
快速安裝並以嚴格模式掃描目前的 Git 專案：
- `gitleaks`（Git 歷史 + 工作樹）
- `trufflehog`（Git 歷史 + 工作樹）
- `git-secrets`（Git 歷史 + 工作樹）

自動安裝會先嘗試常見套件管理與 `go install`，若找不到套件則改用 GitHub Release（需 `curl`/`wget` 與 `tar`/`unzip`）。
工作樹掃描僅包含 Git 已追蹤且未被 `.gitignore` 排除的檔案，並會輸出每次掃描的原始 log。

## 安裝

### 透過安裝腳本 (推薦 - Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Ops-Tools/main/install.sh | bash
```

### 手動安裝

從 [Releases](https://github.com/DennySORA/Ops-Tools/releases) 頁面下載對應你系統的版本。

### 從原始碼編譯

```bash
# 編譯
cargo build --release

# 設定環境變數（可選，用於 MCP 管理功能）
cp .env.example .env
# 編輯 .env 填入你的憑證
```

## 使用

如果透過腳本安裝，可以直接執行：

```bash
ops-tools
```

如果是從原始碼編譯：

```bash
cargo run
# 或
./target/release/tools
```

選擇功能選單：
1. 清理 Terraform/Terragrunt 快取檔案
2. 升級 AI 程式碼助手工具
3. 升級 Rust 專案與工具鏈
4. Git 機密掃描（Gitleaks/TruffleHog/Git-Secrets）
5. 管理 MCP 工具（Claude/Codex/Gemini）
6. 語言設定（英文/繁體中文/簡體中文/日文）

啟動時會先提示選擇語言，之後也可以在選單中切換。
語言偏好會儲存在 `~/.config/ops-tools/config.toml`（Linux）、`~/Library/Application Support/ops-tools/config.toml`（macOS）或 `%APPDATA%\\ops-tools\\config.toml`（Windows）。

## 貢獻

歡迎提交 Pull Request 或建立 Issue！

## 授權

MIT License
