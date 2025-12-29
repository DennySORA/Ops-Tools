mod installer;
mod scanner;
mod tools;

use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use installer::{ensure_installed, is_command_available, resolve_tool_path, InstallStatus};
use scanner::{run_scans, ScanStatus};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use tools::all_tools;

/// 執行 Git 安全掃描功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::GIT_SCANNER_HEADER));

    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            console.error(&crate::tr!(
                keys::GIT_SCANNER_CURRENT_DIR_FAILED,
                error = err
            ));
            return;
        }
    };

    let Some(repo_root) = find_git_root(&current_dir) else {
        console.error(i18n::t(keys::GIT_SCANNER_NOT_GIT_REPO));
        return;
    };

    if is_command_available("git").is_none() {
        console.error(i18n::t(keys::GIT_SCANNER_GIT_NOT_FOUND));
        return;
    }

    console.info(&crate::tr!(
        keys::GIT_SCANNER_SCAN_DIR,
        path = repo_root.display()
    ));
    console.info(i18n::t(keys::GIT_SCANNER_STRICT_MODE));
    console.blank_line();

    let worktree_snapshot = match build_worktree_snapshot(&repo_root, &console) {
        Ok(snapshot) => snapshot,
        Err(err) => {
            console.error(&err.to_string());
            return;
        }
    };

    let tools = all_tools();
    console.info(i18n::t(keys::GIT_SCANNER_TOOLS_INTRO));
    for tool in &tools {
        let status = if resolve_tool_path(*tool).is_some() {
            i18n::t(keys::GIT_SCANNER_STATUS_INSTALLED)
        } else {
            i18n::t(keys::GIT_SCANNER_STATUS_MISSING)
        };
        console.list_item("🔎", &format!("{} ({})", tool.display_name(), status));
    }

    if !prompts.confirm_with_options(i18n::t(keys::GIT_SCANNER_CONFIRM_INSTALL), true) {
        console.warning(i18n::t(keys::GIT_SCANNER_CANCELLED));
        return;
    }

    console.blank_line();

    let mut install_attempted = 0;
    let mut install_success = 0;
    let mut install_failed = 0;

    for tool in &tools {
        if resolve_tool_path(*tool).is_some() {
            console.success_item(&format!(
                "{} {}",
                tool.display_name(),
                i18n::t(keys::GIT_SCANNER_STATUS_INSTALLED)
            ));
            continue;
        }

        console.info(&crate::tr!(
            keys::GIT_SCANNER_INSTALLING,
            tool = tool.display_name()
        ));
        install_attempted += 1;
        match ensure_installed(*tool) {
            Ok(InstallStatus::Installed(path)) => {
                console.success_item(&crate::tr!(
                    keys::GIT_SCANNER_INSTALL_DONE,
                    tool = tool.display_name(),
                    path = path.display()
                ));
                install_success += 1;
            }
            Ok(InstallStatus::AlreadyInstalled(path)) => {
                console.success_item(&crate::tr!(
                    keys::GIT_SCANNER_INSTALL_ALREADY,
                    tool = tool.display_name(),
                    path = path.display()
                ));
                install_success += 1;
            }
            Ok(InstallStatus::Failed(errors)) => {
                let message = errors.join("; ");
                console.error_item(
                    &crate::tr!(keys::GIT_SCANNER_INSTALL_FAILED, tool = tool.display_name()),
                    &message,
                );
                install_failed += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(keys::GIT_SCANNER_INSTALL_FAILED, tool = tool.display_name()),
                    &err.to_string(),
                );
                install_failed += 1;
            }
        }
    }

    if install_attempted > 0 {
        console.show_summary(
            i18n::t(keys::GIT_SCANNER_INSTALL_SUMMARY),
            install_success,
            install_failed,
        );
        console.blank_line();
    }

    let mut scan_success = 0;
    let mut scan_failed = 0;
    let mut has_findings = false;

    for tool in &tools {
        let Some(_) = resolve_tool_path(*tool) else {
            console.warning(&crate::tr!(
                keys::GIT_SCANNER_SKIP_TOOL,
                tool = tool.display_name()
            ));
            continue;
        };

        console.info(&crate::tr!(
            keys::GIT_SCANNER_START_SCAN,
            tool = tool.display_name()
        ));
        match run_scans(*tool, &repo_root, worktree_snapshot.root()) {
            Ok(outcomes) => {
                for outcome in outcomes {
                    console.separator();
                    console.info(&crate::tr!(
                        keys::GIT_SCANNER_STDOUT_TITLE,
                        label = outcome.label
                    ));
                    if outcome.stdout.trim().is_empty() {
                        console.raw(&format!("{}\n", i18n::t(keys::GIT_SCANNER_NO_OUTPUT)));
                    } else {
                        console.raw(&ensure_trailing_newline(&outcome.stdout));
                    }
                    console.info(&crate::tr!(
                        keys::GIT_SCANNER_STDERR_TITLE,
                        label = outcome.label
                    ));
                    if outcome.stderr.trim().is_empty() {
                        console.raw(&format!("{}\n", i18n::t(keys::GIT_SCANNER_NO_OUTPUT)));
                    } else {
                        console.raw(&ensure_trailing_newline(&outcome.stderr));
                    }

                    match outcome.status {
                        ScanStatus::Clean => {
                            console.success_item(&crate::tr!(
                                keys::GIT_SCANNER_PASSED,
                                label = outcome.label
                            ));
                            scan_success += 1;
                        }
                        ScanStatus::Findings => {
                            has_findings = true;
                            console.error_item(
                                &crate::tr!(keys::GIT_SCANNER_FINDINGS, label = outcome.label),
                                &format_exit_code(outcome.exit_code),
                            );
                            scan_failed += 1;
                        }
                        ScanStatus::Error => {
                            console.error_item(
                                &crate::tr!(keys::GIT_SCANNER_SCAN_FAILED, label = outcome.label),
                                &format_exit_code(outcome.exit_code),
                            );
                            scan_failed += 1;
                        }
                    }
                }
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(keys::GIT_SCANNER_SCAN_FAILED, label = tool.display_name()),
                    &err.to_string(),
                );
                scan_failed += 1;
            }
        }

        console.blank_line();
    }

    console.show_summary(
        i18n::t(keys::GIT_SCANNER_SCAN_SUMMARY),
        scan_success,
        scan_failed,
    );
    if has_findings {
        console.warning(i18n::t(keys::GIT_SCANNER_FINDINGS_WARNING));
    }
}

