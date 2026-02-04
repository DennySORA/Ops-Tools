use crate::i18n::{self, keys};

/// Extension type
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExtensionType {
    /// Skill: has SKILL.md, supported by all three CLIs
    Skill,
    /// Plugin: has .claude-plugin/, only supported by Claude
    Plugin,
}

impl ExtensionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ExtensionType::Skill => "Skill",
            ExtensionType::Plugin => "Plugin",
        }
    }
}

/// CLI type (reusing the concept from mcp_manager but specific to skill_installer)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CliType {
    Claude,
    Codex,
    Gemini,
}

impl CliType {
    pub fn display_name(&self) -> &'static str {
        match self {
            CliType::Claude => "Anthropic Claude",
            CliType::Codex => "OpenAI Codex",
            CliType::Gemini => "Google Gemini",
        }
    }

    pub fn config_dir_name(&self) -> &'static str {
        match self {
            CliType::Claude => ".claude",
            CliType::Codex => ".codex",
            CliType::Gemini => ".gemini",
        }
    }
}

/// Extension definition
#[derive(Clone)]
pub struct Extension {
    pub name: &'static str,
    pub display_name_key: &'static str,
    pub extension_type: ExtensionType,
    pub source_repo: &'static str,
    pub source_path: &'static str,
    pub cli_support: &'static [CliType],
    /// For plugins with extractable skills, this is the skill subpath (e.g., "skills/frontend-design")
    /// When installing for Codex/Gemini, this path will be extracted instead of the full plugin
    pub skill_subpath: Option<&'static str>,
    /// For plugins with commands but no skills, this is the command file to convert to skill
    /// (e.g., "commands/code-review.md")
    /// When installing for Codex/Gemini, this file will be converted to SKILL.md format
    pub command_file: Option<&'static str>,
    /// Whether this plugin uses hooks (Gemini supports hooks, Codex does not)
    /// When true for Gemini: installs full plugin with hooks
    /// When true for Codex: falls back to command_file conversion or not supported
    pub has_hooks: bool,
    /// Marketplace name for plugins that require full marketplace structure.
    /// When set, the installer will:
    /// 1. Git clone the full repo to ~/.claude/plugins/marketplaces/<marketplace_name>/
    /// 2. Create symlink in cache/<marketplace_name>/<plugin_name>/<version>/
    /// 3. Register in known_marketplaces.json and installed_plugins.json
    pub marketplace_name: Option<&'static str>,
    /// Plugin path within the marketplace repo (e.g., "plugin" for claude-mem)
    /// Only used when marketplace_name is set
    pub marketplace_plugin_path: Option<&'static str>,
    /// Plugin version for marketplace-based installations
    pub version: Option<&'static str>,
}

impl Extension {
    pub fn display_name(&self) -> &'static str {
        i18n::t(self.display_name_key)
    }
}

/// All available extensions
const EXTENSIONS: &[Extension] = &[
    // Plugins with hooks - Gemini supports hooks via migration, Codex does not
    // For Claude/Gemini: installs full plugin with hooks
    // For Codex: not supported (no hook system)
    Extension {
        name: "ralph-wiggum",
        display_name_key: keys::SKILL_RALPH_WIGGUM,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/ralph-wiggum",
        cli_support: &[CliType::Claude, CliType::Gemini],
        skill_subpath: None,
        command_file: None,
        has_hooks: true,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
    },
    Extension {
        name: "security-guidance",
        display_name_key: keys::SKILL_SECURITY_GUIDANCE,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/security-guidance",
        cli_support: &[CliType::Claude, CliType::Gemini],
        skill_subpath: None,
        command_file: None,
        has_hooks: true,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
    },
    // Plugins with commands that can be converted to skills
    // For Claude: installs as plugin
    // For Codex/Gemini: converts command file to SKILL.md
    Extension {
        name: "code-review",
        display_name_key: keys::SKILL_CODE_REVIEW,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/code-review",
        cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
        skill_subpath: None,
        command_file: Some("commands/code-review.md"),
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
    },
    Extension {
        name: "pr-review-toolkit",
        display_name_key: keys::SKILL_PR_REVIEW_TOOLKIT,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/pr-review-toolkit",
        cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
        skill_subpath: None,
        command_file: Some("commands/review-pr.md"),
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
    },
    Extension {
        name: "commit-commands",
        display_name_key: keys::SKILL_COMMIT_COMMANDS,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/commit-commands",
        cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
        skill_subpath: None,
        command_file: Some("commands/commit.md"),
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
    },
    // Plugins with extractable skills (all CLIs supported)
    // For Claude: installs as plugin
    // For Codex/Gemini: extracts skill subdirectory and converts SKILL.md format
    Extension {
        name: "frontend-design",
        display_name_key: keys::SKILL_FRONTEND_DESIGN,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/frontend-design",
        cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
        skill_subpath: Some("skills/frontend-design"),
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
    },
    Extension {
        name: "writing-rules",
        display_name_key: keys::SKILL_WRITING_RULES,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/hookify",
        cli_support: &[CliType::Claude, CliType::Codex, CliType::Gemini],
        skill_subpath: Some("skills/writing-rules"),
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
    },
    // Third-party plugins requiring full marketplace structure
    // These plugins have scripts that reference the marketplace root
    Extension {
        name: "claude-mem",
        display_name_key: keys::SKILL_CLAUDE_MEM,
        extension_type: ExtensionType::Plugin,
        source_repo: "thedotmack/claude-mem",
        source_path: "plugin", // Not used for marketplace installs
        cli_support: &[CliType::Claude, CliType::Gemini], // Full marketplace support
        skill_subpath: None,
        command_file: None,
        has_hooks: true,
        marketplace_name: Some("thedotmack"),
        marketplace_plugin_path: Some("plugin"),
        version: Some("9.0.12"),
    },
];

