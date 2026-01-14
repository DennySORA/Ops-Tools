//! Shell 執行與檔案系統工具
//!
//! 提供指令執行、檔案下載、壓縮解壓等底層操作

use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{ActionContext, PackageManager, SupportedOs};

// ============================================================================
// 指令執行
// ============================================================================

/// 執行外部指令
pub fn run_command(
    ctx: &ActionContext,
    program: &str,
    args: &[&str],
    use_sudo: bool,
) -> Result<String> {
    let mut args_vec: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let mut program = program.to_string();

    if use_sudo && ctx.sudo_available {
        args_vec.insert(0, program.clone());
        program = "sudo".to_string();
    }

    let output = Command::new(&program)
        .args(&args_vec)
        .output()
        .map_err(|err| OperationError::Command {
            command: program.clone(),
            message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = err),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(OperationError::Command {
            command: format!("{} {}", program, args_vec.join(" ")),
            message: stderr
                .lines()
                .next()
                .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                .to_string(),
        })
    }
}

/// 以路徑執行指令
pub fn run_command_path(
    ctx: &ActionContext,
    program: &Path,
    args: &[&str],
    use_sudo: bool,
) -> Result<String> {
    run_command(ctx, program.to_str().unwrap_or_default(), args, use_sudo)
}

/// 執行 shell 指令
pub fn run_shell(ctx: &ActionContext, command: &str, use_sudo: bool) -> Result<String> {
    if use_sudo && !ctx.sudo_available {
        return Err(OperationError::Command {
            command: "sudo".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_SUDO_REQUIRED).to_string(),
        });
    }

    if use_sudo {
        run_command(ctx, "sudo", &["bash", "-c", command], false)
    } else {
        run_command(ctx, "bash", &["-c", command], false)
    }
}

/// 檢查指令是否可用
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

// ============================================================================
// 檔案下載
// ============================================================================

/// 下載檔案到指定路徑
pub fn download_file(ctx: &ActionContext, url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|err| OperationError::Io {
            path: parent.display().to_string(),
            source: err,
        })?;
    }

    run_command(
        ctx,
        "curl",
        &["-fL", "-o", dest.to_str().unwrap_or_default(), url],
        false,
    )?;
    Ok(())
}

/// 取得 URL 內容
pub fn fetch_text(ctx: &ActionContext, url: &str, extra_args: &[&str]) -> Result<String> {
    let mut args = vec!["-sSfL"];
    args.extend_from_slice(extra_args);
    args.push(url);
    run_command(ctx, "curl", &args, false)
}

// ============================================================================
// 檔案系統操作
// ============================================================================

/// 建立暫存目錄
pub fn create_temp_dir(ctx: &ActionContext, prefix: &str) -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| OperationError::Command {
            command: "time".to_string(),
            message: err.to_string(),
        })?
        .as_millis();
    let dir = ctx
        .temp_dir
        .join(format!("ops-tools-{}-{}", prefix, timestamp));
    fs::create_dir_all(&dir).map_err(|err| OperationError::Io {
        path: dir.display().to_string(),
        source: err,
    })?;
    Ok(dir)
}

/// 安裝執行檔到系統
pub fn install_binary(ctx: &ActionContext, source: &Path, name: &str) -> Result<PathBuf> {
    let system_dir = Path::new("/usr/local/bin");
    if ctx.sudo_available {
        run_command(
            ctx,
            "install",
            &[
                "-m",
                "0755",
                source.to_str().unwrap_or_default(),
                system_dir.join(name).to_str().unwrap_or_default(),
            ],
            true,
        )?;
        return Ok(system_dir.join(name));
    }

    let local_dir = ctx.home_dir.join(".local/bin");
    fs::create_dir_all(&local_dir).map_err(|err| OperationError::Io {
        path: local_dir.display().to_string(),
        source: err,
    })?;
    let target = local_dir.join(name);
    fs::copy(source, &target).map_err(|err| OperationError::Io {
        path: target.display().to_string(),
        source: err,
    })?;
    set_executable(&target)?;
    Ok(target)
}

/// 移除執行檔
pub fn remove_binary(ctx: &ActionContext, name: &str) -> Result<()> {
    if let Some(path) = is_command_available(name) {
        remove_file(ctx, &path)?;
    }
    Ok(())
}

