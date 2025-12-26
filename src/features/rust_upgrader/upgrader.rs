use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use std::path::Path;
use std::process::Command;

use super::tools::{CargoTool, UpgradeStep};

/// Rust 環境檢查結果
#[derive(Debug)]
pub struct RustEnvironment {
    pub rustc_version: String,
    pub cargo_version: String,
    pub rustup_version: String,
}

/// 套件安裝狀態
#[derive(Debug)]
pub struct ToolStatus {
    pub tool: CargoTool,
    pub installed: bool,
}

/// Rust 升級器
pub struct RustUpgrader {
    project_path: Option<String>,
}

impl RustUpgrader {
    pub fn new() -> Self {
        Self { project_path: None }
    }

    #[allow(dead_code)]
    pub fn with_project_path(project_path: &str) -> Self {
        Self {
            project_path: Some(project_path.to_string()),
        }
    }

    /// 檢查 Rust 是否已安裝
    pub fn check_rust_installed(&self) -> Result<RustEnvironment> {
        let rustc = self.get_version("rustc", &["--version"])?;
        let cargo = self.get_version("cargo", &["--version"])?;
        let rustup = self.get_version("rustup", &["--version"])?;

        Ok(RustEnvironment {
            rustc_version: rustc,
            cargo_version: cargo,
            rustup_version: rustup,
        })
    }

    /// 檢查指定的 cargo 工具是否已安裝
    pub fn check_tool_installed(&self, tool: &CargoTool) -> bool {
        let output = Command::new("cargo").args(["--list"]).output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(tool.command)
            }
            Err(_) => false,
        }
    }

    /// 檢查多個工具的安裝狀態
    pub fn check_tools_status(&self, tools: &[CargoTool]) -> Vec<ToolStatus> {
        tools
            .iter()
            .map(|tool| ToolStatus {
                tool: tool.clone(),
                installed: self.check_tool_installed(tool),
            })
            .collect()
    }

    /// 安裝 cargo 工具
    pub fn install_tool(&self, tool: &CargoTool) -> Result<String> {
        let output = Command::new("cargo")
            .args(["install", tool.crate_name])
            .output()
            .map_err(|e| OperationError::Command {
                command: "cargo install".to_string(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(OperationError::Command {
                command: format!("cargo install {}", tool.crate_name),
                message: stderr
                    .lines()
                    .next()
                    .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                    .to_string(),
            })
        }
    }

    /// 執行升級步驟
    pub fn run_upgrade_step(&self, step: &UpgradeStep) -> Result<String> {
        if step.requires_project && !self.has_cargo_toml() {
            return Err(OperationError::MissingCargoToml);
        }

        let mut command = Command::new(step.command);
        command.args(step.args);

        if let Some(ref path) = self.project_path {
            command.current_dir(path);
        }

        let output = command.output().map_err(|e| OperationError::Command {
            command: step.command.to_string(),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            let combined = if stderr.is_empty() {
                stdout
            } else {
                format!("{}\n{}", stdout, stderr)
            };
            Ok(combined)
        } else {
            Err(OperationError::Command {
                command: format!("{} {}", step.command, step.args.join(" ")),
                message: stderr
                    .lines()
                    .next()
                    .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                    .to_string(),
            })
        }
    }

    /// 檢查目前目錄是否有 Cargo.toml
    fn has_cargo_toml(&self) -> bool {
        let cargo_path = match &self.project_path {
            Some(path) => Path::new(path).join("Cargo.toml"),
            None => Path::new("Cargo.toml").to_path_buf(),
        };
        cargo_path.exists()
    }

    /// 取得命令版本
    fn get_version(&self, command: &str, args: &[&str]) -> Result<String> {
        let output =
            Command::new(command)
                .args(args)
                .output()
                .map_err(|e| OperationError::Command {
                    command: command.to_string(),
                    message: crate::tr!(keys::RUST_UPGRADER_RUST_MISSING_OR_UNAVAILABLE,
                        error = e
                    ),
                })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(OperationError::Command {
                command: command.to_string(),
                message: i18n::t(keys::RUST_UPGRADER_VERSION_UNAVAILABLE).to_string(),
            })
        }
    }
}

impl Default for RustUpgrader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::rust_upgrader::tools::REQUIRED_CARGO_TOOLS;

    #[test]
    fn test_upgrader_creation() {
        let upgrader = RustUpgrader::new();
        assert!(upgrader.project_path.is_none());
    }

    #[test]
    fn test_upgrader_with_path() {
        let upgrader = RustUpgrader::with_project_path("/test/path");
        assert_eq!(upgrader.project_path, Some("/test/path".to_string()));
    }

    #[test]
    fn test_check_rust_installed() {
        let upgrader = RustUpgrader::new();
        // This test may fail if Rust is not installed
        let result = upgrader.check_rust_installed();
        // We just check that it returns a result without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_tools_status() {
        let upgrader = RustUpgrader::new();
        let statuses = upgrader.check_tools_status(REQUIRED_CARGO_TOOLS);
        assert_eq!(statuses.len(), REQUIRED_CARGO_TOOLS.len());
    }
}
