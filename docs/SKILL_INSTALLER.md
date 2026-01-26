# Skill Installer Extension Guide

This document describes how to add new extensions to the Skill Installer feature, including the rules, constraints, and conversion logic for supporting multiple AI CLI platforms.

## Overview

The Skill Installer manages extensions (skills and plugins) for three AI CLI platforms:

| CLI | Config Directory | Extension Format | Hook Support |
|-----|-----------------|------------------|--------------|
| Claude Code | `~/.claude/` | Plugins (`plugins/`), Skills (`skills/`) | ✅ Full |
| OpenAI Codex | `~/.codex/` | Skills (`skills/`) with `SKILL.md` | ❌ None |
| Google Gemini | `~/.gemini/` | Extensions (`extensions/`) with TOML commands | ✅ Full |

### Gemini Extension Format

**Important:** Gemini CLI uses a completely different extension format than Claude/Codex.

Gemini extensions are installed to `~/.gemini/extensions/<name>/` with this structure:

```
~/.gemini/extensions/<extension-name>/
├── gemini-extension.json    # Required manifest
├── GEMINI.md                # Context file (like README)
└── commands/
    └── <extension-name>/
        └── <command>.toml   # Commands in TOML format
```

#### gemini-extension.json

```json
{
  "name": "extension-name",
  "version": "1.0.0",
  "contextFileName": "GEMINI.md"
}
```

#### Command TOML Format

```toml
description = "Short description of the command"
prompt = """
Your prompt instructions here...
"""
```

#### Extension Enablement

Extensions must be registered in `~/.gemini/extensions/extension-enablement.json`:

```json
{
  "extension-name": {
    "overrides": ["/Users/username/*"]
  }
}
```

#### Using Gemini Extensions

Invoke commands with `/<extension>:<command>`:

```bash
# In Gemini CLI
> /frontend-design:invoke
> /conductor:status
```

## Extension Types

### Plugin (Claude Only)

Plugins are full-featured extensions that can include:
- `.claude-plugin/plugin.json` - Plugin manifest
- `commands/` - Slash commands (e.g., `/commit`, `/review-pr`)
- `hooks/` - Event hooks (e.g., `PreToolUse`, `PostToolUse`, `Stop`)
- `agents/` - Sub-agent definitions
- `skills/` - Embedded skills

**Plugins are Claude-specific and cannot be directly used by Codex or Gemini.**

### Skill (Claude/Codex)

Skills are simple markdown-based instructions:
- `SKILL.md` - The skill definition file with YAML frontmatter

**Note:** Gemini does not use SKILL.md format. The installer automatically converts to Gemini's native TOML extension format.

## Adding New Extensions

### Step 1: Determine Extension Compatibility

Before adding a new extension, evaluate its structure:

| Plugin Structure | Claude | Codex | Gemini | Configuration |
|-----------------|--------|-------|--------|---------------|
| Has `skills/` subdirectory | Plugin | Skill (extract) | Extension (TOML) | `skill_subpath` |
| Has `commands/` only | Plugin | Skill (convert) | Extension (TOML) | `command_file` |
| Has `hooks/` only | Plugin | **Not supported** | Extension (TOML) | `has_hooks: true` |
| Has `hooks/` + `commands/` | Plugin | Skill (convert) | Extension (TOML) | `has_hooks: true` |

**Key insight:** Gemini uses a native extension format with TOML commands. The installer automatically converts Claude plugins to Gemini extensions.

### Step 2: Add Extension Definition

Edit `src/features/skill_installer/tools.rs` and add a new entry to the `EXTENSIONS` array:

```rust
Extension {
    name: "my-extension",                           // Unique identifier
    display_name_key: keys::SKILL_MY_EXTENSION,     // i18n key for display name
    extension_type: ExtensionType::Plugin,          // Always Plugin for GitHub plugins
    source_repo: "anthropics/claude-code",          // GitHub repo
    source_path: "plugins/my-extension",            // Path within repo
    cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],  // Supported CLIs
    skill_subpath: Some("skills/my-skill"),         // If has skills/ subdirectory
    command_file: None,                             // Or specify command file
    has_hooks: false,                               // Set true if plugin uses hooks
},
```

### Step 3: Choose the Correct Configuration

#### Option A: Plugin with `skills/` subdirectory

Use when the plugin contains a `skills/` folder with SKILL.md:

```rust
Extension {
    name: "frontend-design",
    display_name_key: keys::SKILL_FRONTEND_DESIGN,
    extension_type: ExtensionType::Plugin,
    source_repo: "anthropics/claude-code",
    source_path: "plugins/frontend-design",
    cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
    skill_subpath: Some("skills/frontend-design"),  // Path to skill within plugin
    command_file: None,
},
```