/// 移除 home 目錄下的執行檔
pub fn remove_home_binary(ctx: &ActionContext, name: &str) -> Result<()> {
    let local_bin = ctx.home_dir.join(".local/bin").join(name);
    if local_bin.exists() {
        fs::remove_file(&local_bin).map_err(|err| OperationError::Io {
            path: local_bin.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

/// 移除檔案
pub fn remove_file(ctx: &ActionContext, path: &Path) -> Result<()> {
    if path.exists() {
        if path.starts_with("/usr/local") && ctx.sudo_available {
            run_command(ctx, "rm", &["-f", path.to_str().unwrap_or_default()], true)?;
        } else {
            fs::remove_file(path).map_err(|err| OperationError::Io {
                path: path.display().to_string(),
                source: err,
            })?;
        }
    }
    Ok(())
}

/// 設定檔案為可執行
pub fn set_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|err| OperationError::Io {
                path: path.display().to_string(),
                source: err,
            })?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).map_err(|err| OperationError::Io {
            path: path.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

/// 確保 profile 檔案包含指定行
pub fn ensure_profile_line(ctx: &ActionContext, line: &str) -> Result<()> {
    let profile = ctx.home_dir.join(".profile");
    let mut needs_write = true;
    if let Ok(existing) = fs::read_to_string(&profile) {
        if existing.contains(line) {
            needs_write = false;
        }
    }

    if needs_write {
        let mut content = fs::read_to_string(&profile).unwrap_or_default();
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(line);
        content.push('\n');
        fs::write(&profile, content).map_err(|err| OperationError::Io {
            path: profile.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

/// 寫入設定檔（含備份）
pub fn write_config_with_backup(path: &Path, content: &str) -> Result<()> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return Ok(());
        }
        let backup = backup_path(path);
        fs::copy(path, &backup).map_err(|err| OperationError::Io {
            path: backup.display().to_string(),
            source: err,
        })?;
    }

    fs::write(path, content).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;
    Ok(())
}

/// 產生備份檔案路徑
fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "config".to_string());
    path.with_file_name(format!("{}.bak", name))
}

/// 解壓縮 tar.gz 檔案
pub fn extract_tar(ctx: &ActionContext, archive: &Path, target: &Path) -> Result<()> {
    run_command(
        ctx,
        "tar",
        &[
            "-xzf",
            archive.to_str().unwrap_or_default(),
            "-C",
            target.to_str().unwrap_or_default(),
        ],
        false,
    )?;
    Ok(())
}

/// 建立符號連結
pub fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    if link.exists() {
        let _ = fs::remove_file(link);
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).map_err(|err| OperationError::Io {
            path: link.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

/// 驗證檔案 checksum
pub fn verify_checksum(ctx: &ActionContext, path: &Path, checksum: &str) -> Result<()> {
    if is_command_available("sha256sum").is_some() {
        let command = format!(
            "echo \"{}  {}\" | sha256sum --check",
            checksum,
            path.to_str().unwrap_or_default()
        );
        run_shell(ctx, &command, false)?;
        return Ok(());
    }

    if is_command_available("shasum").is_some() {
        let command = format!(
            "echo \"{}  {}\" | shasum -a 256 -c -",
            checksum,
            path.to_str().unwrap_or_default()
        );
        run_shell(ctx, &command, false)?;
        return Ok(());
    }

    Ok(())
}

/// 在目錄中尋找執行檔
pub fn find_binary(dir: &Path, name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_binary(&path, name) {
                return Some(found);
            }
        } else if path
            .file_name()
            .and_then(|f| f.to_str())
            .map(|f| f == name)
            .unwrap_or(false)
        {
            return Some(path);
        }
    }
    None
}

// ============================================================================
// 套件管理器操作
// ============================================================================

/// 確保 apt 已更新
pub fn ensure_apt_updated(ctx: &mut ActionContext) -> Result<()> {
    if ctx.apt_updated {
        return Ok(());
    }
    run_command(ctx, "apt-get", &["update"], true)?;
    ctx.apt_updated = true;
    Ok(())
}

/// 確保 pacman 已同步
pub fn ensure_pacman_sync(ctx: &mut ActionContext) -> Result<()> {
    if ctx.pacman_synced {
        return Ok(());
    }
    run_command(ctx, "pacman", &["-Sy", "--noconfirm"], true)?;
    ctx.pacman_synced = true;
    Ok(())
}

/// 確保 HashiCorp repo 已設定
pub fn ensure_hashicorp_repo(ctx: &mut ActionContext) -> Result<()> {
    if ctx.hashicorp_repo_ready {
        return Ok(());
    }

    let manager = require_package_manager(ctx)?;
    match manager {
        PackageManager::Apt => {
            ensure_apt_updated(ctx)?;
            run_command(
                ctx,
                "apt-get",
                &["install", "-y", "gnupg", "software-properties-common"],
                true,
            )?;
            let codename = detect_apt_codename(ctx)?;
            let gpg_cmd = "curl -fsSL https://apt.releases.hashicorp.com/gpg | gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg";
            run_shell(ctx, gpg_cmd, true)?;
            let repo_line = format!(
                "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com {codename} main"
            );
            let repo_cmd =
                format!("echo \"{repo_line}\" | tee /etc/apt/sources.list.d/hashicorp.list");
            run_shell(ctx, &repo_cmd, true)?;
            ensure_apt_updated(ctx)?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["install", "-y", "dnf-plugins-core"], true)?;
            run_command(
                ctx,
                "dnf",
                &[
                    "config-manager",
                    "--add-repo",
                    "https://rpm.releases.hashicorp.com/fedora/hashicorp.repo",
                ],
                true,
            )?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["install", "-y", "yum-utils"], true)?;
            run_command(
                ctx,
                "yum-config-manager",
                &[
                    "--add-repo",
                    "https://rpm.releases.hashicorp.com/RHEL/hashicorp.repo",
                ],
                true,
            )?;
        }
        _ => {}
    }

    ctx.hashicorp_repo_ready = true;
    Ok(())
}

/// 取得套件管理器（必須存在）
pub fn require_package_manager(ctx: &ActionContext) -> Result<PackageManager> {
    ctx.package_manager.ok_or_else(|| OperationError::Command {
        command: "package-manager".to_string(),
        message: crate::tr!(keys::PACKAGE_MANAGER_MISSING_PM, os = ctx.os.label()),
    })
}

/// 偵測 apt codename
pub fn detect_apt_codename(ctx: &ActionContext) -> Result<String> {
    if let Some(value) = read_os_release_value("VERSION_CODENAME")
        .or_else(|| read_os_release_value("UBUNTU_CODENAME"))
    {
        return Ok(value);
    }

    if is_command_available("lsb_release").is_some() {
        let output = run_command(ctx, "lsb_release", &["-cs"], false)?;
        let code = output.trim();
        if !code.is_empty() {
            return Ok(code.to_string());
        }
    }

    Err(OperationError::Command {
        command: "lsb_release".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_CODENAME_MISSING).to_string(),
    })
}

