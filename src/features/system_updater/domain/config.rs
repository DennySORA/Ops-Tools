use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct Config {
    pub report: ReportConfig,
    pub runtime: RuntimeConfig,
    pub preflight: PreflightConfig,
    pub apt: AptConfig,
    pub brew: BrewConfig,
    pub macos: MacosConfig,
    pub dgx: DgxConfig,
    pub docker: DockerConfig,
    pub tools: ToolsConfig,
    pub cleanup: CleanupConfig,
    pub notifications: NotificationConfig,
    pub scheduling: SchedulingConfig,
    pub profiles: BTreeMap<String, ProfileConfig>,
}

impl Config {
    pub fn apply_profile(&mut self, profile_name: &str) -> Result<(), String> {
        let profile = self
            .builtin_profiles()
            .remove(profile_name)
            .or_else(|| self.profiles.get(profile_name).cloned())
            .ok_or_else(|| format!("unknown profile: {profile_name}"))?;
        self.apply_profile_config(&profile);
        Ok(())
    }

    pub fn builtin_profiles(&self) -> BTreeMap<String, ProfileConfig> {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "full".to_string(),
            ProfileConfig {
                description: Some("Full maintenance with standard cleanup.".into()),
                ..ProfileConfig::default()
            },
        );
        profiles.insert(
            "safe".to_string(),
            ProfileConfig {
                description: Some(
                    "Conservative maintenance without auto reboot or remote nvm self-update."
                        .into(),
                ),
                auto_reboot: Some(false),
                cleanup_strategy: Some(CleanupStrategy::Conservative),
                disable_nvm_self_update: Some(true),
                disable_docker_prune: Some(true),
                ..ProfileConfig::default()
            },
        );
        profiles.insert(
            "aggressive".to_string(),
            ProfileConfig {
                description: Some("Full maintenance with aggressive cleanup.".into()),
                cleanup_strategy: Some(CleanupStrategy::Aggressive),
                ..ProfileConfig::default()
            },
        );
        profiles
    }

    fn apply_profile_config(&mut self, profile: &ProfileConfig) {
        if let Some(auto_reboot) = profile.auto_reboot {
            self.runtime.auto_reboot = auto_reboot;
        }
        if let Some(strategy) = profile.cleanup_strategy {
            self.cleanup.strategy = strategy;
        }
        if let Some(disable_nvm_self_update) = profile.disable_nvm_self_update {
            self.tools.nvm.self_update = !disable_nvm_self_update;
        }
        if let Some(disable_docker_prune) = profile.disable_docker_prune {
            self.docker.prune = !disable_docker_prune;
            self.cleanup.docker_system_prune = !disable_docker_prune;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ReportConfig {
    pub dir: PathBuf,
    pub diff_history_limit: usize,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            dir: state_dir().join("update").join("reports"),
            diff_history_limit: 20,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RuntimeConfig {
    pub auto_reboot: bool,
    pub lock_path: PathBuf,
    pub needrestart_reject: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            auto_reboot: true,
            lock_path: state_dir().join("update").join("update.lock"),
            needrestart_reject: vec![
                "nvidia-persistenced".into(),
                "nvidia-fabricmanager".into(),
                "dgx-dashboard".into(),
                "dgx-dashboard-admin".into(),
                "nv-docker".into(),
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PreflightConfig {
    pub check_network: bool,
    pub min_root_free_gb: u64,
    pub min_var_free_gb: u64,
}

impl Default for PreflightConfig {
    fn default() -> Self {
        Self {
            check_network: true,
            min_root_free_gb: 5,
            min_var_free_gb: 2,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AptConfig {
    pub maintenance_packages: Vec<String>,
    pub protected_keywords: Vec<String>,
    pub hold_packages: Vec<String>,
    pub deny_remove_packages: Vec<String>,
}

impl Default for AptConfig {
    fn default() -> Self {
        Self {
            maintenance_packages: vec!["fwupd".into(), "needrestart".into()],
            protected_keywords: vec!["nvidia".into(), "dgx".into(), "cuda".into(), "nccl".into()],
            hold_packages: Vec::new(),
            deny_remove_packages: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct BrewConfig {
    pub enabled: bool,
    pub cleanup: bool,
    pub autoremove: bool,
    pub greedy_casks: bool,
}

impl Default for BrewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cleanup: true,
            autoremove: true,
            greedy_casks: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct MacosConfig {
    pub software_update: bool,
    pub recommended_only: bool,
    pub allow_restart: bool,
}

impl Default for MacosConfig {
    fn default() -> Self {
        Self {
            software_update: true,
            recommended_only: true,
            allow_restart: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct DgxConfig {
    pub cuda_major: String,
    pub cuda_arch: String,
    pub kernel_meta: String,
    pub headers_meta: String,
    pub driver_pkg: String,
    pub modules_pkg: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DockerConfig {
    pub compose_projects: Vec<PathBuf>,
    pub prune: bool,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            compose_projects: Vec::new(),
            prune: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct ToolsConfig {
    pub nvm: NvmConfig,
    pub bun: ToggleConfig,
    pub deno: ToggleConfig,
    pub pipx: PackagePolicyConfig,
    pub conda: CondaConfig,
    pub node: NodeConfig,
    pub rust: RustConfig,
    pub uv: UvConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct NvmConfig {
    pub self_update: bool,
    pub installer_version: String,
    pub installer_sha256: Option<String>,
}

impl Default for NvmConfig {
    fn default() -> Self {
        Self {
            self_update: true,
            installer_version: "v0.40.4".into(),
            installer_sha256: Some(
                "4b7412c49960c7d31e8df72da90c1fb5b8cccb419ac99537b737028d497aba4f".into(),
            ),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ToggleConfig {
    pub enabled: bool,
}

impl ToggleConfig {
    pub fn enabled() -> Self {
        Self { enabled: true }
    }
}

impl Default for ToggleConfig {
    fn default() -> Self {
        Self::enabled()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PackagePolicyConfig {
    pub enabled: bool,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

impl PackagePolicyConfig {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            allow: Vec::new(),
            deny: Vec::new(),
        }
    }
}

impl Default for PackagePolicyConfig {
    fn default() -> Self {
        Self::enabled()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CondaConfig {
    pub enabled: bool,
    pub envs: Vec<String>,
}

impl Default for CondaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            envs: vec!["base".into()],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct NodeConfig {
    pub npm_allow: Vec<String>,
    pub npm_deny: Vec<String>,
    pub pnpm_allow: Vec<String>,
    pub pnpm_deny: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct RustConfig {
    pub cargo_allow: Vec<String>,
    pub cargo_deny: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct UvConfig {
    pub enabled: bool,
    pub tool_allow: Vec<String>,
    pub tool_deny: Vec<String>,
}

impl Default for UvConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tool_allow: Vec::new(),
            tool_deny: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CleanupStrategy {
    Conservative,
    Normal,
    Aggressive,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CleanupConfig {
    pub strategy: CleanupStrategy,
    pub flatpak_unused: bool,
    pub uv_cache: bool,
    pub pip_cache: bool,
    pub conda_cache: bool,
    pub bun_cache: bool,
    pub deno_cache: bool,
    pub journal_max_days: Option<u32>,
    pub journal_max_size_mb: Option<u64>,
    pub crash_reports: bool,
    pub docker_system_prune: bool,
    pub docker_builder_prune: bool,
    pub docker_volume_prune: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            strategy: CleanupStrategy::Normal,
            flatpak_unused: true,
            uv_cache: true,
            pip_cache: true,
            conda_cache: true,
            bun_cache: true,
            deno_cache: true,
            journal_max_days: Some(14),
            journal_max_size_mb: Some(512),
            crash_reports: true,
            docker_system_prune: true,
            docker_builder_prune: false,
            docker_volume_prune: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct NotificationConfig {
    pub on_success_command: Option<String>,
    pub on_warning_command: Option<String>,
    pub on_failure_command: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SchedulingConfig {
    pub on_calendar: String,
    pub persistent: bool,
    pub randomized_delay_minutes: u64,
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        Self {
            on_calendar: "daily".into(),
            persistent: true,
            randomized_delay_minutes: 30,
        }
    }
}

fn state_dir() -> PathBuf {
    dirs::state_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join(".local/state")))
        .unwrap_or_else(|| PathBuf::from(".local/state"))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct ProfileConfig {
    pub description: Option<String>,
    pub include_groups: Vec<String>,
    pub exclude_groups: Vec<String>,
    pub skip_steps: Vec<String>,
    pub auto_reboot: Option<bool>,
    pub cleanup_strategy: Option<CleanupStrategy>,
    pub disable_nvm_self_update: Option<bool>,
    pub disable_docker_prune: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::{CleanupStrategy, Config};

    #[test]
    fn applies_safe_profile_overrides() {
        let mut config = Config::default();
        config.apply_profile("safe").expect("safe profile");

        assert!(!config.runtime.auto_reboot);
        assert_eq!(config.cleanup.strategy, CleanupStrategy::Conservative);
        assert!(!config.tools.nvm.self_update);
        assert!(!config.docker.prune);
    }
}
