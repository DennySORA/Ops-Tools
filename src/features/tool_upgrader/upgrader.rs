use super::tools::{AiTool, UpgradeCommand};
use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use std::process::Command;

/// 套件升級器
pub struct PackageUpgrader;

impl PackageUpgrader {
    pub fn new() -> Self {
        Self
    }

    /// 產生要執行的指令
    fn build_command(&self, tool: &AiTool) -> (String, Vec<String>) {
        match tool.command {
            UpgradeCommand::PackageManager { manager, package } => {
                let full_package = format!("{package}@latest");
                let args: Vec<String> = match manager {
                    "pnpm" => vec!["add", "-g", &full_package],
                    "yarn" => vec!["global", "add", &full_package],
                    _ => vec!["install", "-g", &full_package], // 預設 npm 參數格式
                }
                .into_iter()
                .map(String::from)
                .collect();
                (manager.to_string(), args)
            }
            UpgradeCommand::Custom { program, args } => (
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ),
        }
    }

    /// 升級指定工具到最新版本
    pub fn upgrade(&self, tool: &AiTool) -> Result<String> {
        let (program, args) = self.build_command(tool);
        let output =
            Command::new(&program)
                .args(&args)
                .output()
                .map_err(|e| OperationError::Command {
                    command: program.clone(),
                    message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
                })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let command_display = format!("{program} {}", args.join(" "));
            Err(OperationError::Command {
                command: command_display,
                message: stderr
                    .lines()
                    .next()
                    .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                    .to_string(),
            })
        }
    }
}

impl Default for PackageUpgrader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::tool_upgrader::tools::{UpgradeCommand, AI_TOOLS};

    #[test]
    fn test_build_command_for_npm_package() {
        let upgrader = PackageUpgrader::new();
        let codex = AI_TOOLS
            .iter()
            .find(|t| {
                matches!(
                    t.command,
                    UpgradeCommand::PackageManager { package, .. } if package == "@openai/codex"
                )
            })
            .unwrap();

        let (program, args) = upgrader.build_command(codex);
        assert_eq!(program, "npm");
        assert!(args.contains(&"install".to_string()));
        assert!(args.iter().any(|a| a.contains("@openai/codex@latest")));
    }

    #[test]
    fn test_build_command_for_custom() {
        let upgrader = PackageUpgrader::new();
        let claude = AI_TOOLS
            .iter()
            .find(|t| matches!(t.command, UpgradeCommand::Custom { .. }))
            .unwrap();

        let (program, args) = upgrader.build_command(claude);
        assert_eq!(program, "claude");
        assert_eq!(args, vec!["install".to_string()]);
    }
}