/// 讀取 /etc/os-release 的值
fn read_os_release_value(key: &str) -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        let mut parts = line.splitn(2, '=');
        let k = parts.next()?.trim();
        let v = parts.next()?.trim().trim_matches('"');
        if k == key {
            return Some(v.to_string());
        }
    }
    None
}

/// 使用系統套件管理器安裝
pub fn install_with_manager(ctx: &mut ActionContext, package: &str) -> Result<()> {
    let manager = require_package_manager(ctx)?;
    match manager {
        PackageManager::Brew => {
            run_command(ctx, "brew", &["install", package], false)?;
        }
        PackageManager::Apt => {
            ensure_apt_updated(ctx)?;
            run_command(ctx, "apt-get", &["install", "-y", package], true)?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["install", "-y", package], true)?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["install", "-y", package], true)?;
        }
        PackageManager::Pacman => {
            ensure_pacman_sync(ctx)?;
            run_command(ctx, "pacman", &["-S", "--noconfirm", package], true)?;
        }
        PackageManager::Zypper => {
            run_command(ctx, "zypper", &["install", "-y", package], true)?;
        }
        PackageManager::Apk => {
            run_command(ctx, "apk", &["add", package], true)?;
        }
    }
    Ok(())
}

/// 使用系統套件管理器更新
pub fn update_with_manager(ctx: &mut ActionContext, package: &str) -> Result<()> {
    let manager = require_package_manager(ctx)?;
    match manager {
        PackageManager::Brew => {
            run_command(ctx, "brew", &["upgrade", package], false)?;
        }
        PackageManager::Apt => {
            ensure_apt_updated(ctx)?;
            run_command(
                ctx,
                "apt-get",
                &["install", "--only-upgrade", "-y", package],
                true,
            )?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["upgrade", "-y", package], true)?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["update", "-y", package], true)?;
        }
        PackageManager::Pacman => {
            ensure_pacman_sync(ctx)?;
            run_command(ctx, "pacman", &["-S", "--noconfirm", package], true)?;
        }
        PackageManager::Zypper => {
            run_command(ctx, "zypper", &["update", "-y", package], true)?;
        }
        PackageManager::Apk => {
            run_command(ctx, "apk", &["upgrade", package], true)?;
        }
    }
    Ok(())
}

/// 使用系統套件管理器移除
pub fn remove_with_manager(ctx: &mut ActionContext, package: &str) -> Result<()> {
    let manager = require_package_manager(ctx)?;
    match manager {
        PackageManager::Brew => {
            run_command(ctx, "brew", &["uninstall", package], false)?;
        }
        PackageManager::Apt => {
            run_command(ctx, "apt-get", &["remove", "-y", package], true)?;
        }
        PackageManager::Dnf => {
            run_command(ctx, "dnf", &["remove", "-y", package], true)?;
        }
        PackageManager::Yum => {
            run_command(ctx, "yum", &["remove", "-y", package], true)?;
        }
        PackageManager::Pacman => {
            run_command(ctx, "pacman", &["-R", "--noconfirm", package], true)?;
        }
        PackageManager::Zypper => {
            run_command(ctx, "zypper", &["remove", "-y", package], true)?;
        }
        PackageManager::Apk => {
            run_command(ctx, "apk", &["del", package], true)?;
        }
    }
    Ok(())
}

