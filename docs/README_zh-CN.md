# DevOps 工具集

基于 Rust 的高性能 CLI 工具集，专为 DevOps 工作流程设计。

[English](../README.md) | [繁體中文](README_zh-TW.md) | [日本語](README_ja.md)

![Screenshot](image/01.png)

## 功能总览

| 功能 | 说明 |
|------|------|
| Terraform 清理 | 移除 `.terraform`、`.terragrunt-cache` 及 lock 文件 |
| AI 助手升级 | 批量更新 Claude Code、Codex、Gemini CLI |
| 软件包管理 | 安装/更新 nvm、pnpm、Rust、Go、kubectl、k9s、tmux、vim 等 |
| MCP 管理 | 管理 Claude/Codex/Gemini 的 MCP 服务器 |
| 安全扫描 | 运行 gitleaks、trufflehog、git-secrets、trivy、semgrep |
| Prompt 生成 | 4 步骤 LLM 工作流程，支持进度追踪 |
| 容器构建 | Docker/Buildah 多架构构建（x86、arm64、armv7、Jetson） |
| Kubeconfig 管理 | tmux 窗口隔离的 kubeconfig |

## 功能特色

### 1. Terraform/Terragrunt 缓存清理
智能清理 Terraform 和 Terragrunt 产生的缓存文件：
- `.terragrunt-cache` 目录
- `.terraform` 目录
- `.terraform.lock.hcl` 文件
- 自动过滤重复的子路径，避免重复删除。

### 2. AI 代码助手升级
批量升级热门的 AI 代码助手工具：
- `Claude Code` (@anthropic-ai/claude-code)
- `OpenAI Codex` (@openai/codex)
- `Google Gemini CLI` (@google/gemini-cli)

### 3. 软件包安装/更新（macOS/Linux）
通过交互勾选安装、移除与更新常用工具：
- `nvm`（安装最新 Node.js）
- `pnpm`
- `Rust`（通过 rustup）
- `Go`（下载最新版官方压缩包/pkg）
- `Terraform`
- `kubectl`
- `kubectx`
- `k9s`
- `git`
- `uv`（安装最新 Python）
- `tmux`（包含 TPM 与 tmux.conf 设置）
- `vim`（包含 vim-plug 与 molokai 设置）
- `ffmpeg`（Linux 使用构建脚本，macOS 使用 Homebrew）

### 4. MCP 工具管理
管理 Claude、Codex 和 Gemini CLI 的 MCP 服务器：

| MCP 工具 | 说明 |
|----------|------|
| `sequential-thinking` | 循序思考 |
| `context7` | 文档查询 |
| `chrome-devtools` | 浏览器开发工具 |
| `kubernetes` | K8s 管理 |
| `tailwindcss` | Tailwind CSS（CSS 框架）|
| `arxiv-mcp-server` | arXiv 学术论文搜索与下载 |
| `github` | GitHub 整合 |
| `cloudflare-*` | Cloudflare MCP（Docs/Workers/Observability 等） |

**环境变量**（可选 - 仅 MCP 管理功能需要，编译时设定于 `.env`）：
- `GITHUB_PERSONAL_ACCESS_TOKEN`
- `GITHUB_HOST`
- `CONTEXT7_API_KEY`
- `enable_cloudflare_mcp`（设为 `true` 启用 Cloudflare MCP）
- `ARXIV_STORAGE_PATH`（arXiv 论文存储路径，默认 `~/.arxiv-papers`）

对于 Codex MCP 安装，`CONTEXT7_API_KEY`、`GITHUB_PERSONAL_ACCESS_TOKEN`、`GITHUB_HOST` 的编译期值会写入 `~/.codex/config.toml`，运行时不需环境变量。
Cloudflare MCP 通过 OAuth 交互登录，安装时请根据 CLI 显示的 URL 完成授权；WSL 可用 `wslview <URL>` 打开。

### 5. 项目安全扫描
快速安装并以严格模式扫描当前 Git 项目：
- `gitleaks`（Git 历史 + 工作树）
- `trufflehog`（Git 历史 + 工作树）
- `git-secrets`（Git 历史 + 工作树）
- `trivy`（工作树 SCA + Misconfig）
- `semgrep`（工作树 SAST）

自动安装会先尝试常见包管理器、Trivy 安装脚本，以及 Semgrep 的 pipx/venv，若找不到包则改用 GitHub Release（需要 `curl`/`wget` 和 `tar`/`unzip`）。
工作树扫描仅包含 Git 已追踪且未被 `.gitignore` 排除的文件，并会输出每次扫描的原始 log。

### 6. 提示生成器（LLM）
为 LLM 工作流程生成并执行 4 步骤提示：
- **生成**：从 YAML/JSON 规格文件创建提示文件
- **执行**：通过 Claude/Codex/Gemini CLI 交互式执行提示（可选全部或指定功能）
- **状态**：查看功能执行进度与状态
- **验证**：验证规格文件格式
- **YAML Prompt**：生成 YAML 规格 Prompt（内置模板）

4 步骤工作流程：
1. P1：需求、实现与部署
2. P2：验证环境 E2E 验证
3. P3：重构与优化
4. P4：验证环境 E2E 回归测试

每个功能都会追踪进度，支持 session 管理以便中断后继续执行。

### 7. 容器镜像构建器
使用 Docker 或 Buildah 构建多架构容器镜像：
- **构建引擎**：Docker (buildx) 或 Buildah（无守护进程 OCI 构建器）
- **多架构支持**：
  - x86_64 / amd64（Intel/AMD 64 位）
  - arm64 / aarch64（Apple Silicon、AWS Graviton）
  - armv7 / arm/v7（Raspberry Pi 2/3）
  - Jetson Nano（NVIDIA Jetson Nano aarch64）
- **Dockerfile 扫描**：自动检测 Dockerfile、Containerfile 及其变体（Dockerfile.dev 等）
- **Registry 推送**：可选择推送到容器 Registry
- **快速选取**：记住最近使用的镜像名称、标签和 Registry，方便快速重复使用

## 安装

### 通过安装脚本 (推荐 - Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Ops-Tools/main/install.sh | bash
```

### 手动安装

从 [Releases](https://github.com/DennySORA/Ops-Tools/releases) 页面下载对应你系统的版本。

### 从源码编译

```bash
# 编译
cargo build --release

# 设定环境变量（可选，用于 MCP 管理功能）
cp .env.example .env
# 编辑 .env 填入你的凭证
```

## 使用

如果通过脚本安装，可以直接执行：

```bash
ops-tools
```

如果是从源码编译：

```bash
cargo run
# 或
./target/release/tools
```

选择功能菜单：
1. 清理 Terraform/Terragrunt 缓存文件
2. 升级 AI 代码助手工具
3. 安装/更新系统软件包（macOS/Linux）
4. 升级 Rust 项目与工具链
5. 项目安全扫描（Gitleaks/TruffleHog/Git-Secrets/Trivy/Semgrep）
6. 管理 MCP 工具（Claude/Codex/Gemini）
7. 提示生成器（LLM 4 步骤工作流程）
8. 容器镜像构建器（Docker/Buildah 多架构）
9. 语言设置（英文/繁体中文/简体中文/日文）

启动时会先提示选择语言，之后也可以在菜单中切换。
语言偏好会保存在 `~/.config/ops-tools/config.toml`（Linux）、`~/Library/Application Support/ops-tools/config.toml`（macOS）或 `%APPDATA%\\ops-tools\\config.toml`（Windows）。

## 贡献

欢迎提交 Pull Request 或建立 Issue！

## 授权

MIT License
