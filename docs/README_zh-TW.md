# Ops-Tools

基於 Rust 的高效能 CLI 工具集，專為 DevOps 工作流程設計。

[English](../README.md) | [简体中文](README_zh-CN.md) | [日本語](README_ja.md)

![Screenshot](image/01.png)

## 功能總覽

| 分類 | 功能 | 說明 |
|------|------|------|
| 升級 | 系統升級 | 跨平台系統維護（Linux APT / macOS Homebrew + 工具） |
| 升級 | AI 工具升級 | 批次更新 Claude Code、Codex、Gemini CLI |
| 升級 | Rust 升級 | 升級 Rust 工具鏈 + Cargo 工具 |
| 升級 | 套件管理 | 安裝/更新 nvm、pnpm、Rust、Go、kubectl、k9s、tmux 等 |
| 建構 | Rust 編譯 | 跨平台 Rust 可執行檔（cargo/cross，30+ 目標） |
| 建構 | 容器建構 | Docker/Buildah 多架構建構（x86、arm64、armv7、Jetson） |
| 建構 | CUDA ML 建構 | 從原始碼建構 ML 套件（PyTorch、Flash Attention、xFormers） |
| AI | MCP 管理 | 管理 Claude/Codex/Gemini 的 MCP 伺服器 |
| AI | 技能安裝器 | 安裝 AI CLI 擴充套件（Claude/Codex/Gemini） |
| AI | Prompt 生成 | 4 步驟 LLM 工作流程，支援進度追蹤 |
| 基礎設施 | Terraform 清理 | 移除 `.terraform`、`.terragrunt-cache` 及 lock 檔案 |
| 基礎設施 | Kubeconfig 管理 | tmux 視窗隔離的 kubeconfig |
| 安全 | 安全掃描 | 執行 gitleaks、trufflehog、git-secrets、trivy、semgrep |

## 選單結構

互動式選單將功能分為 5 個分類，依使用頻率智慧排序：

```
常用（依使用頻率排序）
  系統升級、AI 工具升級、...

分類
  建構      — Rust 編譯、容器建構、CUDA ML 建構
  AI        — MCP 管理、技能安裝器、Prompt 生成
  升級      — 系統升級、AI 工具升級、Rust 升級、套件管理
  基礎設施  — Terraform 清理、Kubeconfig 管理
  安全      — 安全掃描

設定      — 語言、常用數量、釘選管理
```

釘選的項目會顯示在最上方。常用項目依使用頻率自動排序。

## 功能特色

### 系統升級
具平台感知能力的跨平台系統維護：
- **模式**：完整更新、僅掃描、清理、驗證、備份
- **設定檔**：預設（完整）、安全（不重啟、保守清理）、積極（深度清理）
- **Linux 流程**：APT 升級、NVIDIA/WSL 主機的 CUDA Toolkit runfile 升級、DGX 核心/驅動、Snap/Flatpak/Docker、工具更新（nvm、bun、deno、pipx、conda、pnpm、Rust、uv）、快取清理、驗證、重啟決策
- **macOS 流程**：Homebrew 更新/升級、保守的 `softwareupdate`、工具更新、快取清理、驗證與維護快照
- **CUDA 自動偵測**：從 NVIDIA runfile 索引、`nvidia-smi`、`nvcc`、`dpkg` 自動偵測最新版 runfile、GPU 架構、WSL CUDA 訊號與驅動/核心套件
- **平台偵測**：執行時自動辨識 Linux 與 macOS，並安全跳過不支援步驟
- **設定**：`update.toml` 或 `~/.config/update/config.toml`（參見 `update.example.toml`）
- 支援試運行模式預覽變更

### AI 工具升級
批次升級 AI 程式碼助手工具：
- `Claude Code` (@anthropic-ai/claude-code)
- `OpenAI Codex` (@openai/codex) — 支援從本地 repo 原始碼建構
- `Google Gemini CLI` (@google/gemini-cli)

### 套件管理（macOS / Linux）
透過互動勾選安裝、移除與更新常用工具：
- `nvm`（安裝最新 Node.js）、`pnpm`、`Rust`（透過 rustup）、`Go`（最新官方壓縮包）
- `Terraform`、`kubectl`、`kubectx`、`k9s`、`git`、`uv`（安裝最新 Python）
- `tmux`（包含 TPM + tmux.conf 設定）、`vim`（包含 vim-plug + molokai 設定）
- `ffmpeg`（Linux 使用建置腳本，macOS 使用 Homebrew）

### Rust 升級
升級 Rust 工具鏈與 Cargo 工具：
- 檢查 rustc、cargo、rustup 版本
- 安裝缺少的 Cargo 工具（cargo-edit、cargo-update、cargo-outdated、cargo-audit）
- 6 步驟升級：rustup self-update、rustup update、cargo install-update、cargo upgrade、cargo outdated、cargo audit

### CUDA ML 建構
從原始碼為你的 GPU 建構 CUDA 加速 ML 套件：
- **套件**：PyTorch、TorchVision、TorchAudio、Flash Attention、xFormers、FlashInfer、BitsAndBytes、ExLlamaV2、AutoGPTQ、AutoAWQ、llama-cpp-python、CTranslate2、TensorRT、Transformer Engine、DeepSpeed、vLLM、CuPy、Unsloth
- **模式**：從原始碼建構、從快取安裝、狀態、清理
- 自動偵測 CUDA 版本、GPU 架構及建構優化（ccache、Ninja、clang、mold）
- 隔離建構環境於 `~/.ml-packages/`

