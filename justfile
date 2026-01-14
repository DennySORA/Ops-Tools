# Ops-Tools Development Commands
# Usage: just <recipe>

# Default recipe - show available commands
default:
    @just --list

# Run all quality checks (format, lint, test)
check: fmt-check lint test
    @echo "✓ All checks passed"

# Format check (no changes)
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Lint with clippy (deny warnings)
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --workspace --all-features

# Run tests with output
test-verbose:
    cargo test --workspace --all-features -- --nocapture

# Type check only (fast)
check-types:
    cargo check --workspace --all-targets --all-features

# Build release
build:
    cargo build --release

# Build debug
build-debug:
    cargo build

# Clean build artifacts
clean:
    cargo clean

# Run the application
run:
    cargo run --release

# Watch for changes and run tests
watch-test:
    cargo watch -x "test --workspace --all-features"

# Pre-commit check (run before committing)
pre-commit: fmt-check lint test
    @echo "✓ Ready to commit"

# CI simulation (all strict checks)
ci: fmt-check lint test
    @echo "✓ CI checks passed"

# Count lines of code
loc:
    @find src -name "*.rs" -exec wc -l {} \; | sort -n | tail -20

# Show large files (> 300 lines)
large-files:
    @echo "Files exceeding 300 lines:"
    @find src -name "*.rs" -exec sh -c 'lines=$(wc -l < "{}"); if [ "$lines" -gt 300 ]; then echo "$lines {}"; fi' \; | sort -rn
