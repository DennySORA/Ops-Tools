# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added a Git secret scanner that installs and runs Gitleaks, TruffleHog, and Git-Secrets in strict mode.
- Git worktree scans now only include tracked, non-ignored files and print raw logs per scan.
- Added Cloudflare MCP tools gated by `enable_cloudflare_mcp`.
- Added interactive MCP install mode for tools that require browser-based auth callbacks.
- Added UI language switching with full localization for English, Traditional Chinese, Simplified Chinese, and Japanese.
- Persisted language preference in a standard per-user config directory.
- Added a macOS/Linux package installer/updater with interactive install/remove/update for common dev tools (nvm, pnpm, Rust, Go, Terraform, kubectl, kubectx, k9s, git, uv, tmux, vim, ffmpeg).
- Added a Rust multi-platform builder (cargo/cross) that installs missing targets and builds selected triples.
- Added macOS support to System Updater with auto-detection, Homebrew maintenance, conservative `softwareupdate`, and macOS-specific backup/verification steps.
- Added CUDA Toolkit runfile upgrades to System Updater for NVIDIA/WSL Linux hosts, including official runfile metadata detection and managed `.zshrc` CUDA environment cleanup.

### Changed
- Cloudflare MCP installs now use OAuth interactive login (no API token required).
- MCP package-based installs now use explicit latest selectors for uv and Docker sources.
- Removed the PUA extension from the Skill Installer catalog.
- Prompt generator spec now uses `verification_url` (empty allowed) instead of `int_url`.
- Prompt generator templates now use verification-environment wording and status tokens (`P1_DONE_DEPLOYED`, `P3_REFACTORED_DEPLOYED`) while still accepting legacy INT tokens.
- Renamed Claude Code prompt generator UI to LLM prompt generator in menus and docs.
- Codex upgrades now use `bun install -g @openai/codex`.
- CUDA ML builder now rebuilds selected packages from source, clears stale package artifacts before rebuilds, and reuses locally built torch wheels for dependent builds.
- CUDA ML builder now auto-enables detected build accelerators such as ccache, Ninja, clang, and mold/lld.
- CUDA ML builder install mode now prefers user-site installs without sudo, resolves missing dependencies online, and pins PyTorch runtime installs to the detected CUDA backend.
- CUDA ML builder install mode now installs cached wheel files by local path instead of package name, so cached artifacts are always reused and pip does not replace them from indexes.
- System Updater now uses the DGX OS APT CUDA toolkit path on DGX Spark/GB10 and resolves the configured, installed, or latest APT toolkit package instead of auto-installing the latest NVIDIA runfile.
- Refactored System Updater platform detection around OS-aware capabilities so Linux-only and macOS-only steps are selected cleanly at runtime.

### Fixed
- Improved MCP list parsing to detect Gemini CLI entries with checkmarks and ANSI colors.
- Clarified Codex skill restart and invocation guidance after Skill Installer installs.
- Tailwind CSS MCP installs now use a stdio-safe wrapper so package startup logs do not corrupt MCP handshakes.
- Skill Installer now enables Codex hooks with `[features].hooks` instead of deprecated `[features].codex_hooks`.
- Codex loop-runner installs no longer create stale hook scripts now that the skill uses built-in cron tools.
- Added Gemini MCP settings migration to remove invalid `type` fields and map HTTP URLs.
- Removed invalid `--env` option when adding Context7 MCP via Codex CLI.
- Write Context7 HTTP headers into Codex config when a build-time API key is available.
- Write GitHub MCP env values into Codex config when build-time credentials are available.
- Added GitHub release fallback installs for Gitleaks and TruffleHog when package managers are missing packages.
- Improved security scanner auto-install for Trivy and Semgrep with install script, pipx, and venv fallbacks.

## [0.1.0] - 2025-12-23

### Added
- Initial release of DevOps CLI Toolset.
- **Terraform Cleaner**: Feature to clean `.terragrunt-cache`, `.terraform` directories, and lock files.
- **Tool Upgrader**: Feature to upgrade AI assistants (Claude Code, OpenAI Codex, Gemini CLI).
- **MCP Manager**: Feature to manage MCP servers for Claude and Codex.
- Interactive CLI menu using `dialoguer`.
- Documentation in English, Traditional Chinese, Simplified Chinese, and Japanese.
