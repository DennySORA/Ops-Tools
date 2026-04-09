use super::tools::{AiTool, UpgradeCommand};
use crate::core::{load_config, OperationError, Result};
use crate::i18n::{self, keys};
use std::path::{Path, PathBuf};
use std::process::Command;

/// 套件升級器：處理 PackageManager 和 Custom 兩種升級方式
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
            UpgradeCommand::SourceBuild { .. } => {
                unreachable!("SourceBuild should be handled by SourceBuildExecutor")
            }
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

/// 從本地原始碼編譯安裝的執行器
pub struct SourceBuildExecutor;

impl SourceBuildExecutor {
    pub fn new() -> Self {
        Self
    }

    /// 從設定檔讀取 Codex 原始碼路徑
    fn resolve_source_dir(&self) -> Result<PathBuf> {
        let config = load_config()?.unwrap_or_default();
        let source_path = config
            .codex_source_path
            .ok_or_else(|| OperationError::Config {
                key: "codex_source_path".to_string(),
                message: i18n::t(keys::SOURCE_BUILD_PATH_NOT_SET).to_string(),
            })?;

        let path = PathBuf::from(&source_path);
        if !path.is_dir() {
            return Err(OperationError::Validation(crate::tr!(
                keys::SOURCE_BUILD_DIR_NOT_FOUND,
                path = source_path
            )));
        }

        Ok(path)
    }

    /// 在指定目錄執行 git pull --ff-only 取得最新原始碼
    fn pull_latest(&self, source_dir: &Path) -> Result<String> {
        run_command_in_dir("git", &["pull", "--ff-only"], source_dir, "git pull")
    }

    /// 執行 cargo build --release 編譯指定套件
    fn cargo_build(&self, source_dir: &Path, cargo_package: &str) -> Result<String> {
        run_command_in_dir(
            "cargo",
            &["build", "--release", "-p", cargo_package],
            source_dir,
            &format!("cargo build -p {cargo_package}"),
        )
    }

    /// 找到已安裝的二進位檔位置（透過 `which`）
    fn find_installed_binary(&self, binary_name: &str) -> Result<PathBuf> {
        let output = Command::new("which")
            .arg(binary_name)
            .output()
            .map_err(|e| OperationError::Command {
                command: format!("which {binary_name}"),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(PathBuf::from(path_str))
        } else {
            Err(OperationError::Validation(crate::tr!(
                keys::SOURCE_BUILD_BINARY_NOT_FOUND,
                name = binary_name
            )))
        }
    }

    /// 複製編譯好的二進位檔到安裝位置
    fn install_binary(
        &self,
        source_dir: &Path,
        binary_name: &str,
        install_target: &Path,
    ) -> Result<String> {
        let built_binary = source_dir.join("target").join("release").join(binary_name);

        if !built_binary.is_file() {
            return Err(OperationError::Validation(crate::tr!(
                keys::SOURCE_BUILD_ARTIFACT_NOT_FOUND,
                path = built_binary.display()
            )));
        }

        std::fs::copy(&built_binary, install_target).map_err(|e| OperationError::Io {
            path: install_target.display().to_string(),
            source: e,
        })?;

        Ok(crate::tr!(
            keys::SOURCE_BUILD_INSTALLED,
            source = built_binary.display(),
            target = install_target.display()
        ))
    }

    /// 執行完整的原始碼編譯安裝流程
    pub fn execute(&self, tool: &AiTool) -> Result<String> {
        let UpgradeCommand::SourceBuild {
            cargo_package,
            binary_name,
        } = tool.command
        else {
            unreachable!("SourceBuildExecutor only handles SourceBuild commands");
        };

        let source_dir = self.resolve_source_dir()?;
        let install_target = self.find_installed_binary(binary_name)?;

        let pull_output = self.pull_latest(&source_dir)?;
        let _build_output = self.cargo_build(&source_dir, cargo_package)?;
        let install_output = self.install_binary(&source_dir, cargo_package, &install_target)?;

        let summary = format!("{pull_output}\n{install_output}");
        Ok(summary)
    }
}

impl Default for SourceBuildExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 在指定目錄中執行外部指令並回傳 stdout
fn run_command_in_dir(
    program: &str,
    args: &[&str],
    working_dir: &Path,
    display_label: &str,
) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .output()
        .map_err(|e| OperationError::Command {
            command: display_label.to_string(),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(OperationError::Command {
            command: display_label.to_string(),
            message: stderr
                .lines()
                .next()
                .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::tool_upgrader::tools::{AiTool, UpgradeCommand, AI_TOOLS};

    #[test]
    fn test_build_command_for_npm_package() {
        let upgrader = PackageUpgrader::new();
        let gemini = AI_TOOLS
            .iter()
            .find(|t| {
                matches!(
                    t.command,
                    UpgradeCommand::PackageManager { manager, package }
                        if manager == "npm" && package == "@google/gemini-cli"
                )
            })
            .unwrap();

        let (program, args) = upgrader.build_command(gemini);
        assert_eq!(program, "npm");
        assert!(args.contains(&"install".to_string()));
        assert!(args.iter().any(|a| a.contains("@google/gemini-cli@latest")));
    }

    #[test]
    fn test_build_command_for_codex_bun() {
        let upgrader = PackageUpgrader::new();
        let codex = AI_TOOLS.iter().find(|t| t.name == "OpenAI Codex").unwrap();

        let (program, args) = upgrader.build_command(codex);
        assert_eq!(program, "bun");
        assert_eq!(
            args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "@openai/codex".to_string(),
            ]
        );
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

    #[test]
    fn test_source_build_entry_exists() {
        let source_build = AI_TOOLS
            .iter()
            .find(|t| t.name == "OpenAI Codex (Source Build)")
            .expect("Source Build entry should exist in AI_TOOLS");

        assert!(matches!(
            source_build.command,
            UpgradeCommand::SourceBuild {
                cargo_package: "codex-cli",
                binary_name: "codex",
            }
        ));
    }

    #[test]
    fn test_source_build_executor_rejects_missing_config() {
        // Without a configured source path, the executor should fail
        let executor = SourceBuildExecutor::new();
        let tool =
            AiTool::from_source_build("Test Tool", "test build", "some-package", "some-binary");
        let result = executor.execute(&tool);
        assert!(result.is_err());
    }

    #[test]
    fn test_source_build_executor_rejects_nonexistent_dir() {
        use std::env;
        use std::sync::{Mutex, OnceLock};

        // Use a lock to prevent concurrent env manipulation
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        let temp = tempfile::tempdir().unwrap();
        let old_xdg = env::var_os("XDG_CONFIG_HOME");
        env::set_var("XDG_CONFIG_HOME", temp.path());

        // Write config with a non-existent source path
        let config = crate::core::AppConfig {
            codex_source_path: Some("/nonexistent/path/to/codex".to_string()),
            ..Default::default()
        };
        let _ = crate::core::save_config(&config);

        let executor = SourceBuildExecutor::new();
        let tool = AiTool::from_source_build("Test", "test", "codex-tui", "codex");
        let result = executor.execute(&tool);
        assert!(result.is_err());

        // Restore env
        match old_xdg {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }
}
