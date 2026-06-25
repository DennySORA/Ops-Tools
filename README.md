# Ops-Tools

A robust Rust-based CLI toolset designed for efficient DevOps workflows.

[繁體中文](docs/README_zh-TW.md) | [简体中文](docs/README_zh-CN.md) | [日本語](docs/README_ja.md)

![Screenshot](docs/image/01.png)

## Quick Overview

| Category | Feature | Description |
|----------|---------|-------------|
| Upgrade | System Updater | Cross-platform system maintenance (Linux APT / macOS Homebrew + tooling) |
| Upgrade | AI Tool Upgrader | Batch update Claude Code, Codex CLI |
| Upgrade | Rust Upgrader | Upgrade Rust toolchain + cargo tools |
| Upgrade | Package Manager | Install/update nvm, pnpm, Rust, Go, kubectl, k9s, tmux, etc. |
| Build | Rust Builder | Cross-platform Rust binaries (cargo/cross, 30+ targets) |
| Build | Container Builder | Docker/Buildah multi-arch builds (x86, arm64, armv7, Jetson) |
| Build | CUDA ML Builder | Source-build ML packages (PyTorch, Flash Attention, xFormers) |
| AI | MCP Manager | Manage MCP servers for Claude/Codex |
| AI | Skill Installer | Install AI CLI extensions (Claude/Codex) |
| Infra | Terraform Cleaner | Remove `.terraform`, `.terragrunt-cache` and lock files |
| Infra | Kubeconfig Manager | tmux window-isolated kubeconfig |
| Security | Security Scanner | Run gitleaks, trufflehog, git-secrets, trivy, semgrep |

## Menu Structure

The interactive menu groups features into 5 categories with smart ordering based on usage frequency:

```
Common (sorted by usage)
  System Updater, AI Tool Upgrader, ...

Categories
  Build       — Rust Builder, Container Builder, CUDA ML Builder
  AI          — MCP Manager, Skill Installer
  Upgrade     — System Updater, AI Tool Upgrader, Rust Upgrader, Package Manager
  Infra       — Terraform Cleaner, Kubeconfig Manager
  Security    — Security Scanner

Settings    — Language, Common actions count, Pin management
```

Pinned items appear at the top. Common actions are auto-sorted by how often you use them.

## Features

### System Updater
Cross-platform system maintenance with platform-aware workflows:
- **Modes**: Full update, Scan only, Cleanup, Verify, Backup
- **Profiles**: Default (full), Safe (no reboot, conservative), Aggressive (deep cleanup)
- **Linux workflow**: APT upgrade, CUDA Toolkit runfile upgrade on NVIDIA/WSL hosts, DGX kernel/driver, Snap/Flatpak/Docker, tool updates (nvm, bun, deno, pipx, conda, pnpm, Rust, uv), cache cleanup, verification, reboot decision
- **macOS workflow**: Homebrew update/upgrade, conservative `softwareupdate`, tool updates, cache cleanup, verification, backup snapshots
- **CUDA auto-detection**: latest NVIDIA runfile metadata, GPU arch, WSL CUDA signals, and driver/kernel packages detected at runtime from NVIDIA's runfile index, `nvidia-smi`, `nvcc`, and `dpkg`
- **Platform detection**: auto-detects Linux vs macOS at runtime and skips unsupported steps cleanly
- **Config**: `update.toml` or `~/.config/update/config.toml` (see `update.example.toml`)
- Dry-run mode for previewing changes

### AI Tool Upgrader
Batch upgrades for AI code assistants:
- `Claude Code` (@anthropic-ai/claude-code)
- `OpenAI Codex` (@openai/codex) — supports source build from local repo

### Package Manager (macOS / Linux)
Install, remove, and update common tools with an interactive checklist:
- `nvm` (installs latest Node.js), `pnpm`, `Rust` (via rustup), `Go` (latest official archive)
- `Terraform`, `kubectl`, `kubectx`, `k9s`, `git`, `uv` (installs latest Python)
- `tmux` (includes TPM + tmux.conf setup), `vim` (includes vim-plug + molokai config)
- `ffmpeg` (build script on Linux, Homebrew on macOS)

### Rust Upgrader
Upgrade Rust toolchain and cargo tools:
- Checks rustc, cargo, rustup versions
- Installs missing cargo tools (cargo-edit, cargo-update, cargo-outdated, cargo-audit)
- 6-step upgrade: rustup self-update, rustup update, cargo install-update, cargo upgrade, cargo outdated, cargo audit

