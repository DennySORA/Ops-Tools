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
