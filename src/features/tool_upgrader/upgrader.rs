use super::tools::{AiTool, UpgradeCommand};
use crate::core::{OperationError, Result, load_config};
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

/// Codex source build executor.
/// Reads config for source dir, private remote, and feature branch.
/// Full workflow: pull upstream → checkout branch → rebase → build → install → push.
pub struct SourceBuildExecutor;

/// Default feature branch name when not configured.
const DEFAULT_FEATURE_BRANCH: &str = "feat/cron-scheduler";

impl SourceBuildExecutor {
    /// Resolve source directory from env `CODEX_SOURCE_PATH` or config `codex_source_path`.
    /// Returns None if not configured (caller should fallback to normal upgrade).
    pub fn resolve_source_dir() -> Option<PathBuf> {
        if let Ok(env_path) = std::env::var("CODEX_SOURCE_PATH") {
            let p = PathBuf::from(&env_path);
            if p.is_dir() {
                return Some(p);
            }
        }
        if let Ok(Some(config)) = load_config()
            && let Some(cfg_path) = config.codex_source_path
        {
            let p = PathBuf::from(&cfg_path);
            if p.is_dir() {
                return Some(p);
            }
        }
        None
    }

    /// Full workflow:
    /// 1. git fetch upstream (official OpenAI)
    /// 2. git checkout main && git merge upstream/main --ff-only
    /// 3. git checkout <feature_branch> && git rebase main
    /// 4. cargo build --release -p <cargo_package>
    /// 5. Copy binary to installed location
    /// 6. git push private main && git push private <feature_branch>
    pub fn execute_source_build(
        source_dir: &Path,
        cargo_package: &str,
        binary_name: &str,
    ) -> Result<String> {
        let config = load_config()?.unwrap_or_default();
        let feature_branch = config
            .codex_feature_branch
            .as_deref()
            .unwrap_or(DEFAULT_FEATURE_BRANCH);
        let has_private = config.codex_private_remote.is_some();

        let mut log = Vec::new();

        // 1. Fetch upstream
        let fetch_out = run_command_in_dir(
            "git",
            &["fetch", "upstream"],
            source_dir,
            "git fetch upstream",
        )?;
        log.push(format!("[fetch] {fetch_out}"));

        // 2. Update main from upstream
        run_command_in_dir(
            "git",
            &["checkout", "main"],
            source_dir,
            "git checkout main",
        )?;
        let merge_out = run_command_in_dir(
            "git",
            &["merge", "upstream/main", "--ff-only"],
            source_dir,
            "git merge upstream/main",
        )?;
        log.push(format!("[main] {merge_out}"));

        // 3. Rebase feature branch onto updated main
        run_command_in_dir(
            "git",
            &["checkout", feature_branch],
            source_dir,
            &format!("git checkout {feature_branch}"),
        )?;
        let rebase_out = run_command_in_dir(
            "git",
            &["rebase", "main"],
            source_dir,
            &format!("git rebase main (on {feature_branch})"),
        )?;
        log.push(format!("[rebase] {rebase_out}"));

        // 4. Build
        run_command_in_dir(
            "cargo",
            &["build", "--release", "-p", cargo_package],
            source_dir,
            &format!("cargo build -p {cargo_package}"),
        )?;
        log.push("[build] ok".to_string());

        // 5. Install binary
        let install_target = find_binary_path(binary_name)?;
        let built = source_dir.join("target").join("release").join(binary_name);
        if !built.is_file() {
            return Err(OperationError::Validation(crate::tr!(
                keys::SOURCE_BUILD_ARTIFACT_NOT_FOUND,
                path = built.display()
            )));
        }
        std::fs::copy(&built, &install_target).map_err(|e| OperationError::Io {
            path: install_target.display().to_string(),
            source: e,
        })?;
        log.push(crate::tr!(
            keys::SOURCE_BUILD_INSTALLED,
            source = built.display(),
            target = install_target.display()
        ));

        // 6. Push to private remote (best-effort, don't fail the build)
        if has_private {
            let _ = run_command_in_dir(
                "git",
                &["push", "private", "main"],
                source_dir,
                "git push private main",
            );
            let push_result = run_command_in_dir(
                "git",
                &["push", "private", feature_branch, "--force-with-lease"],
                source_dir,
                &format!("git push private {feature_branch}"),
            );
            match push_result {
                Ok(_) => log.push(format!("[push] private/{feature_branch} synced")),
                Err(_) => log.push("[push] private sync skipped (not critical)".to_string()),
            }
        }

        Ok(log.join("\n"))
    }
}

/// 透過 `which` 找到已安裝的二進位檔位置
fn find_binary_path(binary_name: &str) -> Result<PathBuf> {
    let output = Command::new("which")
        .arg(binary_name)
        .output()
        .map_err(|e| OperationError::Command {
            command: format!("which {binary_name}"),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
        })?;

    if output.status.success() {
        Ok(PathBuf::from(
            String::from_utf8_lossy(&output.stdout).trim(),
        ))
    } else {
        Err(OperationError::Validation(crate::tr!(
            keys::SOURCE_BUILD_BINARY_NOT_FOUND,
            name = binary_name
        )))
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
    use crate::features::tool_upgrader::tools::{AI_TOOLS, UpgradeCommand};

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
    fn test_resolve_source_dir_from_env() {
        use std::env;
        let dir = env::temp_dir();
        unsafe { env::set_var("CODEX_SOURCE_PATH", dir.to_str().unwrap()) };
        let result = SourceBuildExecutor::resolve_source_dir();
        unsafe { env::remove_var("CODEX_SOURCE_PATH") };
        assert!(result.is_some());
        assert_eq!(result.unwrap(), dir);
    }

    #[test]
    fn test_resolve_source_dir_none_when_unset() {
        use std::env;
        unsafe { env::remove_var("CODEX_SOURCE_PATH") };
        let _ = SourceBuildExecutor::resolve_source_dir();
    }

    #[test]
    fn test_resolve_source_dir_ignores_nonexistent_env() {
        use std::env;
        unsafe { env::set_var("CODEX_SOURCE_PATH", "/nonexistent/path/12345") };
        let result = SourceBuildExecutor::resolve_source_dir();
        unsafe { env::remove_var("CODEX_SOURCE_PATH") };
        assert!(result.as_ref().is_none_or(|p| p.is_dir()));
    }
}
