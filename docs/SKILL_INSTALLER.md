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
| Requires marketplace root | Plugin (marketplace) | **Not supported** | **Not supported** | `marketplace_name` |

**Key insight:** Gemini uses a native extension format with TOML commands. The installer automatically converts Claude plugins to Gemini extensions.

**Marketplace plugins:** Some third-party plugins (like `claude-mem`) have scripts that reference the marketplace root directory. These require full marketplace installation with git clone.

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
    marketplace_name: None,                         // For marketplace-based plugins
    marketplace_plugin_path: None,                  // Plugin path within marketplace repo
    version: None,                                  // Plugin version for marketplace installs
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

#### Option E: Marketplace-based Plugin (Third-party)

Use when the plugin has scripts that reference the marketplace root directory (e.g., `smart-install.js` that looks for `package.json` in the parent directories):

```rust
Extension {
    name: "claude-mem",
    display_name_key: keys::SKILL_CLAUDE_MEM,
    extension_type: ExtensionType::Plugin,
    source_repo: "thedotmack/claude-mem",
    source_path: "plugin",  // Not used for marketplace installs
    cli_support: &[CliType::Claude, CliType::Gemini],
    skill_subpath: None,
    command_file: None,
    has_hooks: true,
    marketplace_name: Some("thedotmack"),      // Marketplace identifier
    marketplace_plugin_path: Some("plugin"),   // Path to plugin within repo
    version: Some("9.0.12"),                   // Plugin version
},
```

**Behavior:**
- Claude: Full marketplace installation:
  1. Git clones repo to `~/.claude/plugins/marketplaces/<marketplace_name>/`
  2. Creates symlink: `cache/<marketplace>/<plugin>/<version>/` → `marketplaces/<marketplace>/<plugin_path>/`
  3. Runs `npm install` or `bun install` for dependencies
  4. Updates `known_marketplaces.json` and `installed_plugins.json`
  5. Updates `settings.json` with `enabledPlugins`
- Gemini: Extended marketplace installation with variable conversion:
  1. Git clones repo to temp directory
  2. Copies plugin to `~/.gemini/extensions/<name>/`
  3. Runs `npm install` or `bun install` for dependencies
  4. **Converts `${CLAUDE_PLUGIN_ROOT}` to absolute path** (see Variable Conversion below)
  5. Creates `gemini-extension.json` manifest
  6. Converts hooks to native Gemini format
  7. Registers in `extension-enablement.json`
- Codex: Not supported (marketplace plugins require Claude-specific features or hooks)

## Marketplace Plugin Architecture

Marketplace-based plugins have a more complex installation structure because they contain scripts that reference the marketplace root directory. This section documents the technical details.

### Directory Structure (Claude)

```
~/.claude/plugins/
├── marketplaces/
│   └── <marketplace_name>/           # Full git clone of the repo
│       ├── package.json              # Root package.json (required by some scripts)
│       ├── plugin/                   # Plugin directory
│       │   ├── .claude-plugin/
│       │   │   └── plugin.json
│       │   ├── hooks/
│       │   ├── commands/
│       │   └── node_modules/
│       └── ...other repo files
├── cache/
│   └── <marketplace_name>/
│       └── <plugin_name>/
│           └── <version>/            # Symlink → marketplaces/<marketplace>/<plugin_path>
├── known_marketplaces.json           # Registry of marketplace sources
└── installed_plugins.json            # Registry of installed plugins
```

### Directory Structure (Gemini)

```
~/.gemini/extensions/
└── <plugin_name>/
    ├── gemini-extension.json         # Required manifest
    ├── GEMINI.md                     # Context file
    ├── hooks/                        # Converted from Claude hooks
    ├── commands/
    │   └── <plugin_name>/
    │       └── invoke.toml           # TOML commands
    └── node_modules/                 # Dependencies (if needed)
```

### JSON Registries

#### known_marketplaces.json

Tracks marketplace sources and locations:

```json
{
  "thedotmack": {
    "source": {
      "source": "github",
      "repo": "thedotmack/claude-mem"
    },
    "installLocation": "/Users/username/.claude/plugins/marketplaces/thedotmack",
    "lastUpdated": "2026-02-04T03:49:28.518199Z"
  }
}
```

#### installed_plugins.json

Tracks installed plugins with version info:

```json
{
  "version": 2,
  "plugins": {
    "claude-mem@thedotmack": [
      {
        "scope": "user",
        "installPath": "/Users/username/.claude/plugins/cache/thedotmack/claude-mem/9.0.12",
        "version": "9.0.12",
        "installedAt": "2026-02-04T03:49:28.556745Z",
        "lastUpdated": "2026-02-04T03:49:28.556745Z",
        "isLocal": true
      }
    ]
  }
}
```

#### settings.json (enabledPlugins)

Enables the plugin for use:

```json
{
  "enabledPlugins": {
    "claude-mem@thedotmack": true
  }
}
```

## Variable Conversion

### `${CLAUDE_PLUGIN_ROOT}` Variable

Claude plugins can use the `${CLAUDE_PLUGIN_ROOT}` variable in their scripts and configurations. This variable points to the plugin's installation directory.

**Problem:** This variable is Claude-specific and won't work in Gemini or other CLIs.

**Solution:** The installer automatically converts this variable to an absolute path when installing for Gemini:

```javascript
// Original (Claude hook script)
const pluginRoot = process.env.CLAUDE_PLUGIN_ROOT || '${CLAUDE_PLUGIN_ROOT}';

// Converted (Gemini)
const pluginRoot = '/Users/username/.gemini/extensions/claude-mem';
```

### Conversion Process

The installer scans all files in the plugin directory for `${CLAUDE_PLUGIN_ROOT}` and replaces it with the absolute installation path:

1. **JavaScript files** (`.js`): Replaces string occurrences
2. **JSON files** (`.json`): Replaces in configuration values
3. **Shell scripts** (`.sh`): Replaces variable references
4. **Markdown files** (`.md`): Replaces in documentation

**Files converted:**
- Hook scripts (`hooks/*.js`)
- Configuration files (`*.json`)
- Any other text files referencing the variable

### When Variable Conversion is Needed

| Scenario | Needs Conversion |
|----------|-----------------|
| Plugin uses `${CLAUDE_PLUGIN_ROOT}` | ✅ Yes |
| Plugin has hooks referencing plugin root | ✅ Yes |
| Plugin with static paths only | ❌ No |
| Standard skill/command plugin | ❌ No |

## Dependency Installation

### npm/bun Detection

The installer automatically detects and uses the best available package manager:

1. **bun** (preferred if available): Faster installation
2. **npm** (fallback): Standard Node.js package manager

### Installation Process

```bash
# Installer checks for bun first
if command -v bun &> /dev/null; then
    bun install --production
else
    npm install --production
fi
```

### When Dependencies are Installed

Dependencies are installed for:
- Marketplace-based plugins with `package.json`
- Plugins with hooks that require Node.js modules
- Any plugin with a `package.json` in the installation directory

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

## Hooks Conversion

### Claude Hooks Overview

Claude plugins can define hooks that trigger on specific events:

| Hook Type | Trigger |
|-----------|---------|
| `PreToolUse` | Before a tool is executed |
| `PostToolUse` | After a tool completes |
| `Stop` | When the agent stops or completes |
| `Notification` | On various events |

### Gemini Hook Support

Gemini CLI has native hook support that mirrors Claude's system. The installer converts Claude hooks to Gemini's format:

**Claude hook structure:**
```
.claude-plugin/
└── hooks/
    ├── PreToolUse/
    │   └── my-hook.js
    └── Stop/
        └── cleanup.js
```

**Converted Gemini structure:**
```
hooks/
├── PreToolUse/
│   └── my-hook.js      # Script converted with variable replacement
└── Stop/
    └── cleanup.js      # Script converted with variable replacement
```

### Hook Conversion Process