**Behavior:**
- Claude: Installs full plugin to `~/.claude/plugins/frontend-design/`
- Codex: Extracts skill to `~/.codex/skills/frontend-design/SKILL.md`
- Gemini: Creates extension at `~/.gemini/extensions/frontend-design/` with TOML commands

#### Option B: Plugin with `commands/` only (no skills/)

Use when the plugin only has commands that can be converted to skills:

```rust
Extension {
    name: "code-review",
    display_name_key: keys::SKILL_CODE_REVIEW,
    extension_type: ExtensionType::Plugin,
    source_repo: "anthropics/claude-code",
    source_path: "plugins/code-review",
    cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
    skill_subpath: None,
    command_file: Some("commands/code-review.md"),  // Command file to convert
},
```

**Behavior:**
- Claude: Installs full plugin to `~/.claude/plugins/code-review/`
- Codex: Converts command to `~/.codex/skills/code-review/SKILL.md`
- Gemini: Creates extension at `~/.gemini/extensions/code-review/` with TOML commands

#### Option C: Plugin with Hooks (Claude + Gemini)

Use when the plugin uses hooks. Gemini converts hooks to its native extension format:

```rust
Extension {
    name: "ralph-wiggum",
    display_name_key: keys::SKILL_RALPH_WIGGUM,
    extension_type: ExtensionType::Plugin,
    source_repo: "anthropics/claude-code",
    source_path: "plugins/ralph-wiggum",
    cli_support: &[CliType::Claude, CliType::Gemini],  // Both support hooks!
    skill_subpath: None,
    command_file: None,
    has_hooks: true,  // Important: enables Gemini hook migration
},
```

**Behavior:**
- Claude: Installs full plugin to `~/.claude/plugins/ralph-wiggum/`
- Gemini: Creates extension at `~/.gemini/extensions/ralph-wiggum/` with hooks converted to native format
- Codex: Extension not available (no hook support)

#### Option D: Claude-only Plugin

Use when the plugin has features that truly cannot work on other CLIs:

```rust
Extension {
    name: "claude-specific",
    display_name_key: keys::SKILL_CLAUDE_SPECIFIC,
    extension_type: ExtensionType::Plugin,
    source_repo: "anthropics/claude-code",
    source_path: "plugins/claude-specific",
    cli_support: &[CliType::Claude],  // Claude only
    skill_subpath: None,
    command_file: None,
    has_hooks: false,
},
```

**Behavior:**
- Claude: Installs full plugin
- Codex/Gemini: Extension not available (filtered out)

### Step 4: Add i18n Keys

Add the display name key to `src/i18n/mod.rs`:

```rust
pub const SKILL_MY_EXTENSION: &str = "skill.my_extension";
```

Add translations to all locale files in `src/i18n/locales/`:

**en.toml:**
```toml
"skill.my_extension" = "My Extension Name"
```

**zh-TW.toml:**
```toml
"skill.my_extension" = "我的擴充套件"
```

**zh-CN.toml:**
```toml
"skill.my_extension" = "我的扩展"
```

**ja.toml:**
```toml
"skill.my_extension" = "マイ拡張機能"
```

### Step 5: Update Tests

Update the test assertions in `src/features/skill_installer/tools.rs` if needed:

```rust
#[test]
fn test_get_available_extensions_codex() {
    let extensions = get_available_extensions(CliType::Codex);
    assert!(!extensions.is_empty());
    // All Codex extensions must have either skill_subpath or command_file
    assert!(extensions
        .iter()
        .all(|ext| ext.skill_subpath.is_some() || ext.command_file.is_some()));
}
```

## SKILL.md Format Conversion

The installer automatically converts SKILL.md format based on the target CLI:

### Claude Format (Original)

Claude supports extended fields:

```yaml
---
name: My Skill
description: Multi-line description
  that spans multiple lines
allowed-tools: ["Bash", "Read", "Write"]
context: additional context
license: MIT
---

# Skill Body
Instructions here...
```

### Codex Format (Converted)

Codex only recognizes `name` and `description` (single line):

```yaml
---
name: My Skill
description: Multi-line description
---

# Skill Body
Instructions here...
```

**Conversion rules:**
- Remove all fields except `name` and `description`
- Description is truncated to first line only

### Gemini Format (Converted to TOML)

Gemini uses native extension format with TOML commands:

```
~/.gemini/extensions/my-skill/
├── gemini-extension.json
├── GEMINI.md
└── commands/
    └── my-skill/
        └── invoke.toml
```

**invoke.toml:**
```toml
description = "Multi-line description that spans multiple lines"
prompt = """
# Skill Body
Instructions here...
"""
```

