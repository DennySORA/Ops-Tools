# DevOps 工具集

基于 Rust 的高性能 CLI 工具集，专为 DevOps 工作流程设计。

[English](../README.md) | [繁體中文](README_zh-TW.md) | [日本語](README_ja.md)

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

### 3. MCP 工具管理
管理 Claude、Codex 和 Gemini CLI 的 MCP 服务器：

| MCP 工具 | 说明 |
|----------|------|
| `sequential-thinking` | 循序思考 |
| `context7` | 文档查询 |
| `chrome-devtools` | 浏览器开发工具 |
| `kubernetes` | K8s 管理 |
| `github` | GitHub 整合 |

**环境变量**（可选 - 仅 MCP 管理功能需要，编译时设定于 `.env`）：
- `GITHUB_PERSONAL_ACCESS_TOKEN`
- `GITHUB_HOST`
- `CONTEXT7_API_KEY`

对于 Codex MCP 安装，`CONTEXT7_API_KEY`、`GITHUB_PERSONAL_ACCESS_TOKEN`、`GITHUB_HOST` 的编译期值会写入 `~/.codex/config.toml`，运行时不需环境变量。

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
3. 管理 MCP 工具（Claude/Codex/Gemini）

## 贡献

欢迎提交 Pull Request 或建立 Issue！

## 授权

MIT License
