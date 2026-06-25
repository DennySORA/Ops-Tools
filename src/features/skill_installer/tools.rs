use crate::i18n::{self, keys};

/// Extension type
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExtensionType {
    /// Skill: has SKILL.md, supported by Claude and Codex
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
}

impl CliType {
    pub fn display_name(&self) -> &'static str {
        match self {
            CliType::Claude => "Anthropic Claude",
            CliType::Codex => "OpenAI Codex",
        }
    }

    pub fn config_dir_name(&self) -> &'static str {
        match self {
            CliType::Claude => ".claude",
            CliType::Codex => ".codex",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InstallScope {
    Local,
    Global,
}

#[derive(Clone, Copy)]
pub struct SkillsCliSpec {
    pub source: &'static str,
    pub skill: Option<&'static str>,
    pub path: Option<&'static str>,
    pub installed_name: &'static str,
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
    /// When installing for Codex, this path will be extracted instead of the full plugin
    pub skill_subpath: Option<&'static str>,
    /// For plugins with commands but no skills, this is the command file to convert to skill
    /// (e.g., "commands/code-review.md")
    /// When installing for Codex, this file will be converted to SKILL.md format
    pub command_file: Option<&'static str>,
    /// Whether this plugin uses hooks
    /// When true for Claude: installs full plugin with hooks
    /// When true: installs full plugin with hooks
    /// When true for Codex: converts hooks to hooks.json format (experimental, Bash-only)
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
    /// Install this entry through `npx skills add` instead of built-in GitHub extraction.
    pub skills_cli: Option<SkillsCliSpec>,
}

impl Extension {
    pub fn display_name(&self) -> &'static str {
        i18n::t(self.display_name_key)
    }

    pub fn installed_name(&self) -> &'static str {
        if let Some(spec) = self.skills_cli {
            return spec.installed_name;
        }

        if self.extension_type == ExtensionType::Skill && !self.source_path.is_empty() {
            return self.source_path.split('/').next_back().unwrap_or(self.name);
        }

        self.name
    }

    pub fn supports_scope(&self, cli: CliType, scope: InstallScope) -> bool {
        if scope == InstallScope::Global || self.skills_cli.is_some() {
            return true;
        }

        self.extension_type == ExtensionType::Skill
            || (cli == CliType::Codex
                && (self.skill_subpath.is_some() || self.command_file.is_some()))
    }
}

