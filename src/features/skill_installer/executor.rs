use super::tools::{CliType, Extension, ExtensionType};
use crate::core::{OperationError, Result};
use crate::i18n::keys;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Loop Runner SKILL.md content for Claude Code
/// Claude has built-in /loop support via CronCreate/CronList/CronDelete tools.
/// This skill provides a consistent interface.
const LOOP_RUNNER_CLAUDE: &str = r#"---
name: loop-runner
description: Schedule periodic task execution with customizable intervals
---

# Loop Runner

Schedule recurring tasks to run at specified intervals.

## Usage

When the user requests periodic execution (e.g., "every 5 minutes check if tests pass"),
parse their request and use the built-in cron tools.

## Commands
- `loop <interval> <task>` - Start a new periodic task
- `loop list` - List all active loops
- `loop cancel <id>` - Cancel a specific loop
- `loop results <id>` - Check results of a loop

## Interval Parsing

Convert the user's interval to a cron expression:
- `Ns` or `N seconds` → Round up to 1 minute minimum: `*/1 * * * *`
- `Nm` or `N minutes` → `*/N * * * *`
- `Nh` or `N hours` → `0 */N * * *`
- `Nd` or `N days` → `0 0 */N * *`

## Implementation

1. **Start a loop**: Parse interval and task, call CronCreate with the cron expression and the task as the prompt. Set recurring to true.
2. **List loops**: Call CronList to show all active scheduled tasks.
3. **Cancel a loop**: Call CronDelete with the task ID.
4. **Check results**: The cron system handles execution automatically. Results appear in the conversation.

## Examples
- "loop every 5m to check build status" → CronCreate with `*/5 * * * *`
- "loop hourly to run tests" → CronCreate with `0 * * * *`
- "loop daily to check dependencies" → CronCreate with `0 0 * * *`
"#;

/// Loop Runner SKILL.md content for Codex CLI
/// Codex does not have built-in cron tools, so we use background shell processes.
const LOOP_RUNNER_CODEX: &str = r#"---
name: loop-runner
description: Schedule periodic task execution with customizable intervals
---

# Loop Runner

Schedule recurring tasks to run at specified intervals using background processes.

## Usage

When the user requests periodic execution (e.g., "every 5 minutes check if tests pass"),
parse their request and manage background loop processes.

## Commands
- `loop <interval> <task>` - Start a new periodic task
- `loop list` - List all active loops
- `loop cancel <id>` - Cancel a specific loop
- `loop results <id>` - Show recent results from a loop

## Creating a Loop

1. Generate a unique 8-character ID: `date +%s%N | md5sum | head -c 8`
2. Convert interval to seconds (e.g., 5m → 300, 1h → 3600)
3. Create directory: `mkdir -p ~/.codex/loops`
4. Write the loop script at `~/.codex/loops/<id>.sh`:

```bash
#!/bin/bash
# Loop ID: <id>
# Task: <task description>
# Interval: <seconds>s
# Created: <ISO timestamp>

INTERVAL=<seconds>
LOG="$HOME/.codex/loops/<id>.log"

while true; do
    echo "=== $(date -Iseconds) ===" >> "$LOG"
    ( <task_commands> ) >> "$LOG" 2>&1
    echo "--- exit: $? ---" >> "$LOG"
    echo "" >> "$LOG"
    sleep "$INTERVAL"
done
```

5. Make executable: `chmod +x ~/.codex/loops/<id>.sh`
6. Start in background: `nohup bash ~/.codex/loops/<id>.sh > /dev/null 2>&1 & echo $!`
7. Save PID: `echo <pid> > ~/.codex/loops/<id>.pid`
8. Report to user: "Loop `<id>` started: <task> every <interval>"

## Listing Loops

For each `.pid` file in `~/.codex/loops/`:
1. Read PID from file
2. Check if alive: `kill -0 <pid> 2>/dev/null`
3. Read the header comments from the corresponding `.sh` file for task/interval info
4. Display: `[<id>] <task> (every <interval>) - <status>`
5. Clean up dead entries (remove .pid file if process is gone)

## Cancelling a Loop

1. Read PID: `cat ~/.codex/loops/<id>.pid`
2. Kill process: `kill <pid> 2>/dev/null`
3. Remove files: `rm -f ~/.codex/loops/<id>.pid ~/.codex/loops/<id>.sh`
4. Keep log: `~/.codex/loops/<id>.log` remains for review
5. Report: "Loop `<id>` cancelled"

## Checking Results

Display the last 50 lines: `tail -50 ~/.codex/loops/<id>.log`

## Interval Parsing

- `Ns` or `N seconds` → N seconds (minimum 60)
- `Nm` or `N minutes` → N * 60 seconds
- `Nh` or `N hours` → N * 3600 seconds
- `Nd` or `N days` → N * 86400 seconds
- Default (no unit specified): treat as minutes
"#;

/// Loop Runner content for Gemini CLI
const LOOP_RUNNER_GEMINI: &str = r#"# Loop Runner

Schedule recurring tasks to run at specified intervals using background processes.

## Usage

When the user requests periodic execution, parse their request and manage background loop processes.

## Commands
- `loop <interval> <task>` - Start a new periodic task
- `loop list` - List all active loops
- `loop cancel <id>` - Cancel a specific loop
- `loop results <id>` - Show recent results from a loop

## Creating a Loop

1. Generate a unique 8-character ID: `date +%s%N | md5sum | head -c 8`
2. Convert interval to seconds (e.g., 5m = 300, 1h = 3600)
3. Create directory: `mkdir -p ~/.gemini/loops`
4. Write the loop script at `~/.gemini/loops/<id>.sh`:

```bash
#!/bin/bash
# Loop ID: <id>
# Task: <task description>
# Interval: <seconds>s
# Created: <ISO timestamp>

INTERVAL=<seconds>
LOG="$HOME/.gemini/loops/<id>.log"

while true; do
    echo "=== $(date -Iseconds) ===" >> "$LOG"
    ( <task_commands> ) >> "$LOG" 2>&1
    echo "--- exit: $? ---" >> "$LOG"
    echo "" >> "$LOG"
    sleep "$INTERVAL"
done
```

5. Make executable and start: `chmod +x ~/.gemini/loops/<id>.sh && nohup bash ~/.gemini/loops/<id>.sh > /dev/null 2>&1 & echo $!`
6. Save PID: `echo <pid> > ~/.gemini/loops/<id>.pid`

## Listing Loops

Check each `.pid` file in `~/.gemini/loops/`, verify process is alive with `kill -0`, read task info from `.sh` header comments.

## Cancelling a Loop

Read PID, kill process, remove `.pid` and `.sh` files. Keep `.log` for review.

## Checking Results

Display with: `tail -50 ~/.gemini/loops/<id>.log`

## Interval Parsing

- `Ns` = N seconds (min 60), `Nm` = N*60s, `Nh` = N*3600s, `Nd` = N*86400s
- Default: treat as minutes
"#;

