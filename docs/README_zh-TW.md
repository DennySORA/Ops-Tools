# DevOps 工具集

基於 Rust 的高效能 CLI 工具集，專為 DevOps 工作流程設計。

[English](../README.md) | [简体中文](README_zh-CN.md) | [日本語](README_ja.md)

![Screenshot](image/01.png)

## 功能總覽

| 功能 | 說明 |
|------|------|
| Terraform 清理 | 移除 `.terraform`、`.terragrunt-cache` 及 lock 檔案 |
| AI 助手升級 | 批次更新 Claude Code、Codex、Gemini CLI |
| 套件管理 | 安裝/更新 nvm、pnpm、Rust、Go、kubectl、k9s、tmux、vim 等 |
| MCP 管理 | 管理 Claude/Codex/Gemini 的 MCP 伺服器 |
| 安全掃描 | 執行 gitleaks、trufflehog、git-secrets、trivy、semgrep |
| Prompt 生成 | 4 步驟 LLM 工作流程，支援進度追蹤 |
| 技能安裝器 | 安裝 AI CLI 擴充套件（Claude/Codex/Gemini） |
| Rust 編譯 | 跨平台可執行檔建置（cargo/cross，glibc/musl 可選） |
| 容器建構 | Docker/Buildah 多架構建構（x86、arm64、armv7、Jetson） |
| Kubeconfig 管理 | tmux 視窗隔離的 kubeconfig |

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

### 3. 套件安裝/更新（macOS/Linux）
透過互動勾選安裝、移除與更新常用工具：
- `nvm`（安裝最新 Node.js）
- `pnpm`
- `Rust`（透過 rustup）
- `Go`（下載最新版官方壓縮包/pkg）
- `Terraform`
- `kubectl`
- `kubectx`
- `k9s`
- `git`
- `uv`（安裝最新 Python）
- `tmux`（包含 TPM 與 tmux.conf 設定）
- `vim`（包含 vim-plug 與 molokai 設定）
- `ffmpeg`（Linux 使用建置腳本，macOS 使用 Homebrew）

### 4. MCP 工具管理
管理 Claude、Codex 和 Gemini CLI 的 MCP 伺服器：

| MCP 工具 | 說明 |
|----------|------|
| `sequential-thinking` | 循序思考 |
| `context7` | 文檔查詢 |
| `chrome-devtools` | 瀏覽器開發工具 |
| `kubernetes` | K8s 管理 |
| `tailwindcss` | Tailwind CSS（CSS 框架）|
| `arxiv-mcp-server` | arXiv 學術論文搜尋與下載 |
| `github` | GitHub 整合 |
| `cloudflare-*` | Cloudflare MCP（Docs/Workers/Observability 等） |

**環境變數**（可選 - 僅 MCP 管理功能需要，編譯時設定於 `.env`）：
- `GITHUB_PERSONAL_ACCESS_TOKEN`：GitHub 個人存取權杖（必要）
- `GITHUB_MCP_MODE`：`docker`（預設，不需要 Copilot 訂閱）或 `remote`（需要 GitHub Copilot 訂閱）
- `GITHUB_HOST`：GitHub Enterprise 主機（可選，預設 `github.com`）
- `GITHUB_TOOLSETS`：功能集（可選，如 `repos,issues,pull_requests,actions`）
- `CONTEXT7_API_KEY`
- `enable_cloudflare_mcp`（設為 `true` 啟用 Cloudflare MCP）
- `ARXIV_STORAGE_PATH`（arXiv 論文儲存路徑，預設 `~/.arxiv-papers`）

對 Codex MCP 安裝，`CONTEXT7_API_KEY`、`GITHUB_PERSONAL_ACCESS_TOKEN`、`GITHUB_HOST` 的編譯期值會寫入 `~/.codex/config.toml`，執行時不需環境變數。
Cloudflare MCP 會透過 OAuth 互動登入，安裝時請依 CLI 顯示的 URL 完成授權；WSL 可用 `wslview <URL>` 開啟。

### 5. 專案安全掃描
快速安裝並以嚴格模式掃描目前的 Git 專案：
- `gitleaks`（Git 歷史 + 工作樹）
- `trufflehog`（Git 歷史 + 工作樹）
- `git-secrets`（Git 歷史 + 工作樹）
- `trivy`（工作樹 SCA + Misconfig）
- `semgrep`（工作樹 SAST）

