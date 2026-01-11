use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use std::path::Path;
use std::process::Command;

use super::installer::resolve_tool_path;
use super::tools::{ScanCommand, ScanTool};

pub enum ScanStatus {
    Clean,
    Findings,
    Error,
}

pub struct ScanOutcome {
    pub label: String,
    pub status: ScanStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_scans(
    tool: ScanTool,
    repo_root: &Path,
    worktree_root: &Path,
) -> Result<Vec<ScanOutcome>> {
    let Some(tool_path) = resolve_tool_path(tool) else {
        return Err(OperationError::Command {
            command: tool.binary_name().to_string(),
            message: i18n::t(keys::ERROR_COMMAND_NOT_FOUND).to_string(),
        });
    };

    let steps = tool.scan_commands(repo_root, worktree_root);
    let mut outcomes = Vec::with_capacity(steps.len());

    for step in steps {
        outcomes.push(run_step(&tool_path, &step)?);
    }

    Ok(outcomes)
}

fn run_step(tool_path: &Path, step: &ScanCommand) -> Result<ScanOutcome> {
    let mut command = Command::new(tool_path);
    command.args(&step.args);
    if let Some(dir) = &step.workdir {
        command.current_dir(dir);
    }

    let output = command.output().map_err(|err| OperationError::Command {
        command: tool_path.display().to_string(),
        message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = err),
    })?;

    let exit_code = output.status.code();
    let status = if output.status.success() {
        ScanStatus::Clean
    } else if exit_code == Some(1) {
        ScanStatus::Findings
    } else {
        ScanStatus::Error
    };

    Ok(ScanOutcome {
        label: step.label.clone(),
        status,
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
