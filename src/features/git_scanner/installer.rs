use crate::core::{OperationError, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::tools::{InstallStrategy, ScanTool};

pub enum InstallStatus {
    AlreadyInstalled(PathBuf),
    Installed(PathBuf),
    Failed(Vec<String>),
}

pub fn ensure_installed(tool: ScanTool) -> Result<InstallStatus> {
    if let Some(path) = resolve_tool_path(tool) {
        return Ok(InstallStatus::AlreadyInstalled(path));
    }

    let mut errors = Vec::new();
    let mut attempted = false;

    for strategy in tool.install_strategies() {
        if is_command_available(strategy.program).is_none() {
            continue;
        }

        attempted = true;
        match run_install_strategy(&strategy) {
            Ok(()) => {
                if let Some(path) = resolve_tool_path(tool) {
                    return Ok(InstallStatus::Installed(path));
                }
                errors.push(format!("{} 安裝完成但找不到指令", strategy.label));
            }
            Err(err) => {
                errors.push(format!("{} 失敗: {}", strategy.label, err));
            }
        }
    }

    if let Some(path) = resolve_tool_path(tool) {
        return Ok(InstallStatus::Installed(path));
    }

    if release_repo(tool).is_some() {
        attempted = true;
        match install_from_github_release(tool)? {
            ReleaseInstallOutcome::Installed(path) => {
                return Ok(InstallStatus::Installed(path));
            }
            ReleaseInstallOutcome::Skipped(reason) => {
                if !reason.is_empty() {
                    errors.push(reason);
                }
            }
            ReleaseInstallOutcome::Failed(reason) => {
                errors.push(reason);
            }
        }
    }

    if !attempted && errors.is_empty() {
        errors.push("未找到可用的安裝方式".to_string());
    }

    Ok(InstallStatus::Failed(errors))
}

pub fn resolve_tool_path(tool: ScanTool) -> Option<PathBuf> {
    if let Some(path) = is_command_available(tool.binary_name()) {
        return Some(path);
    }

    if let Some(path) = find_local_bin(tool.binary_name()) {
        return Some(path);
    }

    find_go_binary(tool.binary_name())
}

pub fn is_command_available(command: &str) -> Option<PathBuf> {
    let path = Path::new(command);
    if path.is_absolute() || command.contains(std::path::MAIN_SEPARATOR) {
        if path.is_file() {
            return Some(path.to_path_buf());
        }
        return None;
    }

    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return Some(candidate);
        }

        #[cfg(windows)]
        {
            let extensions = ["exe", "cmd", "bat"];
            for ext in extensions {
                let candidate = dir.join(format!("{}.{}", command, ext));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

fn run_install_strategy(strategy: &InstallStrategy) -> Result<()> {
    let mut program = strategy.program.to_string();
    let mut args = strategy.args.clone();

    if strategy.use_sudo && is_command_available("sudo").is_some() {
        let mut sudo_args = Vec::with_capacity(args.len() + 1);
        sudo_args.push(program);
        sudo_args.extend(args);
        program = "sudo".to_string();
        args = sudo_args;
    }

    let output = Command::new(&program)
        .args(&args)
        .output()
        .map_err(|err| OperationError::Command {
            command: program.clone(),
            message: format!("無法執行: {}", err),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(OperationError::Command {
            command: format!("{} {}", program, args.join(" ")),
            message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
        })
    }
}

enum ReleaseInstallOutcome {
    Installed(PathBuf),
    Skipped(String),
    Failed(String),
}

fn install_from_github_release(tool: ScanTool) -> Result<ReleaseInstallOutcome> {
    let Some(repo) = release_repo(tool) else {
        return Ok(ReleaseInstallOutcome::Skipped(String::new()));
    };

    let Some(platform) = Platform::detect() else {
        return Ok(ReleaseInstallOutcome::Skipped(
            "不支援的作業系統或架構".to_string(),
        ));
    };

    let Some(download) = fetch_release_asset(repo, &platform)? else {
        return Ok(ReleaseInstallOutcome::Failed(
            "無法找到對應的 GitHub Release 版本".to_string(),
        ));
    };

    let archive = download_to_temp(&download.url, download.extension)?;
    let extract_dir = extract_archive(&archive, download.extension)?;
    let binary = find_binary_in_dir(&extract_dir, tool.binary_name())
        .ok_or_else(|| OperationError::Command {
            command: tool.binary_name().to_string(),
            message: "解壓後找不到可執行檔".to_string(),
        })?;

    let installed_path = install_binary(&binary, tool.binary_name())?;
    Ok(ReleaseInstallOutcome::Installed(installed_path))
}

fn release_repo(tool: ScanTool) -> Option<&'static str> {
    match tool {
        ScanTool::Gitleaks => Some("gitleaks/gitleaks"),
        ScanTool::Trufflehog => Some("trufflesecurity/trufflehog"),
        ScanTool::GitSecrets => None,
    }
}

struct Platform {
    os_tokens: Vec<&'static str>,
    arch_tokens: Vec<&'static str>,
    prefer_zip: bool,
}

impl Platform {
    fn detect() -> Option<Self> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let os_tokens = match os {
            "linux" => vec!["linux"],
            "macos" => vec!["darwin", "macos"],
            "windows" => vec!["windows"],
            _ => return None,
        };

        let arch_tokens = match arch {
            "x86_64" => vec!["x86_64", "amd64", "x64"],
            "aarch64" => vec!["aarch64", "arm64"],
            "arm" => vec!["armv7", "armv6", "arm"],
            _ => return None,
        };

        Some(Self {
            os_tokens,
            arch_tokens,
            prefer_zip: os == "windows",
        })
    }
}

#[derive(Clone)]
struct ReleaseAsset {
    url: String,
    extension: ArchiveKind,
}

#[derive(Clone, Copy)]
enum ArchiveKind {
    TarGz,
    Zip,
    Unknown,
}

fn fetch_release_asset(repo: &str, platform: &Platform) -> Result<Option<ReleaseAsset>> {
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let json = fetch_url(&api_url)?;
    let payload: serde_json::Value = serde_json::from_str(&json).map_err(|err| {
        OperationError::Config {
            key: api_url.clone(),
            message: format!("解析 Release 失敗: {}", err),
        }
    })?;

    let assets = payload
        .get("assets")
        .and_then(|val| val.as_array())
        .ok_or_else(|| OperationError::Config {
            key: api_url.clone(),
            message: "Release 資料缺少 assets".to_string(),
        })?;

    let mut matches = Vec::new();

    for asset in assets {
        let Some(name) = asset.get("name").and_then(|val| val.as_str()) else {
            continue;
        };
        let Some(url) = asset
            .get("browser_download_url")
            .and_then(|val| val.as_str())
        else {
            continue;
        };

        let name_lower = name.to_ascii_lowercase();
        if !platform
            .os_tokens
            .iter()
            .any(|token| name_lower.contains(token))
        {
            continue;
        }
        if !platform
            .arch_tokens
            .iter()
            .any(|token| name_lower.contains(token))
        {
            continue;
        }

        let extension = if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") {
            ArchiveKind::TarGz
        } else if name_lower.ends_with(".zip") {
            ArchiveKind::Zip
        } else {
            ArchiveKind::Unknown
        };

        if matches!(extension, ArchiveKind::Unknown) {
            continue;
        }

        matches.push(ReleaseAsset {
            url: url.to_string(),
            extension,
        });
    }

    if matches.is_empty() {
        return Ok(None);
    }

    if platform.prefer_zip {
        if let Some(asset) = matches
            .iter()
            .find(|asset| matches!(asset.extension, ArchiveKind::Zip))
        {
            return Ok(Some(asset.clone()));
        }
    } else if let Some(asset) = matches
        .iter()
        .find(|asset| matches!(asset.extension, ArchiveKind::TarGz))
    {
        return Ok(Some(asset.clone()));
    }

    Ok(Some(matches[0].clone()))
}

fn fetch_url(url: &str) -> Result<String> {
    if let Some(path) = is_command_available("curl") {
        let output = Command::new(path)
            .args([
                "-fsSL",
                "-H",
                "Accept: application/vnd.github+json",
                "-H",
                "User-Agent: ops-tools",
                url,
            ])
            .output()
            .map_err(|err| OperationError::Command {
                command: "curl".to_string(),
                message: format!("無法執行: {}", err),
            })?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(OperationError::Command {
            command: "curl".to_string(),
            message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
        });
    }

    if let Some(path) = is_command_available("wget") {
        let output = Command::new(path)
            .args(["-q", "-O", "-", url])
            .output()
            .map_err(|err| OperationError::Command {
                command: "wget".to_string(),
                message: format!("無法執行: {}", err),
            })?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(OperationError::Command {
            command: "wget".to_string(),
            message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
        });
    }

    Err(OperationError::Command {
        command: "curl/wget".to_string(),
        message: "找不到下載工具".to_string(),
    })
}

fn download_to_temp(url: &str, extension: ArchiveKind) -> Result<PathBuf> {
    let temp_dir = env::temp_dir().join("ops-tools").join("git-scanner");
    std::fs::create_dir_all(&temp_dir).map_err(|err| OperationError::Io {
        path: temp_dir.display().to_string(),
        source: err,
    })?;

    let file_name = match extension {
        ArchiveKind::TarGz => "download.tar.gz",
        ArchiveKind::Zip => "download.zip",
        ArchiveKind::Unknown => "download.bin",
    };
    let target = temp_dir.join(file_name);

    if let Some(path) = is_command_available("curl") {
        let output = Command::new(path)
            .args(["-fsSL", "-o", target.to_str().unwrap_or_default(), url])
            .output()
            .map_err(|err| OperationError::Command {
                command: "curl".to_string(),
                message: format!("無法執行: {}", err),
            })?;
        if output.status.success() {
            return Ok(target);
        }
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(OperationError::Command {
            command: "curl".to_string(),
            message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
        });
    }

    if let Some(path) = is_command_available("wget") {
        let output = Command::new(path)
            .args([
                "-q",
                "-O",
                target.to_str().unwrap_or_default(),
                url,
            ])
            .output()
            .map_err(|err| OperationError::Command {
                command: "wget".to_string(),
                message: format!("無法執行: {}", err),
            })?;
        if output.status.success() {
            return Ok(target);
        }
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(OperationError::Command {
            command: "wget".to_string(),
            message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
        });
    }

    Err(OperationError::Command {
        command: "curl/wget".to_string(),
        message: "找不到下載工具".to_string(),
    })
}

fn extract_archive(path: &Path, extension: ArchiveKind) -> Result<PathBuf> {
    let extract_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("extract");
    std::fs::create_dir_all(&extract_dir).map_err(|err| OperationError::Io {
        path: extract_dir.display().to_string(),
        source: err,
    })?;

    match extension {
        ArchiveKind::TarGz => {
            let Some(tar_path) = is_command_available("tar") else {
                return Err(OperationError::Command {
                    command: "tar".to_string(),
                    message: "找不到 tar".to_string(),
                });
            };
            let output = Command::new(tar_path)
                .args([
                    "-xzf",
                    path.to_str().unwrap_or_default(),
                    "-C",
                    extract_dir.to_str().unwrap_or_default(),
                ])
                .output()
                .map_err(|err| OperationError::Command {
                    command: "tar".to_string(),
                    message: format!("無法執行: {}", err),
                })?;
            if output.status.success() {
                Ok(extract_dir)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(OperationError::Command {
                    command: "tar".to_string(),
                    message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
                })
            }
        }
        ArchiveKind::Zip => {
            let Some(unzip_path) = is_command_available("unzip") else {
                return Err(OperationError::Command {
                    command: "unzip".to_string(),
                    message: "找不到 unzip".to_string(),
                });
            };
            let output = Command::new(unzip_path)
                .args([
                    "-q",
                    path.to_str().unwrap_or_default(),
                    "-d",
                    extract_dir.to_str().unwrap_or_default(),
                ])
                .output()
                .map_err(|err| OperationError::Command {
                    command: "unzip".to_string(),
                    message: format!("無法執行: {}", err),
                })?;
            if output.status.success() {
                Ok(extract_dir)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(OperationError::Command {
                    command: "unzip".to_string(),
                    message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
                })
            }
        }
        ArchiveKind::Unknown => Ok(extract_dir),
    }
}

fn find_binary_in_dir(root: &Path, binary: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = std::fs::read_dir(&path).ok()?;
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }

            let name = entry_path.file_name()?.to_string_lossy().to_string();
            if name == binary {
                return Some(entry_path);
            }

            #[cfg(windows)]
            if name == format!("{}.exe", binary) {
                return Some(entry_path);
            }
        }
    }

    None
}

