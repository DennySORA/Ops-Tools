use super::tools::{CliType, Extension, ExtensionType};
use crate::core::{OperationError, Result};
use crate::i18n::keys;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
                            if marketplace.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                                if let Ok(plugins) = fs::read_dir(marketplace.path()) {
                                    for plugin in plugins.flatten() {
                                        if plugin.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                                            let plugin_name = plugin.file_name().to_string_lossy().to_string();
                                            // Check if any version directory exists with plugin.json
                                            if let Ok(versions) = fs::read_dir(plugin.path()) {
                                                for version in versions.flatten() {
                                                    let plugin_json = version.path().join(".claude-plugin/plugin.json");
                                                    if plugin_json.exists() {
                                                        installed.insert(plugin_name.clone(), ExtensionType::Plugin);
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

    /// Install an extension from GitHub
    pub fn install(&self, ext: &Extension) -> Result<()> {
        // Gemini has completely different extension format
        if self.cli == CliType::Gemini {
            return self.install_for_gemini(ext);
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
                message: crate::tr!(keys::SKILL_INSTALLER_DOWNLOAD_FAILED, error = "git clone failed"),
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
            fs::remove_file(&version_link).or_else(|_| fs::remove_dir_all(&version_link)).ok();
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

        // 3. Update known_marketplaces.json
        self.update_known_marketplaces(marketplace_name, ext.source_repo, &marketplace_dir)?;

        // 4. Update installed_plugins.json
        self.update_installed_plugins(ext.name, marketplace_name, &version_link, version)?;

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
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({
                "version": 2,
                "plugins": {}
            }))
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
        let marketplace_dir = home.join(".claude/plugins/marketplaces").join(marketplace_name);
        if marketplace_dir.exists() {
            fs::remove_dir_all(&marketplace_dir).map_err(|err| OperationError::Io {
                path: marketplace_dir.display().to_string(),
                source: err,
            })?;
        }

        // 4. Remove from known_marketplaces.json
        self.remove_from_known_marketplaces(marketplace_name)?;

        Ok(())
    }

    /// Remove plugin from installed_plugins.json
    fn remove_from_installed_plugins(&self, plugin_name: &str, marketplace_name: &str) -> Result<()> {
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
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({
                "version": 2,
                "plugins": {}
            }));

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
