# Tools - DevOps CLI Toolset

A Rust-based CLI toolset designed for DevOps tasks, adhering to SOLID principles and Clean Code practices.

## Project Overview

This project provides a suite of tools to assist with development and operations workflows:
*   **Terraform/Terragrunt Cache Cleaner:** Intelligently cleans `.terragrunt-cache`, `.terraform` directories, and lock files.
*   **AI Tool Upgrader:** Batch upgrades for AI coding assistants like Claude Code, OpenAI Codex, and Google Gemini CLI.
*   **Package Security Scanner:** Scans dependencies for high-risk packages with performant parallel processing.
*   **MCP Manager:** Manages Model Context Protocol (MCP) servers for Claude and Codex.

## Building and Running

### Prerequisites
*   Rust (latest stable)
*   A `.env` file is required for MCP management features (see `.env.example`).

### Commands

*   **Build Release:**
    ```bash
    cargo build --release
    ```

*   **Run (Development):**
    ```bash
    cargo run
    ```

*   **Run (Release):**
    ```bash
    ./target/release/tools
    ```

*   **Run Tests:**
    ```bash
    cargo test
    ```
    *   Specific feature test: `cargo test terraform_cleaner`

*   **Linting (Clippy):**
    ```bash
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    ```

*   **Formatting:**
    ```bash
    cargo fmt --all -- --check
    ```

## Architecture

The project follows a strict layered architecture:

*   **`src/main.rs`**: Entry point and interactive menu loop.
*   **`src/core/`**: Core abstractions, error handling, result types, and traits (`FileScanner`, `FileCleaner`).
*   **`src/features/`**: Independent modules for each tool.
    *   `terraform_cleaner/`: Logic for cleaning Terraform caches.
    *   `tool_upgrader/`: Logic for upgrading CLI tools.
    *   `package_scanner/`: Logic for scanning package dependencies.
    *   `mcp_manager/`: Logic for managing MCP servers.
*   **`src/ui/`**: UI components like console output, progress bars, and prompts.

## Development Conventions

*   **SOLID Principles:**
    *   **SRP:** Each module/struct has a single responsibility.
    *   **OCP:** Features are extended via traits without modifying core logic.
    *   **DIP:** Services depend on abstractions (traits), not concrete implementations.
*   **Clean Code:**
    *   Clear, specific naming.
    *   Small functions.
    *   Comments explain "why", not "what".
*   **Error Handling:**
    *   Unified `OperationError` enum for all error types.
    *   Errors must be traceable and categorized.

## Skill Installer Development

When adding new extensions to the Skill Installer feature, you **MUST** follow the guidelines in [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md).

### Extension Configuration

Extensions are defined in `src/features/skill_installer/tools.rs`:

```rust
Extension {
    name: "extension-name",
    display_name_key: keys::SKILL_EXTENSION_NAME,
    extension_type: ExtensionType::Plugin,
    source_repo: "anthropics/claude-code",
    source_path: "plugins/extension-name",
    cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
    skill_subpath: Some("skills/skill-name"),  // For skills/ extraction
    command_file: Some("commands/cmd.md"),      // For command conversion
},
```

### Conversion Methods

| Method | When to Use | Example |
|--------|-------------|---------|
| `skill_subpath` | Plugin has `skills/` subdirectory | `Some("skills/frontend-design")` |
| `command_file` | Plugin has `commands/` only | `Some("commands/code-review.md")` |
| Both `None` | Claude-only (hooks dependency) | `skill_subpath: None, command_file: None` |

### Gemini Extension Format

**Important:** Gemini uses a completely different extension format than Claude/Codex.

Extensions are installed to `~/.gemini/extensions/<name>/` with this structure:

```
~/.gemini/extensions/<extension-name>/
├── gemini-extension.json    # Required manifest
├── GEMINI.md                # Context file
└── commands/
    └── <extension-name>/
        └── invoke.toml      # Commands in TOML format
```

The installer automatically converts:
- Claude SKILL.md → Gemini TOML commands
- Claude command markdown → Gemini TOML format
- Registers extensions in `extension-enablement.json`

### Using Extensions

Invoke commands with `/<extension>:<command>` syntax:

```bash
# In Gemini CLI
> /frontend-design:invoke
> /code-review:invoke
```

### Limitations

Some Claude-specific features cannot be fully converted:
- **allowed-tools** restrictions (Claude-specific security sandbox)
- **Sub-agent orchestration** (agent launching is Claude-specific)
- **Dynamic context** (syntax like `!git status` depends on CLI support)

**Note:** Codex has NO hook support. Plugins with hooks are not available on Codex.

See [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md) for complete documentation.
