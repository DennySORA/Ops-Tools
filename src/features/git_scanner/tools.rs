use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub enum ScanTool {
    Gitleaks,
    Trufflehog,
    GitSecrets,
}

pub struct ScanCommand {
    pub label: &'static str,
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
    ]
}

impl ScanTool {
    pub fn display_name(&self) -> &'static str {
        match self {
            ScanTool::Gitleaks => "Gitleaks",
            ScanTool::Trufflehog => "TruffleHog",
            ScanTool::GitSecrets => "Git-Secrets",
        }
    }

    pub fn binary_name(&self) -> &'static str {
        match self {
            ScanTool::Gitleaks => "gitleaks",
            ScanTool::Trufflehog => "trufflehog",
            ScanTool::GitSecrets => "git-secrets",
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

        match self {
            ScanTool::Gitleaks => vec![
                ScanCommand {
                    label: "Gitleaks (Git 歷史)",
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
                    label: "Gitleaks (工作樹)",
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
                    label: "TruffleHog (Git 歷史)",
                    args: vec![
                        "git".to_string(),
                        file_url,
                        "--fail".to_string(),
                        "--json".to_string(),
                    ],
                    workdir: Some(repo_path.clone()),
                },
                ScanCommand {
                    label: "TruffleHog (工作樹)",
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
                    label: "Git-Secrets (工作樹)",
                    args: vec!["--scan".to_string(), "-r".to_string()],
                    workdir: Some(worktree_path.clone()),
                },
                ScanCommand {
                    label: "Git-Secrets (Git 歷史)",
                    args: vec!["--scan-history".to_string()],
                    workdir: Some(repo_path),
                },
            ],
        }
    }

    pub fn install_strategies(&self) -> Vec<InstallStrategy> {
        match self {
            ScanTool::Gitleaks => vec![
                InstallStrategy::new("brew", "brew", &["install", "gitleaks"], false),
                InstallStrategy::new("apt-get", "apt-get", &["install", "-y", "gitleaks"], true),
                InstallStrategy::new("dnf", "dnf", &["install", "-y", "gitleaks"], true),
                InstallStrategy::new(
                    "pacman",
                    "pacman",
                    &["-S", "--noconfirm", "gitleaks"],
                    true,
                ),
                InstallStrategy::new(
                    "go install",
                    "go",
                    &["install", "github.com/gitleaks/gitleaks/v8@latest"],
                    false,
                ),
            ],
            ScanTool::Trufflehog => vec![
                InstallStrategy::new("brew", "brew", &["install", "trufflehog"], false),
                InstallStrategy::new(
                    "apt-get",
                    "apt-get",
                    &["install", "-y", "trufflehog"],
                    true,
                ),
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
        }
    }
}