/// All available extensions
const EXTENSIONS: &[Extension] = &[
    // Plugins with extractable skills (all CLIs supported)
    // For Claude: installs as plugin
    // For Codex: extracts skill subdirectory and converts SKILL.md format
    Extension {
        name: "frontend-design",
        display_name_key: keys::SKILL_FRONTEND_DESIGN,
        extension_type: ExtensionType::Plugin,
        source_repo: "anthropics/claude-code",
        source_path: "plugins/frontend-design",
        cli_support: &[CliType::Claude, CliType::Codex],
        skill_subpath: Some("skills/frontend-design"),
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: None,
    },
    // Third-party plugins requiring full marketplace structure
    // These plugins have scripts that reference the marketplace root
    Extension {
        name: "claude-mem",
        display_name_key: keys::SKILL_CLAUDE_MEM,
        extension_type: ExtensionType::Plugin,
        source_repo: "thedotmack/claude-mem",
        source_path: "plugin",           // Not used for marketplace installs
        cli_support: &[CliType::Claude], // Full marketplace support
        skill_subpath: None,
        command_file: None,
        has_hooks: true,
        marketplace_name: Some("thedotmack"),
        marketplace_plugin_path: Some("plugin"),
        version: Some("10.1.0"),
        skills_cli: None,
    },
    Extension {
        name: "skills-frontend-ui-engineering",
        display_name_key: keys::SKILL_FRONTEND_UI_ENGINEERING,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "addyosmani/agent-skills",
            skill: Some("frontend-ui-engineering"),
            path: None,
            installed_name: "frontend-ui-engineering",
        }),
    },
    Extension {
        name: "skills-antfu-nuxt",
        display_name_key: keys::SKILL_ANTFU_NUXT,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "antfu/skills",
            skill: Some("nuxt"),
            path: None,
            installed_name: "nuxt",
        }),
    },
    Extension {
        name: "skills-nuxt-ui",
        display_name_key: keys::SKILL_NUXT_UI,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "nuxt/ui",
            skill: Some("nuxt-ui"),
            path: None,
            installed_name: "nuxt-ui",
        }),
    },
    Extension {
        name: "skills-onmax-nuxt",
        display_name_key: keys::SKILL_ONMAX_NUXT,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "onmax/nuxt-skills",
            skill: Some("nuxt"),
            path: None,
            installed_name: "nuxt",
        }),
    },
    Extension {
        name: "skills-nextlevel-ui-ux-pro-max",
        display_name_key: keys::SKILL_NEXTLEVEL_UI_UX_PRO_MAX,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "nextlevelbuilder/ui-ux-pro-max-skill",
            skill: None,
            path: None,
            installed_name: "ui-ux-pro-max",
        }),
    },
    Extension {
        name: "skills-frontend-design-system",
        display_name_key: keys::SKILL_FRONTEND_DESIGN_SYSTEM,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "git@github.com:supercent-io/skills-template.git",
            skill: Some("frontend-design-system"),
            path: None,
            installed_name: "frontend-design-system",
        }),
    },
    Extension {
        name: "skills-web-design-reviewer",
        display_name_key: keys::SKILL_WEB_DESIGN_REVIEWER,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "github/awesome-copilot",
            skill: Some("web-design-reviewer"),
            path: None,
            installed_name: "web-design-reviewer",
        }),
    },
    Extension {
        name: "skills-kimny-ui-ux-pro-max",
        display_name_key: keys::SKILL_KIMNY_UI_UX_PRO_MAX,
        extension_type: ExtensionType::Skill,
        source_repo: "git@github.com:kimny1143/claude-code-template.git",
        source_path: ".claude/skills/ui-ux-pro-max",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: None,
    },
    Extension {
        name: "skills-impeccable-frontend-design",
        display_name_key: keys::SKILL_IMPECCABLE_FRONTEND_DESIGN,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "git@github.com:pbakaus/impeccable.git",
            skill: Some("impeccable"),
            path: None,
            installed_name: "impeccable",
        }),
    },
    Extension {
        name: "skills-threejs-animation",
        display_name_key: keys::SKILL_THREEJS_ANIMATION,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "cloudai-x/threejs-skills",
            skill: Some("threejs-animation"),
            path: None,
            installed_name: "threejs-animation",
        }),
    },
    Extension {
        name: "skills-ui-animation",
        display_name_key: keys::SKILL_UI_ANIMATION,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "mblode/agent-skills",
            skill: Some("ui-animation"),
            path: None,
            installed_name: "ui-animation",
        }),
    },
    Extension {
        name: "skills-framer-motion-animator",
        display_name_key: keys::SKILL_FRAMER_MOTION_ANIMATOR,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "patricio0312rev/skills",
            skill: Some("framer-motion-animator"),
            path: None,
            installed_name: "framer-motion-animator",
        }),
    },
    Extension {
        name: "skills-code-review-expert",
        display_name_key: keys::SKILL_CODE_REVIEW_EXPERT,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "sanyuan0704/sanyuan-skills",
            skill: None,
            path: Some("skills/code-review-expert"),
            installed_name: "code-review-expert",
        }),
    },
    Extension {
        name: "skills-playwright-generate-test",
        display_name_key: keys::SKILL_PLAYWRIGHT_GENERATE_TEST,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "github/awesome-copilot",
            skill: Some("playwright-generate-test"),
            path: None,
            installed_name: "playwright-generate-test",
        }),
    },
    Extension {
        name: "skills-playwright-explore-website",
        display_name_key: keys::SKILL_PLAYWRIGHT_EXPLORE_WEBSITE,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "github/awesome-copilot",
            skill: Some("playwright-explore-website"),
            path: None,
            installed_name: "playwright-explore-website",
        }),
    },
    Extension {
        name: "skills-typescript-clean-code",
        display_name_key: keys::SKILL_TYPESCRIPT_CLEAN_CODE,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "bmad-labs/skills",
            skill: Some("typescript-clean-code"),
            path: None,
            installed_name: "typescript-clean-code",
        }),
    },
    Extension {
        name: "skills-typescript-unit-testing",
        display_name_key: keys::SKILL_TYPESCRIPT_UNIT_TESTING,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "bmad-labs/skills",
            skill: Some("typescript-unit-testing"),
            path: None,
            installed_name: "typescript-unit-testing",
        }),
    },
    Extension {
        name: "skills-mastering-typescript",
        display_name_key: keys::SKILL_MASTERING_TYPESCRIPT,
        extension_type: ExtensionType::Skill,
        source_repo: "",
        source_path: "",
        cli_support: &[CliType::Codex],
        skill_subpath: None,
        command_file: None,
        has_hooks: false,
        marketplace_name: None,
        marketplace_plugin_path: None,
        version: None,
        skills_cli: Some(SkillsCliSpec {
            source: "SpillwaveSolutions/mastering-typescript-skill",
            skill: Some("mastering-typescript"),
            path: None,
            installed_name: "mastering-typescript",
        }),
    },
];