**Conversion rules:**
- Creates `gemini-extension.json` manifest
- Creates `GEMINI.md` context file
- Converts skill body to TOML `prompt` field
- Registers in `extension-enablement.json`
- Command invoked via `/<extension>:invoke`

## Command to Skill Conversion

When converting a Claude command file to SKILL.md:

### Original Command Format

```yaml
---
allowed-tools: Bash(git *), Read, Write
description: "Create a git commit"
argument-hint: "[commit message]"
---

## Context
- Current git status: !`git status`

## Your task
Based on the above changes, create a commit...
```

### Converted SKILL.md

```yaml
---
name: commit-commands
description: Create a git commit
---

## Context
- Current git status: !`git status`

## Your task
Based on the above changes, create a commit...
```

**Conversion rules:**
- `allowed-tools` removed (Claude-specific)
- `argument-hint` removed (Claude-specific)
- `name` derived from extension name
- `description` preserved (single line for Codex)
- Body content preserved as-is

## Limitations

### Codex Limitations

The following plugin features **cannot** be used with Codex:

1. **Hooks** - Event-based triggers (PreToolUse, PostToolUse, Stop)
   - Codex has no hook system
   - Example: `ralph-wiggum` uses stop hooks - not available on Codex

### Gemini Extension Format

Gemini uses a native extension format that differs from Claude/Codex:
- Extensions are installed to `~/.gemini/extensions/<name>/`
- Commands are TOML files (not Markdown)
- Extensions must be registered in `extension-enablement.json`
- Invoke commands with `/<extension>:<command>` syntax

The installer automatically converts Claude plugins to Gemini extension format:
- Creates `gemini-extension.json` manifest
- Converts commands to TOML format
- Registers in `extension-enablement.json`

### General Limitations

The following features may have limited functionality:

2. **Dynamic Context** - Live command execution in prompts
   - Syntax like `!git status` may not work in all CLIs
   - The converted skill preserves the syntax, but behavior depends on CLI support

3. **Tool Restrictions** - `allowed-tools` field
   - Claude uses this for security sandboxing
   - Codex/Gemini don't support tool restrictions in skills

4. **Sub-agents** - Complex multi-agent orchestration
   - Commands that launch sub-agents may not work as expected
   - The instructions are preserved but agent launching is Claude-specific

### Best Practices

1. **Prefer plugins with skills/ subdirectory** - These are designed for cross-CLI compatibility
2. **Test converted skills manually** - Verify the skill works in target CLI
3. **Document CLI-specific limitations** - If a skill has reduced functionality, note it in the description
4. **Keep command instructions generic** - Avoid Claude-specific terminology when possible

## Verification Checklist

Before submitting a new extension:

- [ ] Extension definition added to `EXTENSIONS` array
- [ ] i18n keys added to `mod.rs` and all 4 locale files
- [ ] `cli_support` correctly specifies supported CLIs
- [ ] Conversion method configured:
  - `skill_subpath` for plugins with skills/ subdirectory
  - `command_file` for command-based conversion
  - `has_hooks: true` for plugins with hooks (enables Gemini support)
- [ ] Tests pass: `cargo test skill_installer`
- [ ] Lint passes: `cargo clippy`
- [ ] Format passes: `cargo fmt --check`
- [ ] Manual test: Install on all supported CLIs and verify functionality

## File Locations

| File | Purpose |
|------|---------|
| `src/features/skill_installer/tools.rs` | Extension definitions |
| `src/features/skill_installer/executor.rs` | Install/remove/convert logic |
| `src/features/skill_installer/mod.rs` | Main UI flow |
| `src/i18n/mod.rs` | i18n keys |
| `src/i18n/locales/*.toml` | Translations |

## CLI Comparison Summary

### Installation Paths

| CLI | Skills/Extensions Path | Plugins Path |
|-----|----------------------|--------------|
| Claude | `~/.claude/skills/` | `~/.claude/plugins/` |
| Codex | `~/.codex/skills/` | N/A |
| Gemini | `~/.gemini/extensions/` | N/A |

### Format Comparison

| Feature | Claude | Codex | Gemini |
|---------|--------|-------|--------|
| Skill format | `SKILL.md` (YAML frontmatter) | `SKILL.md` (simplified) | `invoke.toml` |
| Plugin support | ✅ Full | ❌ None | ❌ None (uses extensions) |
| Hook support | ✅ Full | ❌ None | ✅ Native |
| Command invocation | `/skill-name` | `/skill-name` | `/extension:command` |
| Registration | Automatic | Automatic | `extension-enablement.json` |

### Usage Examples

**Claude:**
```bash
> /frontend-design
> /commit
```

**Codex:**
```bash
> /frontend-design
> /commit
```

**Gemini:**
```bash
> /frontend-design:invoke
> /commit-commands:invoke
```