自動安裝會先嘗試常見套件管理、Trivy 安裝腳本，以及 Semgrep 的 pipx/venv，若找不到套件則改用 GitHub Release（需 `curl`/`wget` 與 `tar`/`unzip`）。
工作樹掃描僅包含 Git 已追蹤且未被 `.gitignore` 排除的檔案，並會輸出每次掃描的原始 log。

### 6. 提示生成器（LLM）
為 LLM 工作流程生成並執行 4 步驟提示：
- **生成**：從 YAML/JSON 規格檔案建立提示檔案
- **執行**：透過 Claude/Codex/Gemini CLI 交互式執行提示（可選全部或指定功能）
- **狀態**：查看功能執行進度與狀態
- **驗證**：驗證規格檔格式
- **YAML Prompt**：生成 YAML 規格 Prompt（內建模板）

4 步驟工作流程：
1. P1：需求、實作與部署
2. P2：驗證環境 E2E 驗證
3. P3：重構與優化
4. P4：驗證環境 E2E 回歸測試

每個功能都會追蹤進度，支援 session 管理以便中斷後繼續執行。

**Rust Build 提示**
- `*-unknown-linux-gnu`（glibc）：適合主流發行版；動態鏈結，體積小但依賴系統 glibc。
- `*-unknown-linux-musl`（musl，多為靜態）：適合 Alpine/極小映像；單檔部署方便。
- `i686-*` 傳統 32 位元 x86；`powerpc64le-*` OpenPOWER/IBM Cloud；`wasm32-unknown-unknown` 供瀏覽器/wasm 執行環境（無 std）。

### 7. 技能安裝器
安裝與管理 AI CLI 工具的擴充套件：

| CLI | 擴充格式 | 安裝路徑 |
|-----|---------|---------|
| Claude Code | Plugins + Skills | `~/.claude/plugins/`、`~/.claude/skills/` |
| OpenAI Codex | Skills (SKILL.md) | `~/.codex/skills/` |
| Google Gemini | Extensions (TOML) | `~/.gemini/extensions/` |

**可用擴充套件：**
- `ralph-wiggum` - AI 代理迴圈（Claude/Gemini）
- `security-guidance` - 安全最佳實踐（Claude/Gemini）
- `frontend-design` - 前端介面設計（所有 CLI）
- `code-review` - 程式碼審查助手（所有 CLI）
- `pr-review-toolkit` - PR 審查工具（所有 CLI）
- `commit-commands` - Git Commit 助手（所有 CLI）
- `writing-rules` - 寫作風格規則（所有 CLI）

**注意：** Gemini 使用不同的擴充格式。安裝器會自動將 Claude 外掛轉換為 Gemini 原生 TOML 格式，並註冊到 `extension-enablement.json`。

詳見 [docs/SKILL_INSTALLER.md](SKILL_INSTALLER.md) 開發指南。

### 8. 容器映像建構器
使用 Docker 或 Buildah 建構多架構容器映像：
- **建構引擎**：Docker (buildx) 或 Buildah（無背景程序 OCI 建構器）
- **多架構支援**：
  - x86_64 / amd64（Intel/AMD 64 位元）
  - arm64 / aarch64（Apple Silicon、AWS Graviton）
  - armv7 / arm/v7（Raspberry Pi 2/3）
  - Jetson Nano（NVIDIA Jetson Nano aarch64）
- **Dockerfile 掃描**：自動偵測 Dockerfile、Containerfile 及其變體（Dockerfile.dev 等）
- **Registry 推送**：可選擇推送至容器 Registry
- **快速選取**：記住最近使用的映像名稱、標籤和 Registry，方便快速重複使用

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
3. 安裝/更新系統套件（macOS/Linux）
4. 升級 Rust 專案與工具鏈
5. 為多平台建置 Rust 可執行檔（可選 glibc 或 musl，cargo 或 cross）
6. 專案安全掃描（Gitleaks/TruffleHog/Git-Secrets/Trivy/Semgrep）
7. 管理 MCP 工具（Claude/Codex/Gemini）
8. 提示生成器（LLM 4 步驟工作流程）
9. 容器映像建構器（Docker/Buildah 多架構）
10. Kubeconfig 管理（tmux 視窗隔離）
11. 語言設定（英文/繁體中文/簡體中文/日文）

啟動時會先提示選擇語言，之後也可以在選單中切換。
語言偏好會儲存在 `~/.config/ops-tools/config.toml`（Linux）、`~/Library/Application Support/ops-tools/config.toml`（macOS）或 `%APPDATA%\\ops-tools\\config.toml`（Windows）。

## 貢獻

歡迎提交 Pull Request 或建立 Issue！

## 授權

MIT License