/// Get available extensions for a specific CLI
pub fn get_available_extensions(cli: CliType, scope: InstallScope) -> Vec<Extension> {
    EXTENSIONS
        .iter()
        .filter(|ext| ext.cli_support.contains(&cli) && ext.supports_scope(cli, scope))
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
    }

    #[test]
    fn test_cli_type_config_dir() {
        assert_eq!(CliType::Claude.config_dir_name(), ".claude");
        assert_eq!(CliType::Codex.config_dir_name(), ".codex");
    }

    #[test]
    fn test_extension_type_display_name() {
        assert_eq!(ExtensionType::Skill.display_name(), "Skill");
        assert_eq!(ExtensionType::Plugin.display_name(), "Plugin");
    }

    #[test]
    fn test_get_available_extensions_claude() {
        let extensions = get_available_extensions(CliType::Claude, InstallScope::Global);
        assert!(!extensions.is_empty());
        // Claude should have access to plugins
        assert!(
            extensions
                .iter()
                .any(|ext| ext.extension_type == ExtensionType::Plugin)
        );
    }

    #[test]
    fn test_get_available_extensions_codex() {
        let extensions = get_available_extensions(CliType::Codex, InstallScope::Global);
        assert!(!extensions.is_empty());
        // Codex extensions must be installable as skills, converted plugins, hook plugins, or Skills CLI entries.
        assert!(extensions.iter().all(|ext| ext.extension_type == ExtensionType::Skill
            || ext.skill_subpath.is_some()
            || ext.command_file.is_some()
            || ext.has_hooks
            || ext.skills_cli.is_some()));
    }

    #[test]
    fn test_get_available_extensions_codex_local_includes_project_skills() {
        let extensions = get_available_extensions(CliType::Codex, InstallScope::Local);
        assert!(!extensions.is_empty());
        assert!(extensions.iter().all(|ext| ext.skills_cli.is_some()
            || ext.extension_type == ExtensionType::Skill
            || ext.skill_subpath.is_some()
            || ext.command_file.is_some()));
        assert!(
            extensions
                .iter()
                .any(|ext| ext.name == "skills-frontend-ui-engineering")
        );
        assert!(extensions.iter().any(|ext| {
            ext.name == "skills-frontend-design-system"
                && ext
                    .skills_cli
                    .map(|spec| spec.source.starts_with("git@github.com:"))
                    .unwrap_or(false)
        }));
        assert!(
            extensions
                .iter()
                .any(|ext| ext.name == "skills-kimny-ui-ux-pro-max")
        );
    }

    #[test]
    fn test_removed_builtin_extensions_are_not_available() {
        for cli in &[CliType::Claude, CliType::Codex] {
            let extensions = get_available_extensions(*cli, InstallScope::Global);
            assert!(!extensions.iter().any(|ext| ext.name == "ralph-wiggum"));
            assert!(!extensions.iter().any(|ext| ext.name == "loop-runner"));
        }
    }

    #[test]
    fn test_extension_display_name() {
        let _guard = i18n::test_lock();
        let previous = i18n::current_language();
        i18n::set_language(Language::English);

        let extensions = get_available_extensions(CliType::Claude, InstallScope::Global);
        let ext = extensions
            .iter()
            .find(|e| e.name == "frontend-design")
            .expect("Missing frontend-design extension");
        assert_eq!(ext.display_name(), i18n::t(keys::SKILL_FRONTEND_DESIGN));

        i18n::set_language(previous);
    }

    #[test]
    fn test_direct_skill_uses_source_path_installed_name() {
        let extensions = get_available_extensions(CliType::Codex, InstallScope::Global);
        let ext = extensions
            .iter()
            .find(|e| e.name == "skills-kimny-ui-ux-pro-max")
            .expect("Missing kimny UI UX Pro Max extension");
        assert_eq!(ext.installed_name(), "ui-ux-pro-max");
    }
}