1. Copy hook directory structure
2. Convert `${CLAUDE_PLUGIN_ROOT}` to absolute path in all scripts
3. Ensure Node.js dependencies are installed
4. Register extension in `extension-enablement.json`

### Codex Hook Limitation

**Codex does not support hooks.** Plugins that rely on hooks cannot be installed on Codex:

```rust
// This plugin is NOT available on Codex
Extension {
    name: "ralph-wiggum",
    cli_support: &[CliType::Claude, CliType::Gemini],  // No Codex
    has_hooks: true,
}
```

## Limitations

### Codex Limitations

The following plugin features **cannot** be used with Codex:

1. **Hooks** - Event-based triggers (PreToolUse, PostToolUse, Stop)
   - Codex has no hook system
   - Example: `ralph-wiggum` uses stop hooks - not available on Codex

2. **Marketplace-based plugins** - Plugins requiring full repo structure
   - Codex cannot handle complex marketplace installations
   - Example: `claude-mem` requires marketplace structure - not available on Codex

### Marketplace Plugin Limitations

1. **Gemini variable conversion** - Not all Claude-specific features can be converted:
   - `${CLAUDE_PLUGIN_ROOT}` is converted to absolute path
   - Other Claude-specific environment variables may not work
   - Plugin-specific APIs (e.g., Claude memory APIs) are not available

2. **Full repo required** - Some plugins reference files outside the plugin directory:
   - Scripts may look for `package.json` in parent directories
   - Relative imports may reference marketplace root files

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

### Basic Requirements

- [ ] Extension definition added to `EXTENSIONS` array
- [ ] i18n keys added to `mod.rs` and all 4 locale files (en, zh-TW, zh-CN, ja)
- [ ] `cli_support` correctly specifies supported CLIs
- [ ] Conversion method configured:
  - `skill_subpath` for plugins with skills/ subdirectory
  - `command_file` for command-based conversion
  - `has_hooks: true` for plugins with hooks (enables Gemini support)

### Marketplace Plugin Requirements

- [ ] `marketplace_name` set if plugin requires full repo structure
- [ ] `marketplace_plugin_path` specifies path to plugin within repo
- [ ] `version` set for version tracking
- [ ] Verified plugin scripts work with `${CLAUDE_PLUGIN_ROOT}` conversion
- [ ] Dependencies install correctly with npm/bun

### Testing

- [ ] Unit tests pass: `cargo test skill_installer`
- [ ] Lint passes: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Format passes: `cargo fmt --all -- --check`
- [ ] Manual test: Install on all supported CLIs and verify functionality
- [ ] Verify JSON registries are updated correctly (Claude)
- [ ] Verify extension-enablement.json is updated (Gemini)

## File Locations

### Source Code

| File | Purpose |
|------|---------|
| `src/features/skill_installer/tools.rs` | Extension definitions |
| `src/features/skill_installer/executor.rs` | Install/remove/convert logic |
| `src/features/skill_installer/mod.rs` | Main UI flow |
| `src/i18n/mod.rs` | i18n keys |
| `src/i18n/locales/*.toml` | Translations |

### Claude Installation Files

| File | Purpose |
|------|---------|
| `~/.claude/plugins/marketplaces/` | Git cloned marketplace repos |
| `~/.claude/plugins/cache/` | Symlinks to installed plugins |
| `~/.claude/plugins/known_marketplaces.json` | Marketplace source registry |
| `~/.claude/plugins/installed_plugins.json` | Installed plugins registry |
| `~/.claude/settings.json` | Plugin enablement config |

### Gemini Installation Files

| File | Purpose |
|------|---------|
| `~/.gemini/extensions/` | Installed extensions |
| `~/.gemini/extensions/extension-enablement.json` | Extension enablement config |
| `~/.gemini/extensions/<name>/gemini-extension.json` | Extension manifest |
| `~/.gemini/extensions/<name>/GEMINI.md` | Extension context file |

### Codex Installation Files

| File | Purpose |
|------|---------|
| `~/.codex/skills/` | Installed skills |
| `~/.codex/skills/<name>/SKILL.md` | Skill definition file |

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
