use crate::features::system_updater::domain::command::CommandSpec;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("[{code}] {message}")]
    Validation { code: &'static str, message: String },
    #[error("[{code}] {message}")]
    SafetyViolation { code: &'static str, message: String },
}

impl DomainError {
    pub fn validation(code: &'static str, message: impl Into<String>) -> Self {
        Self::Validation {
            code,
            message: message.into(),
        }
    }

    pub fn safety(code: &'static str, message: impl Into<String>) -> Self {
        Self::SafetyViolation {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InfrastructureError {
    #[error("[{code}] command failed: {command} (exit {exit_code:?}) {detail}")]
    CommandFailed {
        code: &'static str,
        command: String,
        exit_code: Option<i32>,
        detail: String,
    },
    #[error("[{code}] failed to start command: {command}: {detail}")]
    CommandSpawn {
        code: &'static str,
        command: String,
        detail: String,
    },
    #[error("[{code}] command timed out: {command} after {timeout_ms} ms")]
    CommandTimedOut {
        code: &'static str,
        command: String,
        timeout_ms: u64,
    },
    #[error("[{code}] filesystem error on {path}: {detail}")]
    FileSystem {
        code: &'static str,
        path: PathBuf,
        detail: String,
    },
    #[error("[{code}] serialization error: {detail}")]
    Serialization { code: &'static str, detail: String },
    #[error("[{code}] probe failed: {operation}: {detail}")]
    Probe {
        code: &'static str,
        operation: String,
        detail: String,
    },
}

impl InfrastructureError {
    pub fn command_failed(
        code: &'static str,
        command: &CommandSpec,
        exit_code: Option<i32>,
        detail: impl Into<String>,
    ) -> Self {
        Self::CommandFailed {
            code,
            command: command.display(),
            exit_code,
            detail: detail.into(),
        }
    }

    pub fn command_spawn(
        code: &'static str,
        command: &CommandSpec,
        detail: impl Into<String>,
    ) -> Self {
        Self::CommandSpawn {
            code,
            command: command.display(),
            detail: detail.into(),
        }
    }

    pub fn command_timed_out(code: &'static str, command: &CommandSpec, timeout_ms: u64) -> Self {
        Self::CommandTimedOut {
            code,
            command: command.display(),
            timeout_ms,
        }
    }

    pub fn filesystem(
        code: &'static str,
        path: impl Into<PathBuf>,
        detail: impl Into<String>,
    ) -> Self {
        Self::FileSystem {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }

    pub fn serialization(code: &'static str, detail: impl Into<String>) -> Self {
        Self::Serialization {
            code,
            detail: detail.into(),
        }
    }

    pub fn probe(
        code: &'static str,
        operation: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::Probe {
            code,
            operation: operation.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    #[error("{0}")]
    Domain(#[from] DomainError),
    #[error("{0}")]
    Infrastructure(#[from] InfrastructureError),
}

pub type AppResult<T> = Result<T, ApplicationError>;