/// Get available extensions for a specific CLI
pub fn get_available_extensions(cli: CliType) -> Vec<Extension> {
    EXTENSIONS
        .iter()
        .filter(|ext| ext.cli_support.contains(&cli))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{self, Language};

    #[test]
    fn test_cli_type_display_name() {
        assert_eq!(CliType::Claude.display_name(), "Anthropic Claude");
        assert_eq!(CliType::Codex.display_name(), "OpenAI Codex");
        assert_eq!(CliType::Gemini.display_name(), "Google Gemini");
    }

    #[test]
    fn test_cli_type_config_dir() {
        assert_eq!(CliType::Claude.config_dir_name(), ".claude");
        assert_eq!(CliType::Codex.config_dir_name(), ".codex");
        assert_eq!(CliType::Gemini.config_dir_name(), ".gemini");
    }

    #[test]
    fn test_extension_type_display_name() {
        assert_eq!(ExtensionType::Skill.display_name(), "Skill");
        assert_eq!(ExtensionType::Plugin.display_name(), "Plugin");
    }

    #[test]
    fn test_get_available_extensions_claude() {
        let extensions = get_available_extensions(CliType::Claude);
        assert!(!extensions.is_empty());
        // Claude should have access to plugins
        assert!(extensions
            .iter()
            .any(|ext| ext.extension_type == ExtensionType::Plugin));
    }

    #[test]
    fn test_get_available_extensions_codex() {
        let extensions = get_available_extensions(CliType::Codex);
        // Codex can have plugins with skill_subpath or command_file (will be installed as skills)
        assert!(!extensions.is_empty());
        // All extensions for Codex should have either skill_subpath or command_file
        assert!(extensions
            .iter()
            .all(|ext| ext.skill_subpath.is_some() || ext.command_file.is_some()));
    }

    #[test]
    fn test_get_available_extensions_gemini() {
        let extensions = get_available_extensions(CliType::Gemini);
        // Gemini supports hooks, skills, and command conversion
        assert!(!extensions.is_empty());
        // All extensions for Gemini should have either:
        // - skill_subpath (extract skill)
        // - command_file (convert to skill)
        // - has_hooks (install as plugin with hooks)
        assert!(extensions
            .iter()
            .all(|ext| ext.skill_subpath.is_some() || ext.command_file.is_some() || ext.has_hooks));
    }

    #[test]
    fn test_extension_display_name() {
        let _guard = i18n::test_lock();
        let previous = i18n::current_language();
        i18n::set_language(Language::English);

        let extensions = get_available_extensions(CliType::Claude);
        let ext = extensions
            .iter()
            .find(|e| e.name == "ralph-wiggum")
            .expect("Missing ralph-wiggum extension");
        assert_eq!(ext.display_name(), i18n::t(keys::SKILL_RALPH_WIGGUM));

        i18n::set_language(previous);
    }
}
