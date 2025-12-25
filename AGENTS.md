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