fn format_exit_code(exit_code: Option<i32>) -> String {
    match exit_code {
        Some(code) => crate::tr!(keys::GIT_SCANNER_EXIT_CODE, code = code),
        None => i18n::t(keys::GIT_SCANNER_EXIT_CODE_UNKNOWN).to_string(),
    }
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let git_path = dir.join(".git");
        if git_path.is_dir() || git_path.is_file() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

struct WorktreeSnapshot {
    root: PathBuf,
    cleanup_path: PathBuf,
}

impl WorktreeSnapshot {
    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for WorktreeSnapshot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.cleanup_path);
    }
}

fn build_worktree_snapshot(repo_root: &Path, console: &Console) -> Result<WorktreeSnapshot> {
    let snapshot_root = create_temp_dir()?;

    let tracked = git_list_tracked(repo_root)?;
    if tracked.is_empty() {
        console.warning(i18n::t(keys::GIT_SCANNER_NO_TRACKED_FILES));
        return Ok(WorktreeSnapshot {
            root: snapshot_root.clone(),
            cleanup_path: snapshot_root,
        });
    }

    let ignored = git_list_ignored(repo_root, &tracked)?;
    let filtered: Vec<String> = tracked
        .into_iter()
        .filter(|path| !ignored.contains(path))
        .collect();

    if filtered.is_empty() {
        console.warning(i18n::t(keys::GIT_SCANNER_ALL_IGNORED));
        return Ok(WorktreeSnapshot {
            root: snapshot_root.clone(),
            cleanup_path: snapshot_root,
        });
    }
    for rel_path in filtered {
        let source = repo_root.join(&rel_path);
        if !source.is_file() {
            continue;
        }
        let destination = snapshot_root.join(&rel_path);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|err| OperationError::Io {
                path: parent.display().to_string(),
                source: err,
            })?;
        }

        if std::fs::hard_link(&source, &destination).is_err() {
            std::fs::copy(&source, &destination).map_err(|err| OperationError::Io {
                path: destination.display().to_string(),
                source: err,
            })?;
        }
    }

    Ok(WorktreeSnapshot {
        root: snapshot_root.clone(),
        cleanup_path: snapshot_root,
    })
}

