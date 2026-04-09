use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum StepStatus {
    #[serde(alias = "Ok")]
    Ok,
    #[serde(alias = "Warning")]
    Warning,
    #[serde(alias = "Failed")]
    Failed,
    #[serde(alias = "Skipped")]
    Skipped,
    #[serde(alias = "DryRun")]
    DryRun,
    #[serde(alias = "Partial")]
    Partial,
    #[serde(alias = "Blocked")]
    Blocked,
}

impl StepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::DryRun => "dry-run",
            Self::Partial => "partial",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatus {
    #[serde(alias = "Ok")]
    Ok,
    #[serde(alias = "Warning")]
    Warning,
    #[serde(alias = "Failed")]
    Failed,
    #[serde(alias = "Partial")]
    Partial,
    #[serde(alias = "ScanOnly")]
    ScanOnly,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Failed => "failed",
            Self::Partial => "partial",
            Self::ScanOnly => "scan-only",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandMode {
    #[serde(alias = "Stream")]
    Stream,
    #[serde(alias = "Capture")]
    Capture,
}

impl CommandMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Capture => "capture",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandStatus {
    #[serde(alias = "Ok")]
    Ok,
    #[serde(alias = "Failed")]
    Failed,
    #[serde(alias = "DryRun")]
    DryRun,
    #[serde(alias = "SpawnError")]
    SpawnError,
    #[serde(alias = "TimedOut")]
    TimedOut,
}

impl CommandStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::DryRun => "dry-run",
            Self::SpawnError => "spawn-error",
            Self::TimedOut => "timed-out",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StepGroup {
    Backup,
    Apt,
    Dgx,
    Services,
    Tooling,
    Cleanup,
    Verify,
    Reboot,
}

impl StepGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backup => "backup",
            Self::Apt => "apt",
            Self::Dgx => "dgx",
            Self::Services => "services",
            Self::Tooling => "tooling",
            Self::Cleanup => "cleanup",
            Self::Verify => "verify",
            Self::Reboot => "reboot",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepOutcome {
    pub status: StepStatus,
    pub detail: Option<String>,
}

impl StepOutcome {
    pub fn new(status: StepStatus, detail: impl Into<Option<String>>) -> Self {
        Self {
            status,
            detail: detail.into(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StepStatus::Ok, None)
    }

    pub fn skipped(detail: impl Into<String>) -> Self {
        Self::new(StepStatus::Skipped, Some(detail.into()))
    }

    pub fn dry_run(detail: impl Into<String>) -> Self {
        Self::new(StepStatus::DryRun, Some(detail.into()))
    }

    pub fn warning(detail: impl Into<String>) -> Self {
        Self::new(StepStatus::Warning, Some(detail.into()))
    }

    pub fn partial(detail: impl Into<String>) -> Self {
        Self::new(StepStatus::Partial, Some(detail.into()))
    }

    pub fn blocked(detail: impl Into<String>) -> Self {
        Self::new(StepStatus::Blocked, Some(detail.into()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StepEvent {
    pub step_id: String,
    pub name: String,
    pub group: String,
    pub status: StepStatus,
    pub index: usize,
    pub total: usize,
    pub duration_ms: u128,
    pub detail: Option<String>,
}

impl Default for StepEvent {
    fn default() -> Self {
        Self {
            step_id: String::new(),
            name: String::new(),
            group: String::new(),
            status: StepStatus::Ok,
            index: 0,
            total: 0,
            duration_ms: 0,
            detail: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CommandEvent {
    pub command_id: Option<u64>,
    pub step_id: Option<String>,
    pub command: String,
    pub mode: CommandMode,
    pub status: CommandStatus,
    pub exit_code: Option<i32>,
    pub detail: Option<String>,
    pub cwd: Option<PathBuf>,
    pub timeout_ms: Option<u64>,
    pub retry_attempt: u8,
    pub sudo: bool,
}

impl Default for CommandEvent {
    fn default() -> Self {
        Self {
            command_id: None,
            step_id: None,
            command: String::new(),
            mode: CommandMode::Stream,
            status: CommandStatus::Ok,
            exit_code: None,
            detail: None,
            cwd: None,
            timeout_ms: None,
            retry_attempt: 0,
            sudo: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteEvent {
    pub timestamp_ms: u128,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct RunContext {
    pub subcommand: String,
    pub profile: Option<String>,
    pub config_path: Option<PathBuf>,
    pub lock_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RunRecord {
    pub run_id: String,
    pub started_at_ms: u128,
    pub finished_at_ms: Option<u128>,
    pub overall_status: Option<String>,
    pub hostname: String,
    pub user: String,
    pub cwd: PathBuf,
    pub dry_run: bool,
    pub scan_only: bool,
    pub context: RunContext,
    pub report_path: PathBuf,
    pub log_path: PathBuf,
    pub artifact_dir: PathBuf,
    pub steps: Vec<StepEvent>,
    pub commands: Vec<CommandEvent>,
    pub notes: Vec<NoteEvent>,
}

impl Default for RunRecord {
    fn default() -> Self {
        Self {
            run_id: String::new(),
            started_at_ms: 0,
            finished_at_ms: None,
            overall_status: None,
            hostname: String::new(),
            user: String::new(),
            cwd: PathBuf::new(),
            dry_run: false,
            scan_only: false,
            context: RunContext::default(),
            report_path: PathBuf::new(),
            log_path: PathBuf::new(),
            artifact_dir: PathBuf::new(),
            steps: Vec::new(),
            commands: Vec::new(),
            notes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportArtifacts {
    pub run_id: String,
    pub report_path: PathBuf,
    pub log_path: PathBuf,
    pub artifact_dir: PathBuf,
}
