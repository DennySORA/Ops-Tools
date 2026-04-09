use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::InfrastructureError;
use crate::features::system_updater::domain::report::{
    CommandEvent, ReportArtifacts, RunStatus, StepEvent,
};
use std::path::{Path, PathBuf};

pub trait CommandObserver {
    fn record_command(&self, event: CommandEvent) -> Result<(), InfrastructureError>;
}

pub trait RunReporter {
    fn run_id(&self) -> &str;
    fn note(&self, message: &str) -> Result<(), InfrastructureError>;
    fn activate_step(&self, step_id: Option<&str>) -> Result<(), InfrastructureError>;
    fn record_step(&self, event: StepEvent) -> Result<(), InfrastructureError>;
    fn artifact_dir(&self) -> PathBuf;
    fn finalize(&self, status: RunStatus) -> Result<ReportArtifacts, InfrastructureError>;
}

pub trait CommandExecutor {
    fn is_dry_run(&self) -> bool;
    fn run(&self, command: &CommandSpec) -> Result<(), InfrastructureError>;
    fn capture(&self, command: &CommandSpec) -> Result<String, InfrastructureError>;
}

pub trait EnvironmentReader {
    fn var(&self, key: &str) -> Option<String>;
    fn current_dir(&self) -> Result<PathBuf, InfrastructureError>;
}

pub trait ToolProbe {
    fn command_path(&self, name: &str) -> Option<PathBuf>;
    fn is_writable(&self, path: &Path) -> bool;
}

pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> Result<String, InfrastructureError>;
    fn write_string(&self, path: &Path, contents: &str) -> Result<(), InfrastructureError>;
    fn copy_file(&self, from: &Path, to: &Path) -> Result<(), InfrastructureError>;
    fn rename(&self, from: &Path, to: &Path) -> Result<(), InfrastructureError>;
    fn create_dir_all(&self, path: &Path) -> Result<(), InfrastructureError>;
}

pub trait SystemProbe {
    fn hostname(&self) -> Result<String, InfrastructureError>;
    fn free_space_gib(&self, path: &Path) -> Result<u64, InfrastructureError>;
    fn dns_resolves(&self, host: &str) -> Result<bool, InfrastructureError>;
}

pub trait HostServices: EnvironmentReader + ToolProbe + FileSystem + SystemProbe {}

impl<T> HostServices for T where T: EnvironmentReader + ToolProbe + FileSystem + SystemProbe {}