fn create_temp_dir() -> Result<PathBuf> {
    let base = std::env::temp_dir().join("ops-tools");
    std::fs::create_dir_all(&base).map_err(|err| OperationError::Io {
        path: base.display().to_string(),
        source: err,
    })?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = base.join(format!("git-scan-{}-{}", std::process::id(), timestamp));
    std::fs::create_dir_all(&dir).map_err(|err| OperationError::Io {
        path: dir.display().to_string(),
        source: err,
    })?;
    Ok(dir)
}

fn git_list_tracked(repo_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["-C", &repo_root.display().to_string(), "ls-files", "-z"])
        .output()
        .map_err(|err| OperationError::Command {
            command: "git ls-files".to_string(),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = err),
        })?;

    if !output.status.success() {
        return Err(OperationError::Command {
            command: "git ls-files".to_string(),
            message: String::from_utf8_lossy(&output.stderr)
                .lines()
                .next()
                .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                .to_string(),
        });
    }

    Ok(split_nul(&output.stdout))
}

fn git_list_ignored(
    repo_root: &Path,
    paths: &[String],
) -> Result<std::collections::HashSet<String>> {
    if paths.is_empty() {
        return Ok(std::collections::HashSet::new());
    }

    let mut child = Command::new("git")
        .args([
            "-C",
            &repo_root.display().to_string(),
            "check-ignore",
            "-z",
            "--stdin",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| OperationError::Command {
            command: "git check-ignore".to_string(),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = err),
        })?;

    if let Some(stdin) = child.stdin.as_mut() {
        let mut buffer = Vec::new();
        for path in paths {
            buffer.extend_from_slice(path.as_bytes());
            buffer.push(0);
        }
        use std::io::Write;
        stdin.write_all(&buffer).map_err(|err| OperationError::Io {
            path: "stdin".to_string(),
            source: err,
        })?;
    }

    let output = child
        .wait_with_output()
        .map_err(|err| OperationError::Command {
            command: "git check-ignore".to_string(),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = err),
        })?;

    let code = output.status.code().unwrap_or(-1);
    if code != 0 && code != 1 {
        return Err(OperationError::Command {
            command: "git check-ignore".to_string(),
            message: String::from_utf8_lossy(&output.stderr)
                .lines()
                .next()
                .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                .to_string(),
        });
    }

    let ignored = split_nul(&output.stdout)
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    Ok(ignored)
}

fn split_nul(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|b| *b == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect()
}

fn ensure_trailing_newline(text: &str) -> String {
    if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{}\n", text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_git_root_current_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let result = find_git_root(dir.path());
        assert_eq!(result.as_deref(), Some(dir.path()));
    }

    #[test]
    fn test_find_git_root_nested_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        let result = find_git_root(&nested);
        assert_eq!(result.as_deref(), Some(dir.path()));
    }

    #[test]
    fn test_find_git_root_missing() {
        let dir = tempfile::tempdir().unwrap();
        let result = find_git_root(dir.path());
        assert!(result.is_none());
    }
}
