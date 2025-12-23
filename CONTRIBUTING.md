# Contributing to DevOps CLI Toolset

Thank you for your interest in contributing to the DevOps CLI Toolset! We welcome contributions from the community.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally.
3.  **Create a new branch** for your feature or bug fix (`git checkout -b feature/amazing-feature`).

## Development Workflow

### Prerequisites

*   Rust (latest stable)
*   A `.env` file (copy from `.env.example`)

### Building and Testing

Ensure the project builds and tests pass:

```bash
cargo build
cargo test
```

### Style Guidelines

We follow standard Rust coding conventions.

*   **Formatting**: Run `cargo fmt` before committing.
*   **Linting**: Run `cargo clippy` and fix any warnings.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Pull Request Process

1.  Update the `README.md` if you change functionality or add new features.
2.  Update `CHANGELOG.md` with your changes under the `[Unreleased]` section.
3.  Submit a Pull Request to the `main` branch.
4.  Provide a clear description of your changes and reference any related issues.

## Reporting Issues

If you find a bug or have a feature request, please open an issue on GitHub.
