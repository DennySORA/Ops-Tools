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

## Skill Installer Development

When adding new extensions to the Skill Installer feature, you **MUST** follow the guidelines in [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md).

### Quick Reference

| Plugin Structure | Claude | Codex | Gemini | Configuration |
|-----------------|--------|-------|--------|---------------|
| Has `skills/` subdirectory | Plugin | Skill (extract) | Extension (TOML) | `skill_subpath` |
| Has `commands/` only | Plugin | Skill (convert) | Extension (TOML) | `command_file` |
| Has `hooks/` only | Plugin | **Not supported** | Extension (TOML) | `has_hooks: true` |
| Has `hooks/` + `commands/` | Plugin | Skill (convert) | Extension (TOML) | `has_hooks: true` |

### Gemini Extension Format

**Important:** Gemini uses a different format than Claude/Codex:
- Extensions installed to `~/.gemini/extensions/<name>/`
- Commands are TOML files (not Markdown)
- Requires `gemini-extension.json` manifest
- Invoke with `/<extension>:<command>` syntax

### Required Steps

1. Add extension to `src/features/skill_installer/tools.rs`
2. Add i18n keys to `src/i18n/mod.rs` and all locale files (en, zh-TW, zh-CN, ja)
3. Set appropriate `cli_support`, `skill_subpath`, or `command_file`
4. Run tests: `cargo test skill_installer`

### Conversion Limitations

- **Hooks** - Codex has no hook system; Gemini converts to native format
- **allowed-tools** field is removed during conversion (Claude-specific)
- **Gemini format** - Commands converted to TOML, registered in enablement file
- **description** truncated to single line for Codex (auto-converted)