fn install_binary(source: &Path, binary: &str) -> Result<PathBuf> {
    let Some(target_dir) = local_bin_dir() else {
        return Err(OperationError::Command {
            command: "install".to_string(),
            message: "找不到可寫入的安裝目錄".to_string(),
        });
    };

    std::fs::create_dir_all(&target_dir).map_err(|err| OperationError::Io {
        path: target_dir.display().to_string(),
        source: err,
    })?;

    let target_path = target_dir.join(binary);
    std::fs::copy(source, &target_path).map_err(|err| OperationError::Io {
        path: target_path.display().to_string(),
        source: err,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target_path)
            .map_err(|err| OperationError::Io {
                path: target_path.display().to_string(),
                source: err,
            })?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target_path, perms).map_err(|err| OperationError::Io {
            path: target_path.display().to_string(),
            source: err,
        })?;
    }

    Ok(target_path)
}

fn find_local_bin(binary: &str) -> Option<PathBuf> {
    let Some(dir) = local_bin_dir() else {
        return None;
    };
    let candidate = dir.join(binary);
    if candidate.is_file() {
        return Some(candidate);
    }
    None
}

fn local_bin_dir() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".local").join("bin"))
}

fn find_go_binary(binary: &str) -> Option<PathBuf> {
    let Some(go_bin) = go_bin_dir() else {
        return None;
    };
    let candidate = go_bin.join(binary);
    if candidate.is_file() {
        return Some(candidate);
    }

    #[cfg(windows)]
    {
        let candidate = go_bin.join(format!("{}.exe", binary));
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn go_bin_dir() -> Option<PathBuf> {
    if let Ok(gobin) = env::var("GOBIN") {
        if !gobin.trim().is_empty() {
            return Some(PathBuf::from(gobin));
        }
    }

    if is_command_available("go").is_none() {
        return None;
    }

    let gobin = run_go_env("GOBIN")?;
    if !gobin.is_empty() {
        return Some(PathBuf::from(gobin));
    }

    let gopath = run_go_env("GOPATH")?;
    if !gopath.is_empty() {
        return Some(PathBuf::from(gopath).join("bin"));
    }

    None
}

fn run_go_env(key: &str) -> Option<String> {
    let output = Command::new("go").args(["env", key]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
