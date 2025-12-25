# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Improved MCP list parsing to detect Gemini CLI entries with checkmarks and ANSI colors.
- Added Gemini MCP settings migration to remove invalid `type` fields and map HTTP URLs.
- Removed invalid `--env` option when adding Context7 MCP via Codex CLI.

## [0.1.0] - 2025-12-23

### Added
- Initial release of DevOps CLI Toolset.
- **Terraform Cleaner**: Feature to clean `.terragrunt-cache`, `.terraform` directories, and lock files.
- **Tool Upgrader**: Feature to upgrade AI assistants (Claude Code, OpenAI Codex, Gemini CLI).
- **MCP Manager**: Feature to manage MCP servers for Claude and Codex.
- Interactive CLI menu using `dialoguer`.
- Documentation in English, Traditional Chinese, Simplified Chinese, and Japanese.
