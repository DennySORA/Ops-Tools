use crate::i18n::{self, keys};
use std::path::{Path, PathBuf};

const TRIVY_INSTALL_CURL_SCRIPT: &str = r#"set -e; command -v curl >/dev/null 2>&1; mkdir -p "$HOME/.local/bin"; tmp="${TMPDIR:-/tmp}/ops-tools-trivy-install.$$"; curl -fsSL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh -o "$tmp"; sh "$tmp" -b "$HOME/.local/bin"; rm -f "$tmp""#;
const TRIVY_INSTALL_WGET_SCRIPT: &str = r#"set -e; command -v wget >/dev/null 2>&1; mkdir -p "$HOME/.local/bin"; tmp="${TMPDIR:-/tmp}/ops-tools-trivy-install.$$"; wget -qO "$tmp" https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh; sh "$tmp" -b "$HOME/.local/bin"; rm -f "$tmp""#;
const SEMGREP_PIPX_APT_SCRIPT: &str = r#"set -e; command -v apt-get >/dev/null 2>&1; if command -v sudo >/dev/null 2>&1; then sudo apt-get install -y pipx; else apt-get install -y pipx; fi; mkdir -p "$HOME/.local/bin"; pipx install semgrep"#;
const SEMGREP_VENV_SCRIPT: &str = r#"set -e; command -v python3 >/dev/null 2>&1; venv_dir="$HOME/.local/share/ops-tools/semgrep-venv"; python3 -m venv "$venv_dir"; "$venv_dir/bin/pip" install semgrep; mkdir -p "$HOME/.local/bin"; ln -sf "$venv_dir/bin/semgrep" "$HOME/.local/bin/semgrep""#;

#[derive(Clone, Copy, Debug)]
pub enum ScanTool {
    Gitleaks,
    Trufflehog,
    GitSecrets,
    Trivy,
    Semgrep,
}

pub struct ScanCommand {
    pub label: String,
    pub args: Vec<String>,
    pub workdir: Option<PathBuf>,
}

pub struct InstallStrategy {
    pub label: &'static str,
    pub program: &'static str,
    pub args: Vec<String>,
    pub use_sudo: bool,
}

impl InstallStrategy {
    fn new(label: &'static str, program: &'static str, args: &[&str], use_sudo: bool) -> Self {
        Self {
            label,
            program,
            args: args.iter().map(|item| item.to_string()).collect(),
            use_sudo,
        }
    }
}

pub fn all_tools() -> Vec<ScanTool> {
    vec![
        ScanTool::Gitleaks,
        ScanTool::Trufflehog,
        ScanTool::GitSecrets,
        ScanTool::Trivy,
        ScanTool::Semgrep,
    ]
}