### MCP 管理
管理 Claude、Codex 和 Gemini CLI 的 MCP 伺服器：

| MCP 工具 | 說明 |
|----------|------|
| `sequential-thinking` | 循序思考 |
| `context7` | 文檔查詢 |
| `chrome-devtools` | 瀏覽器開發工具 |
| `kubernetes` | K8s 管理 |
| `tailwindcss` | Tailwind CSS |
| `arxiv-mcp-server` | arXiv 論文搜尋與下載 |
| `github` | GitHub 整合 |
| `cloudflare-*` | Cloudflare MCP 伺服器 |

**選用 MCP 憑證**（編譯時透過 `.env` 設定）：
1. `cp .env.example .env`
2. 填入所需的值
3. 使用 `cargo build --release` 編譯

可用選項：
- **Context7**：設定 `CONTEXT7_API_KEY`
- **GitHub**：設定 `GITHUB_PERSONAL_ACCESS_TOKEN`（必要），選用 `GITHUB_MCP_MODE`、`GITHUB_HOST`、`GITHUB_TOOLSETS`
- **Cloudflare**：設定 `enable_cloudflare_mcp=true`（安裝時 OAuth）
- **arXiv**：設定 `ARXIV_STORAGE_PATH`（預設 `~/.arxiv-papers`）

### 技能安裝器
安裝 AI CLI 工具的擴充套件：

| CLI | 擴充格式 | 安裝路徑 |
|-----|---------|---------|
| Claude Code | Plugins + Skills | `~/.claude/plugins/`、`~/.claude/skills/` |
| OpenAI Codex | Skills (SKILL.md) | `~/.codex/skills/` |
| Google Gemini | Extensions (TOML) | `~/.gemini/extensions/` |

可用擴充套件：ralph-wiggum、security-guidance、frontend-design、code-review、pr-review-toolkit、commit-commands、writing-rules、claude-mem、loop-runner 等。

詳見 [docs/SKILL_INSTALLER.md](SKILL_INSTALLER.md) 開發指南。

### LLM Prompt 生成器
為 LLM 工作流程生成並執行 4 步驟提示：
- **指令**：生成、執行、狀態、驗證、YAML Prompt
- **4 步驟流程**：P1（實作與部署）、P2（E2E 驗證）、P3（重構與優化）、P4（回歸測試）
- 進度追蹤，支援中斷後繼續執行

### Rust 編譯器
跨平台建置 Rust 可執行檔：
- **引擎**：cargo（原生）或 cross（容器化交叉編譯）
- **30+ 目標**：x86_64-gnu、x86_64-musl、aarch64、i686、powerpc64le、wasm32 等
- 自動安裝缺少的 rustup 目標

### 容器建構器
建構多架構容器映像：
- **引擎**：Docker (buildx) 或 Buildah（無背景程序）
- **架構**：x86_64、arm64、armv7、Jetson Nano
- 自動偵測 Dockerfile/Containerfile 變體
- Registry 推送，記住常用設定

### Terraform 清理
智能清理 Terraform/Terragrunt 快取：
- `.terragrunt-cache`、`.terraform`、`.terraform.lock.hcl`
- 自動去重避免重複刪除

### Kubeconfig 管理
tmux 視窗隔離的 kubeconfig，安全進行多叢集操作：
- 設定、清除、列表、清除全部
- 防止意外切換到其他叢集

### 安全掃描
安裝並以嚴格模式掃描 Git 專案：
- `gitleaks`、`trufflehog`、`git-secrets`（歷史 + 工作樹）
- `trivy`（SCA + misconfig）、`semgrep`（SAST）
- 內建供應鏈啟發式掃描，遞迴偵測子資料夾內的 npm、Python、Rust 套件檔案
- 標記 npm install scripts、遠端/本機依賴、缺少 lockfile、Python lockfile URL/index 來源、Rust 替代 registry、git/path 依賴、缺少 integrity/checksum 資料
- 自動安裝，掃描 Git 追蹤與未被忽略的未追蹤檔案，並遵守 `.gitignore`

## 安裝

### 安裝腳本（Linux / macOS）

```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Ops-Tools/main/install.sh | bash
```

### 手動下載

從 [Releases](https://github.com/DennySORA/Ops-Tools/releases) 頁面下載預建構的版本：
- Linux x86_64
- macOS x86_64 / arm64 (Apple Silicon)
- Windows x86_64

### 從原始碼編譯

```bash
cargo build --release
./target/release/tools

# 選用：設定 MCP 憑證
cp .env.example .env
# 編輯 .env，然後重新編譯
```

## 多語言支援

支援 4 種語言 — 首次啟動時選擇，可從設定中切換：

- English
- 繁體中文
- 简体中文
- 日本語

語言偏好儲存位置：
- Linux：`~/.config/ops-tools/config.toml`
- macOS：`~/Library/Application Support/ops-tools/config.toml`
- Windows：`%APPDATA%\ops-tools\config.toml`

## 貢獻

歡迎提交 Pull Request 或建立 Issue！

詳見 [CONTRIBUTING.md](../CONTRIBUTING.md) 開發指南。

## 授權

MIT License