// ============================================================================
// GitHub / Go 下載工具
// ============================================================================

/// 取得 Go 架構
pub fn go_arch() -> Result<&'static str> {
    match env::consts::ARCH {
        "x86_64" => Ok("amd64"),
        "aarch64" => Ok("arm64"),
        _ => Err(OperationError::Command {
            command: "arch".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_ARCH_UNSUPPORTED).to_string(),
        }),
    }
}

#[derive(Deserialize)]
struct GoRelease {
    stable: bool,
    files: Vec<GoFile>,
}

#[derive(Deserialize)]
struct GoFile {
    filename: String,
    os: String,
    arch: String,
    kind: String,
}

/// Go 下載資訊
pub struct GoDownload {
    pub filename: String,
    pub url: String,
}

/// 取得最新 Go 下載連結
pub fn latest_go_download(ctx: &ActionContext) -> Result<GoDownload> {
    let json = fetch_text(ctx, "https://go.dev/dl/?mode=json", &[])?;
    let releases: Vec<GoRelease> =
        serde_json::from_str(&json).map_err(|err| OperationError::Command {
            command: "go release".to_string(),
            message: err.to_string(),
        })?;
    let release =
        releases
            .into_iter()
            .find(|rel| rel.stable)
            .ok_or_else(|| OperationError::Command {
                command: "go release".to_string(),
                message: i18n::t(keys::PACKAGE_MANAGER_GO_VERSION_MISSING).to_string(),
            })?;

    let arch = go_arch()?;
    let desired_kind = match ctx.os {
        SupportedOs::Linux => "archive",
        SupportedOs::Macos => "installer",
    };
    let file = release
        .files
        .into_iter()
        .find(|file| file.os == ctx.os.go_os() && file.arch == arch && file.kind == desired_kind)
        .ok_or_else(|| OperationError::Command {
            command: "go download".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_GO_FILE_MISSING).to_string(),
        })?;

    Ok(GoDownload {
        filename: file.filename.clone(),
        url: format!("https://go.dev/dl/{}", file.filename),
    })
}

/// GitHub Release Asset
pub struct GithubAsset {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

/// 取得最新 GitHub release asset
pub fn latest_github_asset(
    repo: &str,
    ctx: &ActionContext,
    prefix: &str,
    suffix: &str,
) -> Result<GithubAsset> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let json = fetch_text(ctx, &url, &["-H", "User-Agent: ops-tools"])?;
    let release: GithubRelease =
        serde_json::from_str(&json).map_err(|err| OperationError::Command {
            command: "github release".to_string(),
            message: err.to_string(),
        })?;

    let os_token = match ctx.os {
        SupportedOs::Linux => "Linux",
        SupportedOs::Macos => "Darwin",
    };
    let arch_token = go_arch()?;

    let asset = release
        .assets
        .into_iter()
        .find(|asset| {
            asset.name.contains(prefix)
                && asset.name.contains(os_token)
                && asset.name.contains(arch_token)
                && asset.name.ends_with(suffix)
        })
        .ok_or_else(|| OperationError::Command {
            command: "github release".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_RELEASE_ASSET_MISSING).to_string(),
        })?;

    Ok(GithubAsset {
        name: asset.name,
        url: asset.browser_download_url,
    })
}

// ============================================================================
// 路徑工具
// ============================================================================

/// 取得 NVM 目錄
pub fn nvm_dir(ctx: &ActionContext) -> PathBuf {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("nvm")
    } else {
        ctx.home_dir.join(".nvm")
    }
}

/// 取得 rustup 路徑
pub fn rustup_path(ctx: &ActionContext) -> Option<PathBuf> {
    if let Some(path) = is_command_available("rustup") {
        return Some(path);
    }
    let fallback = ctx.home_dir.join(".cargo/bin/rustup");
    if fallback.is_file() {
        Some(fallback)
    } else {
        None
    }
}

/// 取得 uv 路徑
pub fn uv_path(ctx: &ActionContext) -> Option<PathBuf> {
    if let Some(path) = is_command_available("uv") {
        return Some(path);
    }
    let candidates = [
        ctx.home_dir.join(".local/bin/uv"),
        ctx.home_dir.join(".cargo/bin/uv"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}
