# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` entry point and menu; `src/core/` shared errors, results, utils; `src/features/` for tool modules (terraform_cleaner, tool_upgrader, rust_upgrader, mcp_manager); `src/ui/` console output, prompts, progress.
- `docs/` localized README; `build.rs` loads `.env` into compile-time env; `install.sh` for installer; `target/` build output.

## Build, Test, and Development Commands
- `cargo build` build debug; `cargo build --release` optimized.
- `cargo run` run locally; `./target/release/tools` run release binary.
- `cargo test` unit tests; `cargo test terraform_cleaner` run module-specific tests.
- `cargo fmt --all -- --check` formatting; `cargo clippy --workspace --all-targets --all-features -- -D warnings` lint.

## Coding Style & Naming Conventions
- Rust 2021; rustfmt defaults (4-space indentation).
- Naming: `snake_case` for modules/functions/vars, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Prefer small, single-purpose modules and keep core logic in `src/core`/`src/features` with UI concerns in `src/ui`.

## Testing Guidelines
- Unit tests live beside code in `mod tests` blocks under `src/**`.
- Keep tests fast and deterministic; use `tempfile` when touching filesystem.
- No explicit coverage target; add tests for new behavior and bug fixes.

## Commit & Pull Request Guidelines
- Commit messages follow Conventional Commits (e.g., `feat(mcp): ...`, `fix(install): ...`, `refactor(app)!: ...`).
- PRs target `main`, include a clear description and linked issues.
- Update `CHANGELOG.md` under `[Unreleased]` and update `README.md` when behavior changes.

## Configuration & Security
- For MCP features, copy `.env.example` to `.env` and set build-time variables (e.g., `GITHUB_PERSONAL_ACCESS_TOKEN`, `GITHUB_HOST`, `CONTEXT7_API_KEY`); `.env` should not be committed.

## Agent Notes (Optional)
- See `CLAUDE.md` and `GEMINI.md` for AI-specific development guidance.

## Skill Installer Development（必須參考文件）

**⚠️ 重要：** 開發 Skill Installer 擴充功能前，**必須先閱讀並遵循** [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md)。

該文件是 Skill Installer 開發的完整參考，包含：
- Extension 定義格式與所有欄位說明
- Marketplace-based 插件安裝架構（git clone、symlink、JSON registries）
- `${CLAUDE_PLUGIN_ROOT}` 變數轉換機制（Claude → Gemini）
- Hooks 轉換流程
- npm/bun 依賴安裝流程
- JSON registries（known_marketplaces.json、installed_plugins.json）
- CLI 相容性矩陣與完整限制說明

### CLI 相容性速查表

| Plugin Structure | Claude | Codex | Gemini | Configuration |
|-----------------|--------|-------|--------|---------------|
| Has `skills/` subdirectory | Plugin | Skill (extract) | Extension (TOML) | `skill_subpath` |
| Has `commands/` only | Plugin | Skill (convert) | Extension (TOML) | `command_file` |
| Has `hooks/` only | Plugin | ❌ Not supported | Extension (TOML) | `has_hooks: true` |
| Has `hooks/` + `commands/` | Plugin | Skill (convert) | Extension (TOML) | `has_hooks: true` |
| Requires marketplace root | Plugin (marketplace) | ❌ Not supported | Extension (variable conversion) | `marketplace_name` |

### Marketplace 插件

第三方插件如果有 scripts 引用 marketplace root directory（例如 `smart-install.js` 尋找父層的 `package.json`），需要設定：

```rust
Extension {
    name: "plugin-name",
    marketplace_name: Some("marketplace-id"),      // Marketplace 識別碼
    marketplace_plugin_path: Some("plugin"),       // Repo 內的插件路徑
    version: Some("1.0.0"),                        // 版本號
}
```

### 必要步驟

1. **閱讀文件**：先閱讀 `docs/SKILL_INSTALLER.md` 了解完整架構
2. **Extension 定義**：在 `src/features/skill_installer/tools.rs` 新增
3. **i18n 支援**：在 `src/i18n/mod.rs` 及所有 locale 檔案（en, zh-TW, zh-CN, ja）新增
4. **測試驗證**：`cargo test skill_installer`

### 轉換限制

| 功能 | Claude | Codex | Gemini |
|-----|--------|-------|--------|
| Hooks | ✅ | ❌ | ✅（轉換後）|
| Marketplace | ✅ | ❌ | ✅（變數轉換）|
| `${CLAUDE_PLUGIN_ROOT}` | ✅ | ❌ | ⚠️ 轉為絕對路徑 |
| `allowed-tools` | ✅ | ❌ 移除 | ❌ 移除 |

詳細說明請參閱 [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md)。