### CUDA ML Builder
Source-build CUDA-accelerated ML packages for your exact GPU:
- **Packages**: PyTorch, TorchVision, TorchAudio, Flash Attention, xFormers, FlashInfer, BitsAndBytes, ExLlamaV2, AutoGPTQ, AutoAWQ, llama-cpp-python, CTranslate2, TensorRT, Transformer Engine, DeepSpeed, vLLM, CuPy, Unsloth
- **Modes**: Build from source, Install from cache, Status, Clean
- Auto-detects CUDA version, GPU architecture, and build optimizations (ccache, Ninja, clang, mold)
- Isolated build venv at `~/.ml-packages/`

### MCP Manager
Manages MCP servers for Claude and Codex CLI:

| MCP Tool | Description |
|----------|-------------|
| `sequential-thinking` | Sequential Thinking |
| `context7` | Documentation Query |
| `chrome-devtools` | Chrome DevTools |
| `playwright` | Playwright browser automation |
| `github` | GitHub Integration |
| `cloudflare-*` | Cloudflare MCP Servers |

**Optional MCP Credentials** (build-time via `.env`):
1. `cp .env.example .env`
2. Fill in the values you need
3. Build with `cargo build --release`

Available options:
- **Context7**: optionally set `CONTEXT7_API_KEY` for higher limits
- **GitHub**: set `GITHUB_PERSONAL_ACCESS_TOKEN` (required), optional `GITHUB_MCP_MODE`, `GITHUB_HOST`, `GITHUB_TOOLSETS`
- **Cloudflare**: set `enable_cloudflare_mcp=true` (OAuth during install)

### Skill Installer
Install extensions for AI CLI tools:

| CLI | Extension Format | Install Path |
|-----|-----------------|--------------|
| Claude Code | Plugins + Skills | `~/.claude/plugins/`, `~/.claude/skills/` |
| OpenAI Codex | Skills (SKILL.md) + Skills CLI | `.agents/skills/`, `~/.codex/skills/` |

Available extensions: frontend-design and claude-mem for Claude Code; frontend-design plus curated frontend/testing skills for OpenAI Codex.

See [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md) for development guide.
### Rust Builder
Build cross-platform Rust binaries:
- **Engines**: cargo (native) or cross (containerized cross-compilation)
- **30+ targets**: x86_64-gnu, x86_64-musl, aarch64, i686, powerpc64le, wasm32, and more
- Auto-installs missing rustup targets

### Container Builder
Build multi-architecture container images:
- **Engines**: Docker (buildx) or Buildah (daemonless)
- **Architectures**: x86_64, arm64, armv7, Jetson Nano
- Auto-detects Dockerfile/Containerfile variants
- Registry push with saved preferences

### Terraform Cleaner
Intelligently cleans Terraform/Terragrunt cache:
- `.terragrunt-cache`, `.terraform`, `.terraform.lock.hcl`
- Deduplicates overlapping paths to avoid redundant deletions

### Kubeconfig Manager
tmux window-isolated kubeconfig for safe parallel cluster work:
- Setup, Cleanup, List, Cleanup All
- Prevents accidental cross-cluster context switching

### Security Scanner
Installs and runs strict security scans against the current Git repo:
- `gitleaks`, `trufflehog`, `git-secrets` (history + working tree)
- `trivy` (SCA + misconfig), `semgrep` (SAST)
- Built-in supply chain heuristics for nested npm, Python, and Rust package files
- Flags npm install scripts, remote/local dependencies, missing lockfiles, Python lockfile URL/index sources, alternate Rust registries, git/path dependencies, and missing integrity/checksum data
- Auto-install via package managers or GitHub releases
- Scans Git tracked plus untracked non-ignored files, respects `.gitignore`

## Installation

### Install Script (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/DennySORA/Ops-Tools/main/install.sh | bash
```

### Manual Download

Download pre-built binaries from the [Releases](https://github.com/DennySORA/Ops-Tools/releases) page:
- Linux x86_64
- macOS x86_64 / arm64 (Apple Silicon)
- Windows x86_64

### Build from Source

```bash
cargo build --release
./target/release/tools

# Optional: configure MCP credentials
cp .env.example .env
# Edit .env, then rebuild
```

## Internationalization

4 languages supported — selected at first launch, changeable from Settings:

- English
- 繁體中文 (Traditional Chinese)
- 简体中文 (Simplified Chinese)
- 日本語 (Japanese)

Language preference is saved to:
- Linux: `~/.config/ops-tools/config.toml`
- macOS: `~/Library/Application Support/ops-tools/config.toml`
- Windows: `%APPDATA%\ops-tools\config.toml`

## Contributing

Contributions are welcome! Please submit a Pull Request or open an Issue.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

MIT License