/// SessionStart hook script for Codex loop-runner
/// Displays active loops when a new Codex session starts
const LOOP_RUNNER_HOOK_SCRIPT: &str = r#"#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const loopsDir = path.join(process.env.HOME || '', '.codex', 'loops');

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (d) => { input += d; });
process.stdin.on('end', () => {
  const activeLoops = [];

  try {
    if (fs.existsSync(loopsDir)) {
      const files = fs.readdirSync(loopsDir);
      for (const file of files) {
        if (!file.endsWith('.pid')) continue;
        const id = file.replace('.pid', '');
        const pidFile = path.join(loopsDir, file);
        const pid = parseInt(fs.readFileSync(pidFile, 'utf8').trim(), 10);

        try {
          process.kill(pid, 0); // Check if alive
          const shFile = path.join(loopsDir, id + '.sh');
          if (fs.existsSync(shFile)) {
            const script = fs.readFileSync(shFile, 'utf8');
            const taskMatch = script.match(/^# Task: (.+)$/m);
            const intervalMatch = script.match(/^# Interval: (.+)$/m);
            activeLoops.push({
              id,
              task: taskMatch ? taskMatch[1] : 'unknown',
              interval: intervalMatch ? intervalMatch[1] : 'unknown'
            });
          }
        } catch (e) {
          // Process dead, clean up stale PID file
          try { fs.unlinkSync(pidFile); } catch (_) {}
        }
      }
    }
  } catch (e) { /* ignore errors */ }

  const output = { continue: true };
  if (activeLoops.length > 0) {
    const lines = activeLoops.map(
      (l) => `  [${l.id}] ${l.task} (every ${l.interval})`
    );
    output.systemMessage = 'Active loops:\\n' + lines.join('\\n') +
      '\\n\\nUse /loop-runner to manage loops.';
  }

  process.stdout.write(JSON.stringify(output) + '\n');
});
"#;

/// Check if a hooks.json entry contains a hook command matching the given path prefix
fn entry_contains_plugin_path(entry: &serde_json::Value, path_prefix: &str) -> bool {
    entry
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks.iter().any(|hook| {
                hook.get("command")
                    .and_then(|c| c.as_str())
                    .map(|cmd| cmd.contains(path_prefix))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Extension executor for installing and removing extensions
pub struct ExtensionExecutor {
    cli: CliType,
}

impl ExtensionExecutor {
    pub fn new(cli: CliType) -> Self {
        Self { cli }
    }

    /// Get the installation directory for a specific extension type
    fn install_dir(&self, ext_type: ExtensionType) -> PathBuf {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let cli_dir = home.join(self.cli.config_dir_name());

        match (self.cli, ext_type) {
            // Gemini uses extensions/ for everything (not skills/)
            (CliType::Gemini, _) => cli_dir.join("extensions"),
            (_, ExtensionType::Skill) => cli_dir.join("skills"),
            (_, ExtensionType::Plugin) => cli_dir.join("plugins"),
        }
    }

    /// Get the Gemini extension enablement file path
    fn gemini_enablement_file(&self) -> PathBuf {
        let home = dirs::home_dir().expect("Cannot find home directory");
        home.join(".gemini/extensions/extension-enablement.json")
    }

    /// List installed extensions (returns a map of name -> extension type)
    pub fn list_installed(&self) -> Result<HashMap<String, ExtensionType>> {
        let mut installed = HashMap::new();

        if self.cli == CliType::Gemini {
            // Gemini: scan extensions directory for directories with gemini-extension.json
            let extensions_dir = self.install_dir(ExtensionType::Skill); // returns ~/.gemini/extensions/
            if extensions_dir.exists() {
                if let Ok(entries) = fs::read_dir(&extensions_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            // Check if it has gemini-extension.json (our installed extensions)
                            let manifest = entry.path().join("gemini-extension.json");
                            if manifest.exists() {
                                // Check if it has hooks/ directory -> Plugin, otherwise Skill
                                let hooks_dir = entry.path().join("hooks");
                                if hooks_dir.exists() {
                                    installed.insert(name, ExtensionType::Plugin);
                                } else {
                                    installed.insert(name, ExtensionType::Skill);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Claude/Codex: scan skills directory
            let skills_dir = self.install_dir(ExtensionType::Skill);
            if skills_dir.exists() {
                if let Ok(entries) = fs::read_dir(&skills_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            let name = entry.file_name().to_string_lossy().to_string();
                            installed.insert(name, ExtensionType::Skill);
                        }
                    }
                }
            }

            // For Codex, also scan plugins directory (hook-based plugins)
            if self.cli == CliType::Codex {
                let plugins_dir = self.codex_plugins_dir();
                if plugins_dir.exists() {
                    if let Ok(entries) = fs::read_dir(&plugins_dir) {
                        for entry in entries.flatten() {
                            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                                let name = entry.file_name().to_string_lossy().to_string();
                                // Check if it has a hooks/ subdirectory
                                let hooks_dir = entry.path().join("hooks");
                                if hooks_dir.exists() {
                                    installed.insert(name, ExtensionType::Plugin);
                                }
                            }
                        }
                    }
                }
            }

            // For Claude, also scan plugins directory and cache directory
            if self.cli == CliType::Claude {
                let plugins_dir = self.install_dir(ExtensionType::Plugin);
                if plugins_dir.exists() {
                    if let Ok(entries) = fs::read_dir(&plugins_dir) {
                        for entry in entries.flatten() {
                            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                                let name = entry.file_name().to_string_lossy().to_string();
                                // Skip cache and marketplaces directories
                                if name != "cache" && name != "marketplaces" {
                                    installed.insert(name, ExtensionType::Plugin);
                                }
                            }
                        }
                    }
                }

                // Also scan cache directory for marketplace-based plugins
                let cache_dir = plugins_dir.join("cache");
                if cache_dir.exists() {
                    // Structure: cache/<marketplace>/<plugin>/<version>/
                    if let Ok(marketplaces) = fs::read_dir(&cache_dir) {
                        for marketplace in marketplaces.flatten() {
                            if marketplace
                                .file_type()
                                .map(|ft| ft.is_dir())
                                .unwrap_or(false)
                            {
                                if let Ok(plugins) = fs::read_dir(marketplace.path()) {
                                    for plugin in plugins.flatten() {
                                        if plugin.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                                        {
                                            let plugin_name =
                                                plugin.file_name().to_string_lossy().to_string();
                                            // Check if any version directory exists with plugin.json
                                            if let Ok(versions) = fs::read_dir(plugin.path()) {
                                                for version in versions.flatten() {
                                                    let plugin_json = version
                                                        .path()
                                                        .join(".claude-plugin/plugin.json");
                                                    if plugin_json.exists() {
                                                        installed.insert(
                                                            plugin_name.clone(),
                                                            ExtensionType::Plugin,
                                                        );
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(installed)
    }

    /// Get the Codex plugins directory (for hook-based plugins)
    fn codex_plugins_dir(&self) -> PathBuf {
        let home = dirs::home_dir().expect("Cannot find home directory");
        home.join(".codex/plugins")
    }

    /// Get the Codex hooks.json file path
    fn codex_hooks_file(&self) -> PathBuf {
        let home = dirs::home_dir().expect("Cannot find home directory");
        home.join(".codex/hooks.json")
    }

    /// Get the Codex config.toml file path
    fn codex_config_file(&self) -> PathBuf {
        let home = dirs::home_dir().expect("Cannot find home directory");
        home.join(".codex/config.toml")
    }

    /// Install an extension from GitHub
    pub fn install(&self, ext: &Extension) -> Result<()> {
        // Handle embedded extensions first (content generated by executor)
        if ext.is_embedded {
            return self.install_embedded(ext);
        }

        // Gemini has completely different extension format
        if self.cli == CliType::Gemini {
            return self.install_for_gemini(ext);
        }

        // Codex with hooks → install plugin with hook conversion
        if self.cli == CliType::Codex && ext.has_hooks {
            return self.install_plugin_for_codex(ext);
        }

        // Check if this plugin requires full marketplace structure (Claude only)
        if self.cli == CliType::Claude && ext.marketplace_name.is_some() {
            return self.install_marketplace_plugin(ext);
        }

        // Claude/Codex installation logic
        // For Codex: extract skill or convert command
        let install_as_skill_from_subpath =
            self.cli == CliType::Codex && ext.skill_subpath.is_some();
        let install_as_skill_from_command =
            self.cli == CliType::Codex && ext.command_file.is_some() && ext.skill_subpath.is_none();

        let (install_type, dest_name) = if install_as_skill_from_subpath {
            // For Codex: extract skill subdirectory to skills folder
            let skill_name = ext
                .skill_subpath
                .unwrap()
                .split('/')
                .next_back()
                .unwrap_or(ext.name);
            (ExtensionType::Skill, skill_name)
        } else if install_as_skill_from_command {
            // For Codex: convert command to skill
            (ExtensionType::Skill, ext.name)
        } else {
            // For Claude: install as plugin
            (ext.extension_type, ext.name)
        };

        let dest = self.install_dir(install_type).join(dest_name);

        // Create parent directory if needed
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|err| OperationError::Io {
                path: parent.display().to_string(),
                source: err,
            })?;
        }

        if install_as_skill_from_command {
            // For Codex with command_file: download command and convert to SKILL.md
            self.install_from_command(ext, &dest)?;
        } else {
            // Determine source path
            let source_path = if install_as_skill_from_subpath {
                // For Codex: use skill_subpath combined with source_path
                format!("{}/{}", ext.source_path, ext.skill_subpath.unwrap())
            } else {
                ext.source_path.to_string()
            };

            // Download and extract
            self.download_and_extract(ext.source_repo, &source_path, &dest)?;

            // Convert SKILL.md format for target CLI (for skill installations)
            if install_as_skill_from_subpath || ext.extension_type == ExtensionType::Skill {
                self.convert_skill_for_cli(&dest)?;
            }
        }

        Ok(())
    }

    /// Install a plugin that requires full marketplace structure (Claude only)
    /// This handles plugins like claude-mem that have scripts referencing the marketplace root
    fn install_marketplace_plugin(&self, ext: &Extension) -> Result<()> {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let marketplace_name = ext.marketplace_name.unwrap();
        let plugin_path = ext.marketplace_plugin_path.unwrap_or(".");
        let version = ext.version.unwrap_or("1.0.0");

        // 1. Clone the full repo to marketplaces directory
        let marketplaces_dir = home.join(".claude/plugins/marketplaces");
        let marketplace_dir = marketplaces_dir.join(marketplace_name);

        fs::create_dir_all(&marketplaces_dir).map_err(|err| OperationError::Io {
            path: marketplaces_dir.display().to_string(),
            source: err,
        })?;

        // Remove existing marketplace directory if it exists
        if marketplace_dir.exists() {
            fs::remove_dir_all(&marketplace_dir).map_err(|err| OperationError::Io {
                path: marketplace_dir.display().to_string(),
                source: err,
            })?;
        }

        // Git clone the repository
        let repo_url = format!("https://github.com/{}.git", ext.source_repo);
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &repo_url,
                marketplace_dir.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| OperationError::Command {
                command: "git".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "git clone".to_string(),
                message: crate::tr!(
                    keys::SKILL_INSTALLER_DOWNLOAD_FAILED,
                    error = "git clone failed"
                ),
            });
        }

        // 2. Create cache directory and symlink
        let cache_dir = home
            .join(".claude/plugins/cache")
            .join(marketplace_name)
            .join(ext.name);
        fs::create_dir_all(&cache_dir).map_err(|err| OperationError::Io {
            path: cache_dir.display().to_string(),
            source: err,
        })?;

        let version_link = cache_dir.join(version);
        let plugin_source = marketplace_dir.join(plugin_path);

        // Remove existing symlink if it exists
        if version_link.exists() || version_link.is_symlink() {
            fs::remove_file(&version_link)
                .or_else(|_| fs::remove_dir_all(&version_link))
                .ok();
        }

        // Create symlink
        #[cfg(unix)]
        std::os::unix::fs::symlink(&plugin_source, &version_link).map_err(|err| {
            OperationError::Io {
                path: version_link.display().to_string(),
                source: err,
            }
        })?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&plugin_source, &version_link).map_err(|err| {
            OperationError::Io {
                path: version_link.display().to_string(),
                source: err,
            }
        })?;

        // 3. Install dependencies in the plugin directory
        let package_json = plugin_source.join("package.json");
        if package_json.exists() {
            let bun_status = Command::new("bun")
                .args(["install"])
                .current_dir(&plugin_source)
                .status();

            if bun_status.is_err() || !bun_status.unwrap().success() {
                let _ = Command::new("npm")
                    .args(["install", "--silent"])
                    .current_dir(&plugin_source)
                    .status();
            }
        }

        // 4. Update known_marketplaces.json
        self.update_known_marketplaces(marketplace_name, ext.source_repo, &marketplace_dir)?;

        // 5. Update installed_plugins.json
        self.update_installed_plugins(ext.name, marketplace_name, &version_link, version)?;

        // 6. Update settings.json enabledPlugins
        self.update_settings_enabled_plugins(ext.name, marketplace_name, true)?;

        Ok(())
    }

    /// Install a plugin with hooks for Codex CLI
    /// Converts Claude plugin hooks to Codex hooks.json format
    fn install_plugin_for_codex(&self, ext: &Extension) -> Result<()> {
        let plugins_dir = self.codex_plugins_dir();
        let plugin_dir = plugins_dir.join(ext.name);

        // Create plugin directory
        fs::create_dir_all(&plugin_dir).map_err(|err| OperationError::Io {
            path: plugin_dir.display().to_string(),
            source: err,
        })?;

        // Download the full plugin to temp dir
        let temp_dir = tempfile::tempdir().map_err(|err| OperationError::Io {
            path: "tempdir".to_string(),
            source: err,
        })?;
        let temp_plugin = temp_dir.path().join("plugin");
        self.download_and_extract(ext.source_repo, ext.source_path, &temp_plugin)?;

        // Find hooks directory in the downloaded plugin
        let hooks_source = self.find_hooks_dir(&temp_plugin);

        if let Some(hooks_dir) = hooks_source {
            // Copy hook scripts to plugin directory
            let dest_hooks = plugin_dir.join("hooks");
            self.copy_dir_recursive(&hooks_dir, &dest_hooks)?;

            // Replace ${CLAUDE_PLUGIN_ROOT} with actual plugin path
            self.replace_plugin_root_variable(&dest_hooks, &plugin_dir)?;

            // Generate and merge hooks.json entries
            self.update_codex_hooks_json(ext.name, &dest_hooks)?;
        }

        // Enable hooks feature in config.toml
        self.enable_codex_hooks_feature()?;

        Ok(())
    }

    /// Find the hooks directory within a Claude plugin
    /// Looks for .claude-plugin/hooks/ or hooks/ at the top level
    fn find_hooks_dir(&self, plugin_dir: &Path) -> Option<PathBuf> {
        let claude_plugin_hooks = plugin_dir.join(".claude-plugin/hooks");
        if claude_plugin_hooks.exists() {
            return Some(claude_plugin_hooks);
        }
        let top_level_hooks = plugin_dir.join("hooks");
        if top_level_hooks.exists() {
            return Some(top_level_hooks);
        }
        None
    }

    /// Replace ${CLAUDE_PLUGIN_ROOT} with the actual plugin path in all text files
    fn replace_plugin_root_variable(&self, dir: &Path, plugin_dir: &Path) -> Result<()> {
        let plugin_path_str = plugin_dir.display().to_string();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.replace_plugin_root_variable(&path, plugin_dir)?;
                } else if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if matches!(ext_str.as_ref(), "js" | "json" | "sh" | "md" | "toml") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if content.contains("${CLAUDE_PLUGIN_ROOT}") {
                                let converted =
                                    content.replace("${CLAUDE_PLUGIN_ROOT}", &plugin_path_str);
                                let _ = fs::write(&path, converted);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Update ~/.codex/hooks.json with hook entries from a plugin
    fn update_codex_hooks_json(&self, plugin_name: &str, hooks_dir: &Path) -> Result<()> {
        let hooks_file = self.codex_hooks_file();

        // Read existing hooks.json or create new
        let mut hooks_config: serde_json::Value = if hooks_file.exists() {
            let content = fs::read_to_string(&hooks_file).map_err(|err| OperationError::Io {
                path: hooks_file.display().to_string(),
                source: err,
            })?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({"hooks": {}}))
        } else {
            serde_json::json!({"hooks": {}})
        };

        // Ensure hooks object exists
        if hooks_config.get("hooks").is_none() {
            hooks_config["hooks"] = serde_json::json!({});
        }

        // Supported Codex hook events that map from Claude events
        let codex_events = ["PreToolUse", "PostToolUse", "Stop", "SessionStart"];

        // Walk the hooks directory and register scripts
        if let Ok(event_dirs) = fs::read_dir(hooks_dir) {
            for event_entry in event_dirs.flatten() {
                if !event_entry
                    .file_type()
                    .map(|ft| ft.is_dir())
                    .unwrap_or(false)
                {
                    continue;
                }
                let event_name = event_entry.file_name().to_string_lossy().to_string();

                // Only convert events that Codex supports
                if !codex_events.contains(&event_name.as_str()) {
                    continue;
                }

                // Collect all scripts for this event
                if let Ok(scripts) = fs::read_dir(event_entry.path()) {
                    let mut hook_commands: Vec<serde_json::Value> = Vec::new();

                    for script in scripts.flatten() {
                        let script_path = script.path();
                        if script_path.is_file() {
                            let command = format!("node {}", script_path.display());
                            hook_commands.push(serde_json::json!({
                                "type": "command",
                                "command": command,
                                "timeout": 600
                            }));
                        }
                    }

                    if !hook_commands.is_empty() {
                        // Determine appropriate matcher
                        let matcher = match event_name.as_str() {
                            "PreToolUse" | "PostToolUse" => "*",
                            _ => "",
                        };

                        let new_entry = serde_json::json!({
                            "matcher": matcher,
                            "hooks": hook_commands
                        });

                        // Get or create the event array
                        let event_array = hooks_config["hooks"]
                            .get_mut(&event_name)
                            .and_then(|v| v.as_array_mut());

                        if let Some(arr) = event_array {
                            // Remove any existing entries for this plugin (by command path)
                            let plugin_prefix = format!(
                                "{}/{}/hooks/",
                                self.codex_plugins_dir().display(),
                                plugin_name
                            );
                            arr.retain(|entry| !entry_contains_plugin_path(entry, &plugin_prefix));
                            arr.push(new_entry);
                        } else {
                            hooks_config["hooks"][&event_name] = serde_json::json!([new_entry]);
                        }
                    }
                }
            }
        }

        // Write hooks.json
        if let Some(parent) = hooks_file.parent() {
            fs::create_dir_all(parent).map_err(|err| OperationError::Io {
                path: parent.display().to_string(),
                source: err,
            })?;
        }
        let content = serde_json::to_string_pretty(&hooks_config).unwrap_or_default();
        fs::write(&hooks_file, content).map_err(|err| OperationError::Io {
            path: hooks_file.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Enable the codex_hooks feature in ~/.codex/config.toml
    fn enable_codex_hooks_feature(&self) -> Result<()> {
        let config_file = self.codex_config_file();

        let mut content = if config_file.exists() {
            fs::read_to_string(&config_file).unwrap_or_default()
        } else {
            String::new()
        };

        // Check if codex_hooks is already enabled
        if content.contains("codex_hooks") {
            // Update existing entry
            let re = regex::Regex::new(r"codex_hooks\s*=\s*\w+").unwrap();
            content = re.replace(&content, "codex_hooks = true").to_string();
        } else if content.contains("[features]") {
            // Add under existing [features] section
            content = content.replace("[features]", "[features]\ncodex_hooks = true");
        } else {
            // Add new [features] section
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("\n[features]\ncodex_hooks = true\n");
        }

        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent).map_err(|err| OperationError::Io {
                path: parent.display().to_string(),
                source: err,
            })?;
        }
        fs::write(&config_file, content).map_err(|err| OperationError::Io {
            path: config_file.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Remove a Codex plugin's hooks from hooks.json
    fn remove_codex_plugin_hooks(&self, plugin_name: &str) -> Result<()> {
        let hooks_file = self.codex_hooks_file();
        if !hooks_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&hooks_file).map_err(|err| OperationError::Io {
            path: hooks_file.display().to_string(),
            source: err,
        })?;

        let mut hooks_config: serde_json::Value =
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({"hooks": {}}));

        let plugin_prefix = format!(
            "{}/{}/hooks/",
            self.codex_plugins_dir().display(),
            plugin_name
        );

        // Remove entries matching this plugin from all event arrays
        if let Some(hooks) = hooks_config
            .get_mut("hooks")
            .and_then(|h| h.as_object_mut())
        {
            for (_event, entries) in hooks.iter_mut() {
                if let Some(arr) = entries.as_array_mut() {
                    arr.retain(|entry| !entry_contains_plugin_path(entry, &plugin_prefix));
                }
            }
            // Remove empty event arrays
            hooks.retain(|_, v| !v.as_array().map(|a| a.is_empty()).unwrap_or(false));
        }

        let content = serde_json::to_string_pretty(&hooks_config).unwrap_or_default();
        fs::write(&hooks_file, content).map_err(|err| OperationError::Io {
            path: hooks_file.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Update known_marketplaces.json for marketplace-based plugins
    fn update_known_marketplaces(
        &self,
        marketplace_name: &str,
        source_repo: &str,
        install_location: &Path,
    ) -> Result<()> {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let file_path = home.join(".claude/plugins/known_marketplaces.json");

        // Read existing content or create new
        let mut marketplaces: serde_json::Value = if file_path.exists() {
            let content = fs::read_to_string(&file_path).map_err(|err| OperationError::Io {
                path: file_path.display().to_string(),
                source: err,
            })?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Add/update marketplace entry
        let now = chrono::Utc::now().to_rfc3339();
        marketplaces[marketplace_name] = serde_json::json!({
            "source": {
                "source": "github",
                "repo": source_repo
            },
            "installLocation": install_location.display().to_string(),
            "lastUpdated": now
        });

        // Write back
        let content = serde_json::to_string_pretty(&marketplaces).unwrap_or_default();
        fs::write(&file_path, content).map_err(|err| OperationError::Io {
            path: file_path.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Update installed_plugins.json for marketplace-based plugins
    fn update_installed_plugins(
        &self,
        plugin_name: &str,
        marketplace_name: &str,
        install_path: &Path,
        version: &str,
    ) -> Result<()> {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let file_path = home.join(".claude/plugins/installed_plugins.json");

        // Read existing content or create new
        let mut installed: serde_json::Value = if file_path.exists() {
            let content = fs::read_to_string(&file_path).map_err(|err| OperationError::Io {
                path: file_path.display().to_string(),
                source: err,
            })?;
            serde_json::from_str(&content).unwrap_or_else(|_| {
                serde_json::json!({
                    "version": 2,
                    "plugins": {}
                })
            })
        } else {
            serde_json::json!({
                "version": 2,
                "plugins": {}
            })
        };

        // Create plugin key
        let plugin_key = format!("{}@{}", plugin_name, marketplace_name);
        let now = chrono::Utc::now().to_rfc3339();

        // Add/update plugin entry
        installed["plugins"][&plugin_key] = serde_json::json!([{
            "scope": "user",
            "installPath": install_path.display().to_string(),
            "version": version,
            "installedAt": now,
            "lastUpdated": now,
            "isLocal": true
        }]);

        // Write back
        let content = serde_json::to_string_pretty(&installed).unwrap_or_default();
        fs::write(&file_path, content).map_err(|err| OperationError::Io {
            path: file_path.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Install extension for Gemini CLI (uses extension format, not skill format)
    fn install_for_gemini(&self, ext: &Extension) -> Result<()> {
        let dest = self.install_dir(ExtensionType::Skill).join(ext.name);

        // Create extension directory
        fs::create_dir_all(&dest).map_err(|err| OperationError::Io {
            path: dest.display().to_string(),
            source: err,
        })?;

        // Check if this is a marketplace-based plugin requiring special handling
        if ext.marketplace_name.is_some() {
            return self.install_marketplace_plugin_for_gemini(ext, &dest);
        }

        // Determine what to download based on extension configuration
        if ext.has_hooks {
            // For plugins with hooks: download full plugin and convert to Gemini format
            self.install_plugin_for_gemini(ext, &dest)?;
        } else if ext.skill_subpath.is_some() {
            // For plugins with skills/: extract skill and convert to Gemini extension
            self.install_skill_for_gemini(ext, &dest)?;
        } else if ext.command_file.is_some() {
            // For plugins with commands/: convert command to Gemini extension
            self.install_command_for_gemini(ext, &dest)?;
        } else {
            // Fallback: download as-is
            self.download_and_extract(ext.source_repo, ext.source_path, &dest)?;
        }

        // Register in extension-enablement.json
        self.register_gemini_extension(ext.name)?;

        Ok(())
    }

    /// Install a plugin with hooks for Gemini
    fn install_plugin_for_gemini(&self, ext: &Extension, dest: &Path) -> Result<()> {
        // Download the full plugin
        self.download_and_extract(ext.source_repo, ext.source_path, dest)?;

        // Create gemini-extension.json manifest
        let manifest = format!(
            r#"{{
  "name": "{}",
  "version": "1.0.0",
  "contextFileName": "GEMINI.md"
}}"#,
            ext.name
        );
        let manifest_path = dest.join("gemini-extension.json");
        fs::write(&manifest_path, manifest).map_err(|err| OperationError::Io {
            path: manifest_path.display().to_string(),
            source: err,
        })?;

        // Create GEMINI.md from plugin description or README
        let gemini_md_path = dest.join("GEMINI.md");
        if !gemini_md_path.exists() {
            let readme_path = dest.join("README.md");
            if readme_path.exists() {
                fs::copy(&readme_path, &gemini_md_path).map_err(|err| OperationError::Io {
                    path: readme_path.display().to_string(),
                    source: err,
                })?;
            } else {
                let content = format!("# {}\n\nExtension for Gemini CLI.", ext.name);
                fs::write(&gemini_md_path, content).map_err(|err| OperationError::Io {
                    path: gemini_md_path.display().to_string(),
                    source: err,
                })?;
            }
        }

        // Convert commands/ from .md to .toml format if they exist
        let commands_dir = dest.join("commands");
        if commands_dir.exists() {
            self.convert_commands_to_toml(&commands_dir)?;
        }

        Ok(())
    }

    /// Install a marketplace-based plugin for Gemini
    /// This handles plugins like claude-mem that need full repo clone and dependency installation
    fn install_marketplace_plugin_for_gemini(&self, ext: &Extension, dest: &Path) -> Result<()> {
        let plugin_path = ext.marketplace_plugin_path.unwrap_or(".");

        // 1. Git clone the full repo to a temp directory
        let temp_dir = tempfile::tempdir().map_err(|err| OperationError::Io {
            path: "tempdir".to_string(),
            source: err,
        })?;
        let repo_dir = temp_dir.path().join("repo");

        let repo_url = format!("https://github.com/{}.git", ext.source_repo);
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &repo_url,
                repo_dir.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| OperationError::Command {
                command: "git".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "git clone".to_string(),
                message: crate::tr!(
                    keys::SKILL_INSTALLER_DOWNLOAD_FAILED,
                    error = "git clone failed"
                ),
            });
        }

        // 2. Copy the plugin directory to destination
        let plugin_source = repo_dir.join(plugin_path);
        self.copy_dir_recursive(&plugin_source, dest)?;

        // 3. Run installation script if it exists (install dependencies)
        let install_script = dest.join("scripts/smart-install.js");
        if install_script.exists() {
            // We need to also copy the root package.json for smart-install.js
            let root_package_json = repo_dir.join("package.json");
            if root_package_json.exists() {
                // Create a parent directory structure for the install script
                let parent_dir = dest.parent().unwrap().join(format!("{}-root", ext.name));
                fs::create_dir_all(&parent_dir).ok();
                fs::copy(&root_package_json, parent_dir.join("package.json")).ok();

                // Run smart-install from the plugin directory
                let _ = Command::new("node")
                    .arg(&install_script)
                    .current_dir(dest)
                    .env("GEMINI_PLUGIN_ROOT", dest.to_str().unwrap())
                    .status();
            }
        }

        // 4. Install npm dependencies if package.json exists in plugin
        let package_json = dest.join("package.json");
        if package_json.exists() {
            // Try bun first, fall back to npm
            let bun_status = Command::new("bun")
                .args(["install"])
                .current_dir(dest)
                .status();

            if bun_status.is_err() || !bun_status.unwrap().success() {
                let _ = Command::new("npm")
                    .args(["install", "--silent"])
                    .current_dir(dest)
                    .status();
            }
        }

        // 5. Convert hooks.json - replace ${CLAUDE_PLUGIN_ROOT} with actual path
        let hooks_json = dest.join("hooks/hooks.json");
        if hooks_json.exists() {
            if let Ok(content) = fs::read_to_string(&hooks_json) {
                let converted =
                    content.replace("${CLAUDE_PLUGIN_ROOT}", dest.to_str().unwrap_or(""));
                let _ = fs::write(&hooks_json, converted);
            }
        }

        // 6. Create gemini-extension.json manifest
        let version = ext.version.unwrap_or("1.0.0");
        let manifest = format!(
            r#"{{
  "name": "{}",
  "version": "{}",
  "contextFileName": "GEMINI.md"
}}"#,
            ext.name, version
        );
        let manifest_path = dest.join("gemini-extension.json");
        fs::write(&manifest_path, manifest).map_err(|err| OperationError::Io {
            path: manifest_path.display().to_string(),
            source: err,
        })?;

        // 7. Create GEMINI.md from README or plugin description
        let gemini_md_path = dest.join("GEMINI.md");
        if !gemini_md_path.exists() {
            let readme_path = dest.join("README.md");
            let claude_md_path = dest.join("CLAUDE.md");
            if readme_path.exists() {
                fs::copy(&readme_path, &gemini_md_path).ok();
            } else if claude_md_path.exists() {
                fs::copy(&claude_md_path, &gemini_md_path).ok();
            } else {
                let content = format!("# {}\n\nExtension for Gemini CLI.", ext.name);
                fs::write(&gemini_md_path, content).ok();
            }
        }

        // 8. Convert commands/ to TOML format if they exist
        let commands_dir = dest.join("commands");
        if commands_dir.exists() {
            self.convert_commands_to_toml(&commands_dir)?;
        }

        // Register in extension-enablement.json
        self.register_gemini_extension(ext.name)?;

        Ok(())
    }

    /// Install a skill for Gemini (from skill_subpath)
    fn install_skill_for_gemini(&self, ext: &Extension, dest: &Path) -> Result<()> {
        let skill_subpath = ext.skill_subpath.unwrap();
        let source_path = format!("{}/{}", ext.source_path, skill_subpath);

        // Download the skill subdirectory to a temp location
        let temp_dir = tempfile::tempdir().map_err(|err| OperationError::Io {
            path: "tempdir".to_string(),
            source: err,
        })?;
        let temp_skill = temp_dir.path().join("skill");
        self.download_and_extract(ext.source_repo, &source_path, &temp_skill)?;

        // Read SKILL.md
        let skill_md_path = temp_skill.join("SKILL.md");
        let skill_content = if skill_md_path.exists() {
            fs::read_to_string(&skill_md_path).map_err(|err| OperationError::Io {
                path: skill_md_path.display().to_string(),
                source: err,
            })?
        } else {
            String::new()
        };

        let (frontmatter, body) = self.parse_skill_md(&skill_content);
        let description = frontmatter
            .get("description")
            .cloned()
            .unwrap_or_else(|| format!("{} extension", ext.name));

        // Create gemini-extension.json
        let manifest = format!(
            r#"{{
  "name": "{}",
  "version": "1.0.0",
  "contextFileName": "GEMINI.md"
}}"#,
            ext.name
        );
        fs::write(dest.join("gemini-extension.json"), manifest).map_err(|err| {
            OperationError::Io {
                path: dest.join("gemini-extension.json").display().to_string(),
                source: err,
            }
        })?;

        // Create GEMINI.md with skill content
        let gemini_md = format!("# {}\n\n{}\n\n{}", ext.name, description, body);
        fs::write(dest.join("GEMINI.md"), gemini_md).map_err(|err| OperationError::Io {
            path: dest.join("GEMINI.md").display().to_string(),
            source: err,
        })?;

        // Create commands/<name>/invoke.toml
        let commands_dir = dest.join("commands").join(ext.name);
        fs::create_dir_all(&commands_dir).map_err(|err| OperationError::Io {
            path: commands_dir.display().to_string(),
            source: err,
        })?;

        let invoke_toml = format!(
            r#"description = "{}"
prompt = """
{}
"""
"#,
            description.lines().next().unwrap_or(&description),
            body.trim()
        );
        fs::write(commands_dir.join("invoke.toml"), invoke_toml).map_err(|err| {
            OperationError::Io {
                path: commands_dir.join("invoke.toml").display().to_string(),
                source: err,
            }
        })?;

        Ok(())
    }

    /// Install a command for Gemini (from command_file)
    fn install_command_for_gemini(&self, ext: &Extension, dest: &Path) -> Result<()> {
        let command_file = ext.command_file.unwrap();

        // Download the command file
        let url = format!(
            "https://github.com/{}/archive/refs/heads/main.tar.gz",
            ext.source_repo
        );

        let temp_dir = tempfile::tempdir().map_err(|err| OperationError::Io {
            path: "tempdir".to_string(),
            source: err,
        })?;
        let archive = temp_dir.path().join("archive.tar.gz");

        // Download
        let status = Command::new("curl")
            .args(["-L", "-s", "-o", archive.to_str().unwrap(), &url])
            .status()
            .map_err(|e| OperationError::Command {
                command: "curl".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "curl".to_string(),
                message: crate::tr!(keys::SKILL_INSTALLER_DOWNLOAD_FAILED, error = "curl failed"),
            });
        }

        // Extract command file
        let repo_name = ext
            .source_repo
            .split('/')
            .next_back()
            .unwrap_or(ext.source_repo);
        let extract_path = format!("{}-main/{}/{}", repo_name, ext.source_path, command_file);

        let status = Command::new("tar")
            .args([
                "-xzf",
                archive.to_str().unwrap(),
                "-C",
                temp_dir.path().to_str().unwrap(),
                &extract_path,
            ])
            .status()
            .map_err(|e| OperationError::Command {
                command: "tar".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "tar".to_string(),
                message: crate::tr!(keys::SKILL_INSTALLER_EXTRACT_FAILED, error = "tar failed"),
            });
        }

        // Read command content
        let command_path = temp_dir.path().join(&extract_path);
        let command_content =
            fs::read_to_string(&command_path).map_err(|err| OperationError::Io {
                path: command_path.display().to_string(),
                source: err,
            })?;

        let (frontmatter, body) = self.parse_skill_md(&command_content);
        let description = frontmatter
            .get("description")
            .cloned()
            .unwrap_or_else(|| format!("{} command", ext.name));

        // Create gemini-extension.json
        let manifest = format!(
            r#"{{
  "name": "{}",
  "version": "1.0.0",
  "contextFileName": "GEMINI.md"
}}"#,
            ext.name
        );
        fs::write(dest.join("gemini-extension.json"), manifest).map_err(|err| {
            OperationError::Io {
                path: dest.join("gemini-extension.json").display().to_string(),
                source: err,
            }
        })?;

        // Create GEMINI.md
        let gemini_md = format!("# {}\n\n{}", ext.name, description);
        fs::write(dest.join("GEMINI.md"), gemini_md).map_err(|err| OperationError::Io {
            path: dest.join("GEMINI.md").display().to_string(),
            source: err,
        })?;

        // Create commands/<name>/invoke.toml
        let commands_dir = dest.join("commands").join(ext.name);
        fs::create_dir_all(&commands_dir).map_err(|err| OperationError::Io {
            path: commands_dir.display().to_string(),
            source: err,
        })?;

        let invoke_toml = format!(
            r#"description = "{}"
prompt = """
{}
"""
"#,
            description.lines().next().unwrap_or(&description),
            body.trim()
        );
        fs::write(commands_dir.join("invoke.toml"), invoke_toml).map_err(|err| {
            OperationError::Io {
                path: commands_dir.join("invoke.toml").display().to_string(),
                source: err,
            }
        })?;

        Ok(())
    }

    /// Convert markdown command files to TOML format for Gemini
    fn convert_commands_to_toml(&self, commands_dir: &Path) -> Result<()> {
        if let Ok(entries) = fs::read_dir(commands_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    // Read markdown command
                    let content = fs::read_to_string(&path).map_err(|err| OperationError::Io {
                        path: path.display().to_string(),
                        source: err,
                    })?;

                    let (frontmatter, body) = self.parse_skill_md(&content);
                    let description = frontmatter
                        .get("description")
                        .cloned()
                        .unwrap_or_else(|| "Command".to_string());

                    // Create TOML version
                    let toml_content = format!(
                        r#"description = "{}"
prompt = """
{}
"""
"#,
                        description.lines().next().unwrap_or(&description),
                        body.trim()
                    );

                    // Write as .toml with same name
                    let toml_path = path.with_extension("toml");
                    fs::write(&toml_path, toml_content).map_err(|err| OperationError::Io {
                        path: toml_path.display().to_string(),
                        source: err,
                    })?;

                    // Remove original .md file
                    fs::remove_file(&path).map_err(|err| OperationError::Io {
                        path: path.display().to_string(),
                        source: err,
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Register extension in Gemini's extension-enablement.json
    fn register_gemini_extension(&self, name: &str) -> Result<()> {
        let enablement_path = self.gemini_enablement_file();

        // Read existing content or create new
        let mut enablement: serde_json::Value = if enablement_path.exists() {
            let content =
                fs::read_to_string(&enablement_path).map_err(|err| OperationError::Io {
                    path: enablement_path.display().to_string(),
                    source: err,
                })?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Add/update extension entry with wildcard override
        let home = dirs::home_dir().expect("Cannot find home directory");
        let override_path = format!("{}/*", home.display());
        enablement[name] = serde_json::json!({
            "overrides": [override_path]
        });

        // Write back
        let content = serde_json::to_string_pretty(&enablement).unwrap_or_default();
        fs::write(&enablement_path, content).map_err(|err| OperationError::Io {
            path: enablement_path.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Unregister extension from Gemini's extension-enablement.json
    fn unregister_gemini_extension(&self, name: &str) -> Result<()> {
        let enablement_path = self.gemini_enablement_file();

        if !enablement_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&enablement_path).map_err(|err| OperationError::Io {
            path: enablement_path.display().to_string(),
            source: err,
        })?;

        let mut enablement: serde_json::Value =
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));

        // Remove extension entry
        if let Some(obj) = enablement.as_object_mut() {
            obj.remove(name);
        }

        // Write back
        let content = serde_json::to_string_pretty(&enablement).unwrap_or_default();
        fs::write(&enablement_path, content).map_err(|err| OperationError::Io {
            path: enablement_path.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Install extension by converting command file to SKILL.md
    fn install_from_command(&self, ext: &Extension, dest: &Path) -> Result<()> {
        let command_file = ext.command_file.unwrap();
        let url = format!(
            "https://github.com/{}/archive/refs/heads/main.tar.gz",
            ext.source_repo
        );

        // Create temporary directory
        let temp_dir = tempfile::tempdir().map_err(|err| OperationError::Io {
            path: "tempdir".to_string(),
            source: err,
        })?;

        let archive = temp_dir.path().join("archive.tar.gz");

        // Download using curl
        let status = Command::new("curl")
            .args(["-L", "-s", "-o", archive.to_str().unwrap(), &url])
            .status()
            .map_err(|e| OperationError::Command {
                command: "curl".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "curl".to_string(),
                message: crate::tr!(keys::SKILL_INSTALLER_DOWNLOAD_FAILED, error = "curl failed"),
            });
        }

        // Extract the command file from the archive
        let repo_name = ext
            .source_repo
            .split('/')
            .next_back()
            .unwrap_or(ext.source_repo);
        let extract_path = format!("{}-main/{}/{}", repo_name, ext.source_path, command_file);

        let status = Command::new("tar")
            .args([
                "-xzf",
                archive.to_str().unwrap(),
                "-C",
                temp_dir.path().to_str().unwrap(),
                &extract_path,
            ])
            .status()
            .map_err(|e| OperationError::Command {
                command: "tar".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "tar".to_string(),
                message: crate::tr!(keys::SKILL_INSTALLER_EXTRACT_FAILED, error = "tar failed"),
            });
        }

        // Read the command file
        let command_path = temp_dir.path().join(&extract_path);
        let command_content =
            fs::read_to_string(&command_path).map_err(|err| OperationError::Io {
                path: command_path.display().to_string(),
                source: err,
            })?;

        // Convert command to SKILL.md format
        let skill_content = self.convert_command_to_skill(ext.name, &command_content);

        // Create destination directory
        fs::create_dir_all(dest).map_err(|err| OperationError::Io {
            path: dest.display().to_string(),
            source: err,
        })?;

        // Write SKILL.md
        let skill_md = dest.join("SKILL.md");
        fs::write(&skill_md, skill_content).map_err(|err| OperationError::Io {
            path: skill_md.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Convert a Claude command markdown file to SKILL.md format
    fn convert_command_to_skill(&self, name: &str, content: &str) -> String {
        let (frontmatter, body) = self.parse_skill_md(content);

        // Get description from frontmatter
        let description = frontmatter
            .get("description")
            .cloned()
            .unwrap_or_else(|| format!("{} skill", name));

        // Format name based on target CLI
        let formatted_name = match self.cli {
            CliType::Claude => name.to_string(),
            CliType::Codex => name.to_string(),
            CliType::Gemini => {
                let normalized = name.to_lowercase().replace(' ', "-");
                if normalized.len() > 64 {
                    normalized[..64].to_string()
                } else {
                    normalized
                }
            }
        };

        // Format description based on target CLI (Codex needs single line)
        let formatted_desc = match self.cli {
            CliType::Codex => description
                .lines()
                .next()
                .unwrap_or(&description)
                .to_string(),
            _ => description,
        };

        // Build SKILL.md content (without Claude-specific fields like allowed-tools)
        format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}",
            formatted_name, formatted_desc, body
        )
    }

    /// Remove an installed extension
    pub fn remove(&self, ext: &Extension) -> Result<()> {
        // Handle embedded extensions
        if ext.is_embedded {
            return self.remove_embedded(ext);
        }

        if self.cli == CliType::Gemini {
            // Gemini: remove from extensions/ directory and unregister
            let dest = self.install_dir(ExtensionType::Skill).join(ext.name);
            if dest.exists() {
                fs::remove_dir_all(&dest).map_err(|err| OperationError::Io {
                    path: dest.display().to_string(),
                    source: err,
                })?;
            }
            self.unregister_gemini_extension(ext.name)?;
            return Ok(());
        }

        // Codex hook-based plugins
        if self.cli == CliType::Codex && ext.has_hooks {
            let plugin_dir = self.codex_plugins_dir().join(ext.name);
            if plugin_dir.exists() {
                fs::remove_dir_all(&plugin_dir).map_err(|err| OperationError::Io {
                    path: plugin_dir.display().to_string(),
                    source: err,
                })?;
            }
            self.remove_codex_plugin_hooks(ext.name)?;
            return Ok(());
        }

        // Check if this is a marketplace-based plugin (Claude only)
        if self.cli == CliType::Claude && ext.marketplace_name.is_some() {
            return self.remove_marketplace_plugin(ext);
        }

        // Claude/Codex removal logic
        let installed_as_skill_from_subpath =
            self.cli == CliType::Codex && ext.skill_subpath.is_some();
        let installed_as_skill_from_command =
            self.cli == CliType::Codex && ext.command_file.is_some() && ext.skill_subpath.is_none();

        let (install_type, dest_name) = if installed_as_skill_from_subpath {
            let skill_name = ext
                .skill_subpath
                .unwrap()
                .split('/')
                .next_back()
                .unwrap_or(ext.name);
            (ExtensionType::Skill, skill_name)
        } else if installed_as_skill_from_command {
            (ExtensionType::Skill, ext.name)
        } else {
            // Claude: installed as plugin
            (ext.extension_type, ext.name)
        };

        let dest = self.install_dir(install_type).join(dest_name);

        if dest.exists() {
            fs::remove_dir_all(&dest).map_err(|err| OperationError::Io {
                path: dest.display().to_string(),
                source: err,
            })?;
        }

        Ok(())
    }

    /// Install an embedded extension (content generated by executor)
    fn install_embedded(&self, ext: &Extension) -> Result<()> {
        match self.cli {
            CliType::Claude | CliType::Codex => self.install_embedded_skill(ext),
            CliType::Gemini => self.install_embedded_for_gemini(ext),
        }
    }

    /// Install an embedded extension as SKILL.md for Claude/Codex
    fn install_embedded_skill(&self, ext: &Extension) -> Result<()> {
        let dest = self.install_dir(ExtensionType::Skill).join(ext.name);
        fs::create_dir_all(&dest).map_err(|err| OperationError::Io {
            path: dest.display().to_string(),
            source: err,
        })?;

        // Generate SKILL.md content based on extension name and CLI
        let skill_content = self.generate_embedded_content(ext.name);
        let skill_md = dest.join("SKILL.md");
        fs::write(&skill_md, skill_content).map_err(|err| OperationError::Io {
            path: skill_md.display().to_string(),
            source: err,
        })?;

        // For Codex loop-runner, also install SessionStart hook
        if self.cli == CliType::Codex && ext.name == "loop-runner" {
            self.install_loop_runner_hook()?;
        }

        Ok(())
    }

    /// Install an embedded extension as Gemini TOML extension
    fn install_embedded_for_gemini(&self, ext: &Extension) -> Result<()> {
        let dest = self.install_dir(ExtensionType::Skill).join(ext.name);
        fs::create_dir_all(&dest).map_err(|err| OperationError::Io {
            path: dest.display().to_string(),
            source: err,
        })?;

        let content = self.generate_embedded_content(ext.name);

        // Create gemini-extension.json
        let manifest = format!(
            r#"{{
  "name": "{}",
  "version": "1.0.0",
  "contextFileName": "GEMINI.md"
}}"#,
            ext.name
        );
        fs::write(dest.join("gemini-extension.json"), manifest).map_err(|err| {
            OperationError::Io {
                path: dest.join("gemini-extension.json").display().to_string(),
                source: err,
            }
        })?;

        // Create GEMINI.md
        let gemini_md = format!("# {}\n\n{}", ext.name, content);
        fs::write(dest.join("GEMINI.md"), gemini_md).map_err(|err| OperationError::Io {
            path: dest.join("GEMINI.md").display().to_string(),
            source: err,
        })?;

        // Create commands/<name>/invoke.toml
        let commands_dir = dest.join("commands").join(ext.name);
        fs::create_dir_all(&commands_dir).map_err(|err| OperationError::Io {
            path: commands_dir.display().to_string(),
            source: err,
        })?;

        let description = match ext.name {
            "loop-runner" => "Schedule periodic task execution at specified intervals",
            _ => "Embedded extension",
        };
        let invoke_toml = format!(
            r#"description = "{}"
prompt = """
{}
"""
"#,
            description,
            content.trim()
        );
        fs::write(commands_dir.join("invoke.toml"), invoke_toml).map_err(|err| {
            OperationError::Io {
                path: commands_dir.join("invoke.toml").display().to_string(),
                source: err,
            }
        })?;

        self.register_gemini_extension(ext.name)?;
        Ok(())
    }

    /// Generate embedded content for a specific extension and CLI
    fn generate_embedded_content(&self, name: &str) -> String {
        match name {
            "loop-runner" => self.generate_loop_runner_content(),
            _ => format!(
                "---\nname: {}\ndescription: Embedded extension\n---\n",
                name
            ),
        }
    }

    /// Generate loop-runner SKILL.md content based on target CLI
    fn generate_loop_runner_content(&self) -> String {
        match self.cli {
            CliType::Claude => LOOP_RUNNER_CLAUDE.to_string(),
            CliType::Codex => LOOP_RUNNER_CODEX.to_string(),
            CliType::Gemini => LOOP_RUNNER_GEMINI.to_string(),
        }
    }

    /// Install the SessionStart hook for loop-runner on Codex
    fn install_loop_runner_hook(&self) -> Result<()> {
        let plugin_dir = self.codex_plugins_dir().join("loop-runner");
        let hooks_dir = plugin_dir.join("hooks").join("SessionStart");
        fs::create_dir_all(&hooks_dir).map_err(|err| OperationError::Io {
            path: hooks_dir.display().to_string(),
            source: err,
        })?;

        // Write the hook script
        let hook_path = hooks_dir.join("check-loops.js");
        fs::write(&hook_path, LOOP_RUNNER_HOOK_SCRIPT).map_err(|err| OperationError::Io {
            path: hook_path.display().to_string(),
            source: err,
        })?;

        // Register in hooks.json
        let hooks_dir_parent = plugin_dir.join("hooks");
        self.update_codex_hooks_json("loop-runner", &hooks_dir_parent)?;

        // Enable hooks feature
        self.enable_codex_hooks_feature()?;

        Ok(())
    }

    /// Remove an embedded extension
    fn remove_embedded(&self, ext: &Extension) -> Result<()> {
        match self.cli {
            CliType::Gemini => {
                let dest = self.install_dir(ExtensionType::Skill).join(ext.name);
                if dest.exists() {
                    fs::remove_dir_all(&dest).map_err(|err| OperationError::Io {
                        path: dest.display().to_string(),
                        source: err,
                    })?;
                }
                self.unregister_gemini_extension(ext.name)?;
            }
            CliType::Codex => {
                // Remove skill
                let skill_dir = self.install_dir(ExtensionType::Skill).join(ext.name);
                if skill_dir.exists() {
                    fs::remove_dir_all(&skill_dir).map_err(|err| OperationError::Io {
                        path: skill_dir.display().to_string(),
                        source: err,
                    })?;
                }
                // Remove hook plugin directory and hooks.json entries
                let plugin_dir = self.codex_plugins_dir().join(ext.name);
                if plugin_dir.exists() {
                    fs::remove_dir_all(&plugin_dir).map_err(|err| OperationError::Io {
                        path: plugin_dir.display().to_string(),
                        source: err,
                    })?;
                }
                self.remove_codex_plugin_hooks(ext.name)?;
            }
            CliType::Claude => {
                let dest = self.install_dir(ExtensionType::Skill).join(ext.name);
                if dest.exists() {
                    fs::remove_dir_all(&dest).map_err(|err| OperationError::Io {
                        path: dest.display().to_string(),
                        source: err,
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Remove a marketplace-based plugin
    fn remove_marketplace_plugin(&self, ext: &Extension) -> Result<()> {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let marketplace_name = ext.marketplace_name.unwrap();

        // 1. Remove from installed_plugins.json
        self.remove_from_installed_plugins(ext.name, marketplace_name)?;

        // 2. Remove cache directory (symlink and parent)
        let cache_dir = home
            .join(".claude/plugins/cache")
            .join(marketplace_name)
            .join(ext.name);
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir).map_err(|err| OperationError::Io {
                path: cache_dir.display().to_string(),
                source: err,
            })?;
        }

        // 3. Remove marketplace directory
        let marketplace_dir = home
            .join(".claude/plugins/marketplaces")
            .join(marketplace_name);
        if marketplace_dir.exists() {
            fs::remove_dir_all(&marketplace_dir).map_err(|err| OperationError::Io {
                path: marketplace_dir.display().to_string(),
                source: err,
            })?;
        }

        // 4. Remove from known_marketplaces.json
        self.remove_from_known_marketplaces(marketplace_name)?;

        // 5. Remove from settings.json enabledPlugins
        self.update_settings_enabled_plugins(ext.name, marketplace_name, false)?;

        Ok(())
    }

    /// Remove plugin from installed_plugins.json
    fn remove_from_installed_plugins(
        &self,
        plugin_name: &str,
        marketplace_name: &str,
    ) -> Result<()> {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let file_path = home.join(".claude/plugins/installed_plugins.json");

        if !file_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&file_path).map_err(|err| OperationError::Io {
            path: file_path.display().to_string(),
            source: err,
        })?;

        let mut installed: serde_json::Value =
            serde_json::from_str(&content).unwrap_or_else(|_| {
                serde_json::json!({
                    "version": 2,
                    "plugins": {}
                })
            });

        // Remove plugin entry
        let plugin_key = format!("{}@{}", plugin_name, marketplace_name);
        if let Some(plugins) = installed.get_mut("plugins").and_then(|p| p.as_object_mut()) {
            plugins.remove(&plugin_key);
        }

        // Write back
        let content = serde_json::to_string_pretty(&installed).unwrap_or_default();
        fs::write(&file_path, content).map_err(|err| OperationError::Io {
            path: file_path.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Remove marketplace from known_marketplaces.json
    fn remove_from_known_marketplaces(&self, marketplace_name: &str) -> Result<()> {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let file_path = home.join(".claude/plugins/known_marketplaces.json");

        if !file_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&file_path).map_err(|err| OperationError::Io {
            path: file_path.display().to_string(),
            source: err,
        })?;

        let mut marketplaces: serde_json::Value =
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));

        // Remove marketplace entry
        if let Some(obj) = marketplaces.as_object_mut() {
            obj.remove(marketplace_name);
        }

        // Write back
        let content = serde_json::to_string_pretty(&marketplaces).unwrap_or_default();
        fs::write(&file_path, content).map_err(|err| OperationError::Io {
            path: file_path.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Update settings.json enabledPlugins for marketplace-based plugins
    fn update_settings_enabled_plugins(
        &self,
        plugin_name: &str,
        marketplace_name: &str,
        enabled: bool,
    ) -> Result<()> {
        let home = dirs::home_dir().expect("Cannot find home directory");
        let file_path = home.join(".claude/settings.json");

        let mut settings: serde_json::Value = if file_path.exists() {
            let content = fs::read_to_string(&file_path).map_err(|err| OperationError::Io {
                path: file_path.display().to_string(),
                source: err,
            })?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let plugin_key = format!("{}@{}", plugin_name, marketplace_name);

        if enabled {
            // Ensure enabledPlugins object exists
            if settings.get("enabledPlugins").is_none() {
                settings["enabledPlugins"] = serde_json::json!({});
            }
            settings["enabledPlugins"][&plugin_key] = serde_json::json!(true);
        } else if let Some(plugins) = settings
            .get_mut("enabledPlugins")
            .and_then(|p| p.as_object_mut())
        {
            plugins.remove(&plugin_key);
        }

        let content = serde_json::to_string_pretty(&settings).unwrap_or_default();
        fs::write(&file_path, content).map_err(|err| OperationError::Io {
            path: file_path.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Download and extract from GitHub
    fn download_and_extract(&self, repo: &str, path: &str, dest: &Path) -> Result<()> {
        let url = format!("https://github.com/{}/archive/refs/heads/main.tar.gz", repo);

        // Create temporary directory
        let temp_dir = tempfile::tempdir().map_err(|err| OperationError::Io {
            path: "tempdir".to_string(),
            source: err,
        })?;

        let archive = temp_dir.path().join("archive.tar.gz");

        // Download using curl
        let status = Command::new("curl")
            .args(["-L", "-s", "-o", archive.to_str().unwrap(), &url])
            .status()
            .map_err(|e| OperationError::Command {
                command: "curl".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "curl".to_string(),
                message: crate::tr!(keys::SKILL_INSTALLER_DOWNLOAD_FAILED, error = "curl failed"),
            });
        }

        // Extract the specific path from the archive
        let repo_name = repo.split('/').next_back().unwrap_or(repo);
        let extract_path = format!("{}-main/{}", repo_name, path);

        let status = Command::new("tar")
            .args([
                "-xzf",
                archive.to_str().unwrap(),
                "-C",
                temp_dir.path().to_str().unwrap(),
                &extract_path,
            ])
            .status()
            .map_err(|e| OperationError::Command {
                command: "tar".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if !status.success() {
            return Err(OperationError::Command {
                command: "tar".to_string(),
                message: crate::tr!(keys::SKILL_INSTALLER_EXTRACT_FAILED, error = "tar failed"),
            });
        }

        // Move extracted content to destination
        let extracted = temp_dir.path().join(&extract_path);
        if !extracted.exists() {
            return Err(OperationError::Command {
                command: "tar".to_string(),
                message: crate::tr!(
                    keys::SKILL_INSTALLER_EXTRACT_FAILED,
                    error = "extracted path not found"
                ),
            });
        }

        // Remove existing destination if it exists
        if dest.exists() {
            fs::remove_dir_all(dest).map_err(|err| OperationError::Io {
                path: dest.display().to_string(),
                source: err,
            })?;
        }

        // Move using shell command (cross-platform move)
        self.move_directory(&extracted, dest)?;

        Ok(())
    }

    /// Move directory (handles cross-device moves)
    fn move_directory(&self, src: &Path, dest: &Path) -> Result<()> {
        // Try rename first (same filesystem)
        if fs::rename(src, dest).is_ok() {
            return Ok(());
        }

        // Fall back to copy + remove for cross-device moves
        self.copy_dir_recursive(src, dest)?;
        fs::remove_dir_all(src).map_err(|err| OperationError::Io {
            path: src.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Recursively copy a directory
    fn copy_dir_recursive(&self, src: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(dest).map_err(|err| OperationError::Io {
            path: dest.display().to_string(),
            source: err,
        })?;

        for entry in fs::read_dir(src).map_err(|err| OperationError::Io {
            path: src.display().to_string(),
            source: err,
        })? {
            let entry = entry.map_err(|err| OperationError::Io {
                path: src.display().to_string(),
                source: err,
            })?;
            let file_type = entry.file_type().map_err(|err| OperationError::Io {
                path: entry.path().display().to_string(),
                source: err,
            })?;
            let dest_path = dest.join(entry.file_name());

            if file_type.is_dir() {
                self.copy_dir_recursive(&entry.path(), &dest_path)?;
            } else {
                fs::copy(entry.path(), &dest_path).map_err(|err| OperationError::Io {
                    path: entry.path().display().to_string(),
                    source: err,
                })?;
            }
        }

        Ok(())
    }

    /// Convert SKILL.md format based on target CLI
    fn convert_skill_for_cli(&self, skill_dir: &Path) -> Result<()> {
        let skill_md = skill_dir.join("SKILL.md");
        if !skill_md.exists() {
            return Ok(()); // No SKILL.md to convert
        }

        let content = fs::read_to_string(&skill_md).map_err(|err| OperationError::Io {
            path: skill_md.display().to_string(),
            source: err,
        })?;

        let converted = match self.cli {
            CliType::Claude => {
                // Claude supports extended fields, keep as-is
                content
            }
            CliType::Codex => {
                // Codex only recognizes name/description (single line)
                self.format_codex_skill(&content)
            }
            CliType::Gemini => {
                // Gemini: name must be lowercase + dash, max 64 chars
                self.format_gemini_skill(&content)
            }
        };

        fs::write(&skill_md, converted).map_err(|err| OperationError::Io {
            path: skill_md.display().to_string(),
            source: err,
        })?;

        Ok(())
    }

    /// Parse SKILL.md frontmatter and body
    fn parse_skill_md(&self, content: &str) -> (HashMap<String, String>, String) {
        let mut frontmatter = HashMap::new();
        let mut body = String::new();
        let mut in_frontmatter = false;
        let mut frontmatter_done = false;

        for line in content.lines() {
            if line.trim() == "---" {
                if !in_frontmatter && !frontmatter_done {
                    in_frontmatter = true;
                    continue;
                } else if in_frontmatter {
                    in_frontmatter = false;
                    frontmatter_done = true;
                    continue;
                }
            }

            if in_frontmatter {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_string();
                    let value = value.trim().trim_matches('"').to_string();
                    frontmatter.insert(key, value);
                }
            } else if frontmatter_done && (!body.is_empty() || !line.trim().is_empty()) {
                body.push_str(line);
                body.push('\n');
            }
        }

        (frontmatter, body)
    }

    /// Format skill for Codex (single-line name/description)
    fn format_codex_skill(&self, content: &str) -> String {
        let (frontmatter, body) = self.parse_skill_md(content);

        let name = frontmatter.get("name").map(|s| s.trim()).unwrap_or("");
        let desc = frontmatter
            .get("description")
            .map(|s| s.lines().next().unwrap_or("").trim())
            .unwrap_or("");

        format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}",
            name, desc, body
        )
    }

    /// Format skill for Gemini (lowercase name, max 64 chars)
    fn format_gemini_skill(&self, content: &str) -> String {
        let (frontmatter, body) = self.parse_skill_md(content);

        let name = frontmatter
            .get("name")
            .map(|s| {
                let normalized = s.to_lowercase().replace(' ', "-");
                if normalized.len() > 64 {
                    normalized[..64].to_string()
                } else {
                    normalized
                }
            })
            .unwrap_or_default();

        let desc = frontmatter.get("description").cloned().unwrap_or_default();

        format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}",
            name, desc, body
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_dir_claude_skill() {
        let executor = ExtensionExecutor::new(CliType::Claude);
        let dir = executor.install_dir(ExtensionType::Skill);
        assert!(dir.to_string_lossy().contains(".claude/skills"));
    }

    #[test]
    fn test_install_dir_claude_plugin() {
        let executor = ExtensionExecutor::new(CliType::Claude);
        let dir = executor.install_dir(ExtensionType::Plugin);
        assert!(dir.to_string_lossy().contains(".claude/plugins"));
    }

    #[test]
    fn test_install_dir_codex_skill() {
        let executor = ExtensionExecutor::new(CliType::Codex);
        let dir = executor.install_dir(ExtensionType::Skill);
        assert!(dir.to_string_lossy().contains(".codex/skills"));
    }

    #[test]
    fn test_install_dir_gemini_extension() {
        let executor = ExtensionExecutor::new(CliType::Gemini);
        // Gemini uses extensions/ directory for everything
        let dir = executor.install_dir(ExtensionType::Skill);
        assert!(dir.to_string_lossy().contains(".gemini/extensions"));
        let dir = executor.install_dir(ExtensionType::Plugin);
        assert!(dir.to_string_lossy().contains(".gemini/extensions"));
    }

    #[test]
    fn test_parse_skill_md() {
        let executor = ExtensionExecutor::new(CliType::Claude);
        let content = r#"---
name: Test Skill
description: A test skill
custom_field: value
---

# Test Body

Some content here.
"#;
        let (frontmatter, body) = executor.parse_skill_md(content);
        assert_eq!(frontmatter.get("name"), Some(&"Test Skill".to_string()));
        assert_eq!(
            frontmatter.get("description"),
            Some(&"A test skill".to_string())
        );
        assert!(body.contains("Test Body"));
    }

    #[test]
    fn test_format_codex_skill() {
        let executor = ExtensionExecutor::new(CliType::Codex);
        let content = r#"---
name: Test Skill
description: Line one
  Line two
custom_field: value
---

# Body
"#;
        let result = executor.format_codex_skill(content);
        assert!(result.contains("name: Test Skill"));
        assert!(result.contains("description: Line one"));
        assert!(!result.contains("Line two"));
        assert!(!result.contains("custom_field"));
    }

    #[test]
    fn test_format_gemini_skill() {
        let executor = ExtensionExecutor::new(CliType::Gemini);
        let content = r#"---
name: Test Skill Name
description: A description
---

# Body
"#;
        let result = executor.format_gemini_skill(content);
        assert!(result.contains("name: test-skill-name"));
    }

    #[test]
    fn test_format_gemini_skill_long_name() {
        let executor = ExtensionExecutor::new(CliType::Gemini);
        let long_name = "a".repeat(100);
        let content = format!(
            r#"---
name: {}
description: A description
---

# Body
"#,
            long_name
        );
        let result = executor.format_gemini_skill(&content);
        // Name should be truncated to 64 chars
        let name_line = result.lines().find(|l| l.starts_with("name:")).unwrap();
        let name = name_line.strip_prefix("name: ").unwrap();
        assert_eq!(name.len(), 64);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_gemini_extension_structure() {
        // Create a mock skill content
        let skill_content = r#"---
name: Test Skill
description: A test skill for Gemini
---

# Test Skill

This is a test skill.
"#;

        let executor = ExtensionExecutor::new(CliType::Gemini);
        let (frontmatter, body) = executor.parse_skill_md(skill_content);

        // Verify parsing
        assert_eq!(frontmatter.get("name"), Some(&"Test Skill".to_string()));
        assert!(body.contains("This is a test skill"));

        // Verify TOML generation format
        let description = frontmatter.get("description").unwrap();
        let toml_content = format!(
            r#"description = "{}"
prompt = """
{}
"""
"#,
            description.lines().next().unwrap_or(description),
            body.trim()
        );
        assert!(toml_content.contains("description = \"A test skill for Gemini\""));
        assert!(toml_content.contains("prompt = \"\"\""));
    }
}