impl ScanTool {
    pub fn display_name(&self) -> &'static str {
        match self {
            ScanTool::Gitleaks => "Gitleaks",
            ScanTool::Trufflehog => "TruffleHog",
            ScanTool::GitSecrets => "Git-Secrets",
            ScanTool::Trivy => "Trivy",
            ScanTool::Semgrep => "Semgrep",
        }
    }

    pub fn binary_name(&self) -> &'static str {
        match self {
            ScanTool::Gitleaks => "gitleaks",
            ScanTool::Trufflehog => "trufflehog",
            ScanTool::GitSecrets => "git-secrets",
            ScanTool::Trivy => "trivy",
            ScanTool::Semgrep => "semgrep",
        }
    }

    pub fn scan_commands(&self, repo_root: &Path, worktree_root: &Path) -> Vec<ScanCommand> {
        let repo_path = repo_root
            .canonicalize()
            .unwrap_or_else(|_| repo_root.to_path_buf());
        let worktree_path = worktree_root
            .canonicalize()
            .unwrap_or_else(|_| worktree_root.to_path_buf());
        let repo_str = repo_path.display().to_string();
        let worktree_str = worktree_path.display().to_string();
        let file_url = format!("file://{}", repo_str);
        let tool_name = self.display_name();
        let history_scope = i18n::t(keys::SECURITY_SCANNER_SCOPE_GIT_HISTORY);
        let worktree_scope = i18n::t(keys::SECURITY_SCANNER_SCOPE_WORKTREE);
        let label_for = |scope: &str| -> String {
            crate::tr!(
                keys::SECURITY_SCANNER_COMMAND_LABEL,
                tool = tool_name,
                scope = scope
            )
        };

        match self {
            ScanTool::Gitleaks => vec![
                ScanCommand {
                    label: label_for(history_scope),
                    args: vec![
                        "detect".to_string(),
                        "--source".to_string(),
                        repo_str.clone(),
                        "--no-banner".to_string(),
                        "--redact".to_string(),
                        "--exit-code".to_string(),
                        "1".to_string(),
                    ],
                    workdir: Some(repo_path.clone()),
                },
                ScanCommand {
                    label: label_for(worktree_scope),
                    args: vec![
                        "detect".to_string(),
                        "--source".to_string(),
                        worktree_str.clone(),
                        "--no-git".to_string(),
                        "--no-banner".to_string(),
                        "--redact".to_string(),
                        "--exit-code".to_string(),
                        "1".to_string(),
                    ],
                    workdir: Some(worktree_path.clone()),
                },
            ],
            ScanTool::Trufflehog => vec![
                ScanCommand {
                    label: label_for(history_scope),
                    args: vec![
                        "git".to_string(),
                        file_url,
                        "--fail".to_string(),
                        "--json".to_string(),
                    ],
                    workdir: Some(repo_path.clone()),
                },
                ScanCommand {
                    label: label_for(worktree_scope),
                    args: vec![
                        "filesystem".to_string(),
                        worktree_str.clone(),
                        "--fail".to_string(),
                        "--json".to_string(),
                    ],
                    workdir: Some(worktree_path.clone()),
                },
            ],
            ScanTool::GitSecrets => vec![
                ScanCommand {
                    label: label_for(worktree_scope),
                    args: vec!["--scan".to_string(), "-r".to_string()],
                    workdir: Some(worktree_path.clone()),
                },
                ScanCommand {
                    label: label_for(history_scope),
                    args: vec!["--scan-history".to_string()],
                    workdir: Some(repo_path),
                },
            ],
            ScanTool::Trivy => vec![ScanCommand {
                label: label_for("SCA & Misconfig"),
                args: vec![
                    "fs".to_string(),
                    worktree_str.clone(),
                    "--scanners".to_string(),
                    "vuln,config".to_string(),
                    "--exit-code".to_string(),
                    "1".to_string(),
                    "--no-progress".to_string(),
                ],
                workdir: Some(worktree_path.clone()),
            }],
            ScanTool::Semgrep => vec![ScanCommand {
                label: label_for("SAST"),
                args: vec![
                    "scan".to_string(),
                    "--config=auto".to_string(),
                    "--error".to_string(),
                    "--quiet".to_string(),
                    worktree_str.clone(),
                ],
                workdir: Some(worktree_path.clone()),
            }],
        }
    }

    pub fn install_strategies(&self) -> Vec<InstallStrategy> {
        match self {
            ScanTool::Gitleaks => vec![
                InstallStrategy::new("brew", "brew", &["install", "gitleaks"], false),
                InstallStrategy::new("apt-get", "apt-get", &["install", "-y", "gitleaks"], true),
                InstallStrategy::new("dnf", "dnf", &["install", "-y", "gitleaks"], true),
                InstallStrategy::new("pacman", "pacman", &["-S", "--noconfirm", "gitleaks"], true),
                InstallStrategy::new(
                    "go install",
                    "go",
                    &["install", "github.com/gitleaks/gitleaks/v8@latest"],
                    false,
                ),
            ],
            ScanTool::Trufflehog => vec![
                InstallStrategy::new("brew", "brew", &["install", "trufflehog"], false),
                InstallStrategy::new("apt-get", "apt-get", &["install", "-y", "trufflehog"], true),
                InstallStrategy::new("dnf", "dnf", &["install", "-y", "trufflehog"], true),
                InstallStrategy::new(
                    "pacman",
                    "pacman",
                    &["-S", "--noconfirm", "trufflehog"],
                    true,
                ),
                InstallStrategy::new(
                    "go install",
                    "go",
                    &["install", "github.com/trufflesecurity/trufflehog@latest"],
                    false,
                ),
            ],
            ScanTool::GitSecrets => vec![
                InstallStrategy::new("brew", "brew", &["install", "git-secrets"], false),
                InstallStrategy::new(
                    "apt-get",
                    "apt-get",
                    &["install", "-y", "git-secrets"],
                    true,
                ),
                InstallStrategy::new("dnf", "dnf", &["install", "-y", "git-secrets"], true),
                InstallStrategy::new(
                    "pacman",
                    "pacman",
                    &["-S", "--noconfirm", "git-secrets"],
                    true,
                ),
            ],
            ScanTool::Trivy => vec![
                InstallStrategy::new("brew", "brew", &["install", "trivy"], false),
                InstallStrategy::new(
                    "install.sh (curl)",
                    "sh",
                    &["-c", TRIVY_INSTALL_CURL_SCRIPT],
                    false,
                ),
                InstallStrategy::new(
                    "install.sh (wget)",
                    "sh",
                    &["-c", TRIVY_INSTALL_WGET_SCRIPT],
                    false,
                ),
                InstallStrategy::new("apt-get", "apt-get", &["install", "-y", "trivy"], true),
                InstallStrategy::new("dnf", "dnf", &["install", "-y", "trivy"], true),
                InstallStrategy::new("pacman", "pacman", &["-S", "--noconfirm", "trivy"], true),
                InstallStrategy::new(
                    "go install",
                    "go",
                    &["install", "github.com/aquasecurity/trivy/cmd/trivy@latest"],
                    false,
                ),
            ],
            ScanTool::Semgrep => vec![
                InstallStrategy::new("brew", "brew", &["install", "semgrep"], false),
                InstallStrategy::new("pipx", "pipx", &["install", "semgrep"], false),
                InstallStrategy::new(
                    "apt-get pipx",
                    "sh",
                    &["-c", SEMGREP_PIPX_APT_SCRIPT],
                    false,
                ),
                InstallStrategy::new(
                    "python venv",
                    "sh",
                    &["-c", SEMGREP_VENV_SCRIPT],
                    false,
                ),
                InstallStrategy::new("pip", "pip", &["install", "semgrep"], false),
                InstallStrategy::new("pip3", "pip3", &["install", "semgrep"], false),
            ],
        }
    }
}
