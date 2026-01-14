//! 套件安裝器
//!
//! 各套件的安裝、更新、移除實作

use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use std::fs;

use super::config_content::{
    FFMPEG_BUILD_SCRIPT, NVM_INSTALL_SCRIPT, PNPM_INSTALL_SCRIPT, RUSTUP_INSTALL_SCRIPT,
    TMUX_CONF_CONTENT, UV_INSTALL_SCRIPT, VIMRC_CONTENT,
};
use super::shell::{
    create_symlink, create_temp_dir, download_file, ensure_hashicorp_repo, ensure_profile_line,
    extract_tar, fetch_text, find_binary, go_arch, install_binary, install_with_manager,
    is_command_available, latest_github_asset, latest_go_download, nvm_dir, remove_binary,
    remove_file, remove_home_binary, remove_with_manager, run_command, run_command_path, run_shell,
    rustup_path, update_with_manager, uv_path, verify_checksum, write_config_with_backup,
};
use super::types::{ActionContext, PackageId, SupportedOs};

// ============================================================================
// 公開 API
// ============================================================================

/// 檢查套件是否已安裝
pub fn is_installed(package: PackageId, ctx: &ActionContext) -> bool {
    match package {
        PackageId::Nvm => nvm_dir(ctx).join("nvm.sh").is_file(),
        PackageId::Pnpm => is_command_available("pnpm").is_some(),
        PackageId::Rust => is_command_available("rustup").is_some(),
        PackageId::Go => is_command_available("go").is_some(),
        PackageId::Terraform => is_command_available("terraform").is_some(),
        PackageId::Kubectl => is_command_available("kubectl").is_some(),
        PackageId::Kubectx => is_command_available("kubectx").is_some(),
        PackageId::K9s => is_command_available("k9s").is_some(),
        PackageId::Git => is_command_available("git").is_some(),
        PackageId::Uv => is_command_available("uv").is_some(),
        PackageId::Tmux => is_command_available("tmux").is_some(),
        PackageId::Vim => is_command_available("vim").is_some(),
        PackageId::Ffmpeg => is_command_available("ffmpeg").is_some(),
    }
}

/// 安裝套件
pub fn install_package(package: PackageId, ctx: &mut ActionContext) -> Result<()> {
    match package {
        PackageId::Nvm => install_nvm(ctx),
        PackageId::Pnpm => install_pnpm(ctx),
        PackageId::Rust => install_rust(ctx),
        PackageId::Go => install_go(ctx),
        PackageId::Terraform => install_terraform(ctx),
        PackageId::Kubectl => install_kubectl(ctx),
        PackageId::Kubectx => install_kubectx(ctx),
        PackageId::K9s => install_k9s(ctx),
        PackageId::Git => install_git(ctx),
        PackageId::Uv => install_uv(ctx),
        PackageId::Tmux => install_tmux(ctx),
        PackageId::Vim => install_vim(ctx),
        PackageId::Ffmpeg => install_ffmpeg(ctx),
    }
}

/// 更新套件
pub fn update_package(package: PackageId, ctx: &mut ActionContext) -> Result<()> {
    match package {
        PackageId::Nvm => update_nvm(ctx),
        PackageId::Pnpm => update_pnpm(ctx),
        PackageId::Rust => update_rust(ctx),
        PackageId::Go => install_go(ctx),
        PackageId::Terraform => update_terraform(ctx),
        PackageId::Kubectl => install_kubectl(ctx),
        PackageId::Kubectx => update_kubectx(ctx),
        PackageId::K9s => update_k9s(ctx),
        PackageId::Git => update_git(ctx),
        PackageId::Uv => update_uv(ctx),
        PackageId::Tmux => update_tmux(ctx),
        PackageId::Vim => update_vim(ctx),
        PackageId::Ffmpeg => update_ffmpeg(ctx),
    }
}

/// 移除套件
pub fn remove_package(package: PackageId, ctx: &mut ActionContext) -> Result<()> {
    match package {
        PackageId::Nvm => remove_nvm(ctx),
        PackageId::Pnpm => remove_pnpm(ctx),
        PackageId::Rust => remove_rust(ctx),
        PackageId::Go => remove_go(ctx),
        PackageId::Terraform => remove_terraform(ctx),
        PackageId::Kubectl => remove_binary(ctx, "kubectl"),
        PackageId::Kubectx => remove_kubectx(ctx),
        PackageId::K9s => remove_k9s(ctx),
        PackageId::Git => remove_git(ctx),
        PackageId::Uv => remove_uv(ctx),
        PackageId::Tmux => remove_tmux(ctx),
        PackageId::Vim => remove_vim(ctx),
        PackageId::Ffmpeg => remove_ffmpeg(ctx),
    }
}

// ============================================================================
// NVM
// ============================================================================

fn install_nvm(ctx: &mut ActionContext) -> Result<()> {
    run_shell(ctx, &format!("curl -o- {NVM_INSTALL_SCRIPT} | bash"), false)?;
    let nvm_dir = nvm_dir(ctx);
    let command = format!(
        "export NVM_DIR=\"{dir}\"; [ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\"; nvm install node; nvm alias default node",
        dir = nvm_dir.display()
    );
    run_shell(ctx, &command, false)?;
    Ok(())
}

fn update_nvm(ctx: &mut ActionContext) -> Result<()> {
    install_nvm(ctx)
}

fn remove_nvm(ctx: &mut ActionContext) -> Result<()> {
    let dir = nvm_dir(ctx);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|err| OperationError::Io {
            path: dir.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

// ============================================================================
// PNPM
// ============================================================================

fn install_pnpm(ctx: &mut ActionContext) -> Result<()> {
    run_shell(
        ctx,
        &format!("curl -fsSL {PNPM_INSTALL_SCRIPT} | sh -"),
        false,
    )?;
    Ok(())
}

fn update_pnpm(ctx: &mut ActionContext) -> Result<()> {
    install_pnpm(ctx)
}

fn remove_pnpm(ctx: &mut ActionContext) -> Result<()> {
    let pnpm_home = ctx.home_dir.join(".local/share/pnpm");
    let pnpm_global = ctx.home_dir.join(".local/share/pnpm-global");
    if pnpm_home.exists() {
        fs::remove_dir_all(&pnpm_home).map_err(|err| OperationError::Io {
            path: pnpm_home.display().to_string(),
            source: err,
        })?;
    }
    if pnpm_global.exists() {
        fs::remove_dir_all(&pnpm_global).map_err(|err| OperationError::Io {
            path: pnpm_global.display().to_string(),
            source: err,
        })?;
    }
    remove_home_binary(ctx, "pnpm")?;
    remove_home_binary(ctx, "pnpx")?;
    Ok(())
}

// ============================================================================
// Rust
// ============================================================================

fn install_rust(ctx: &mut ActionContext) -> Result<()> {
    run_shell(
        ctx,
        &format!("curl --proto '=https' --tlsv1.2 -sSf {RUSTUP_INSTALL_SCRIPT} | sh -s -- -y"),
        false,
    )?;
    Ok(())
}

fn update_rust(ctx: &mut ActionContext) -> Result<()> {
    let rustup = rustup_path(ctx).ok_or_else(|| OperationError::Command {
        command: "rustup".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_RUSTUP_MISSING).to_string(),
    })?;
    run_command_path(ctx, &rustup, &["self", "update"], false)?;
    run_command_path(ctx, &rustup, &["update"], false)?;
    Ok(())
}

fn remove_rust(ctx: &mut ActionContext) -> Result<()> {
    if let Some(rustup) = rustup_path(ctx) {
        run_command_path(ctx, &rustup, &["self", "uninstall", "-y"], false)?;
    }
    let rustup_dir = ctx.home_dir.join(".rustup");
    let cargo_dir = ctx.home_dir.join(".cargo");
    if rustup_dir.exists() {
        let _ = fs::remove_dir_all(&rustup_dir);
    }
    if cargo_dir.exists() {
        let _ = fs::remove_dir_all(&cargo_dir);
    }
    Ok(())
}

// ============================================================================
// Go
// ============================================================================

fn install_go(ctx: &mut ActionContext) -> Result<()> {
    let download = latest_go_download(ctx)?;
    let temp_dir = create_temp_dir(ctx, "go-download")?;
    let archive_path = temp_dir.join(&download.filename);
    download_file(ctx, &download.url, &archive_path)?;

    match ctx.os {
        SupportedOs::Linux => {
            run_command(ctx, "rm", &["-rf", "/usr/local/go"], ctx.sudo_available)?;
            run_command(
                ctx,
                "tar",
                &[
                    "-C",
                    "/usr/local",
                    "-xzf",
                    archive_path.to_str().unwrap_or_default(),
                ],
                ctx.sudo_available,
            )?;
            ensure_profile_line(ctx, "export PATH=$PATH:/usr/local/go/bin")?;
        }
        SupportedOs::Macos => {
            run_command(
                ctx,
                "installer",
                &[
                    "-pkg",
                    archive_path.to_str().unwrap_or_default(),
                    "-target",
                    "/",
                ],
                ctx.sudo_available,
            )?;
        }
    }
    Ok(())
}

fn remove_go(ctx: &mut ActionContext) -> Result<()> {
    run_command(ctx, "rm", &["-rf", "/usr/local/go"], ctx.sudo_available)?;
    Ok(())
}

// ============================================================================
// Terraform
// ============================================================================

fn install_terraform(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "terraform"),
        SupportedOs::Linux => install_terraform_linux(ctx),
    }
}

fn update_terraform(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => update_with_manager(ctx, "terraform"),
        SupportedOs::Linux => update_terraform_linux(ctx),
    }
}

fn remove_terraform(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "terraform")
}

fn install_terraform_linux(ctx: &mut ActionContext) -> Result<()> {
    ensure_hashicorp_repo(ctx)?;
    install_with_manager(ctx, "terraform")
}

fn update_terraform_linux(ctx: &mut ActionContext) -> Result<()> {
    ensure_hashicorp_repo(ctx)?;
    update_with_manager(ctx, "terraform")
}

// ============================================================================
// Kubectl
// ============================================================================

fn install_kubectl(ctx: &mut ActionContext) -> Result<()> {
    let version = fetch_text(
        ctx,
        "https://dl.k8s.io/release/stable.txt",
        &["-H", "User-Agent: ops-tools"],
    )?;
    let version = version.trim();
    let arch = go_arch()?;
    let os = ctx.os.kubectl_os();
    let url = format!(
        "https://dl.k8s.io/release/{}/bin/{}/{}/kubectl",
        version, os, arch
    );
    let checksum_url = format!("{}.sha256", url);

    let temp_dir = create_temp_dir(ctx, "kubectl")?;
    let bin_path = temp_dir.join("kubectl");
    download_file(ctx, &url, &bin_path)?;

    let checksum = fetch_text(ctx, &checksum_url, &["-H", "User-Agent: ops-tools"])?;
    verify_checksum(ctx, &bin_path, checksum.trim())?;

    install_binary(ctx, &bin_path, "kubectl")?;
    Ok(())
}

// ============================================================================
// Kubectx
// ============================================================================

fn install_kubectx(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "kubectx"),
        SupportedOs::Linux => install_kubectx_linux(ctx),
    }
}

fn update_kubectx(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => update_with_manager(ctx, "kubectx"),
        SupportedOs::Linux => update_kubectx_linux(ctx),
    }
}

fn remove_kubectx(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => remove_with_manager(ctx, "kubectx"),
        SupportedOs::Linux => remove_kubectx_linux(ctx),
    }
}

fn install_kubectx_linux(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("git").is_none() {
        return Err(OperationError::Command {
            command: "git".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_GIT_REQUIRED).to_string(),
        });
    }

    let repo_dir = ctx.home_dir.join(".kubectx");
    if repo_dir.exists() {
        run_command(
            ctx,
            "git",
            &[
                "-C",
                repo_dir.to_str().unwrap_or_default(),
                "pull",
                "--ff-only",
            ],
            false,
        )?;
    } else {
        run_command(
            ctx,
            "git",
            &[
                "clone",
                "https://github.com/ahmetb/kubectx",
                repo_dir.to_str().unwrap_or_default(),
            ],
            false,
        )?;
    }

    let bin_dir = ctx.home_dir.join(".local/bin");
    fs::create_dir_all(&bin_dir).map_err(|err| OperationError::Io {
        path: bin_dir.display().to_string(),
        source: err,
    })?;
    let link_path = bin_dir.join("kubectx");
    let target = repo_dir.join("kubectx");
    create_symlink(&target, &link_path)?;
    Ok(())
}

fn update_kubectx_linux(ctx: &mut ActionContext) -> Result<()> {
    install_kubectx_linux(ctx)
}

fn remove_kubectx_linux(ctx: &mut ActionContext) -> Result<()> {
    let repo_dir = ctx.home_dir.join(".kubectx");
    if repo_dir.exists() {
        let _ = fs::remove_dir_all(&repo_dir);
    }
    remove_home_binary(ctx, "kubectx")?;
    Ok(())
}

// ============================================================================
// K9s
// ============================================================================

fn install_k9s(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "k9s"),
        SupportedOs::Linux => install_k9s_linux(ctx),
    }
}

fn update_k9s(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => update_with_manager(ctx, "k9s"),
        SupportedOs::Linux => install_k9s_linux(ctx),
    }
}

fn remove_k9s(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => remove_with_manager(ctx, "k9s"),
        SupportedOs::Linux => remove_binary(ctx, "k9s"),
    }
}

fn install_k9s_linux(ctx: &mut ActionContext) -> Result<()> {
    let asset = latest_github_asset("derailed/k9s", ctx, "k9s_", ".tar.gz")?;
    let temp_dir = create_temp_dir(ctx, "k9s")?;
    let archive = temp_dir.join(&asset.name);
    download_file(ctx, &asset.url, &archive)?;
    extract_tar(ctx, &archive, &temp_dir)?;
    let binary = find_binary(&temp_dir, "k9s").ok_or_else(|| OperationError::Command {
        command: "k9s".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_BINARY_NOT_FOUND).to_string(),
    })?;
    install_binary(ctx, &binary, "k9s")?;
    Ok(())
}

// ============================================================================
// Git
// ============================================================================

fn install_git(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "git"),
        SupportedOs::Linux => install_with_manager(ctx, "git"),
    }
}

fn update_git(ctx: &mut ActionContext) -> Result<()> {
    update_with_manager(ctx, "git")
}

fn remove_git(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "git")
}

// ============================================================================
// UV (Python)
// ============================================================================

fn install_uv(ctx: &mut ActionContext) -> Result<()> {
    run_shell(ctx, &format!("curl -LsSf {UV_INSTALL_SCRIPT} | sh"), false)?;
    install_uv_python(ctx)?;
    Ok(())
}

fn update_uv(ctx: &mut ActionContext) -> Result<()> {
    install_uv(ctx)
}

fn remove_uv(ctx: &mut ActionContext) -> Result<()> {
    if let Some(path) = uv_path(ctx) {
        remove_file(ctx, &path)?;
    }
    let uv_dir = ctx.home_dir.join(".local/share/uv");
    if uv_dir.exists() {
        let _ = fs::remove_dir_all(&uv_dir);
    }
    Ok(())
}

fn install_uv_python(ctx: &mut ActionContext) -> Result<()> {
    let uv = uv_path(ctx).ok_or_else(|| OperationError::Command {
        command: "uv".to_string(),
        message: i18n::t(keys::PACKAGE_MANAGER_UV_MISSING).to_string(),
    })?;
    run_command_path(ctx, &uv, &["python", "install"], false)?;
    Ok(())
}

// ============================================================================
// Tmux
// ============================================================================

fn install_tmux(ctx: &mut ActionContext) -> Result<()> {
    install_with_manager(ctx, "tmux")?;
    setup_tmux_config(ctx)?;
    Ok(())
}

fn update_tmux(ctx: &mut ActionContext) -> Result<()> {
    update_with_manager(ctx, "tmux")?;
    setup_tmux_config(ctx)?;
    Ok(())
}

fn remove_tmux(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "tmux")
}

fn setup_tmux_config(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("git").is_none() {
        return Err(OperationError::Command {
            command: "git".to_string(),
            message: i18n::t(keys::PACKAGE_MANAGER_GIT_REQUIRED).to_string(),
        });
    }

    let plugins_dir = ctx.home_dir.join(".tmux/plugins");
    let tpm_dir = plugins_dir.join("tpm");
    fs::create_dir_all(&plugins_dir).map_err(|err| OperationError::Io {
        path: plugins_dir.display().to_string(),
        source: err,
    })?;

    if tpm_dir.exists() {
        run_command(
            ctx,
            "git",
            &[
                "-C",
                tpm_dir.to_str().unwrap_or_default(),
                "pull",
                "--ff-only",
            ],
            false,
        )?;
    } else {
        run_command(
            ctx,
            "git",
            &[
                "clone",
                "https://github.com/tmux-plugins/tpm",
                tpm_dir.to_str().unwrap_or_default(),
            ],
            false,
        )?;
    }

    let vim_plug = ctx.home_dir.join(".vim/autoload/plug.vim");
    download_file(
        ctx,
        "https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim",
        &vim_plug,
    )?;

    write_config_with_backup(&ctx.home_dir.join(".tmux.conf"), TMUX_CONF_CONTENT)?;
    Ok(())
}

// ============================================================================
// Vim
// ============================================================================

fn install_vim(ctx: &mut ActionContext) -> Result<()> {
    install_with_manager(ctx, "vim")?;
    setup_vim_config(ctx)?;
    Ok(())
}

fn update_vim(ctx: &mut ActionContext) -> Result<()> {
    update_with_manager(ctx, "vim")?;
    setup_vim_config(ctx)?;
    Ok(())
}

fn remove_vim(ctx: &mut ActionContext) -> Result<()> {
    remove_with_manager(ctx, "vim")
}

fn setup_vim_config(ctx: &mut ActionContext) -> Result<()> {
    let vim_plug = ctx.home_dir.join(".vim/autoload/plug.vim");
    download_file(
        ctx,
        "https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim",
        &vim_plug,
    )?;

    let colors_dir = ctx.home_dir.join(".vim/colors");
    fs::create_dir_all(&colors_dir).map_err(|err| OperationError::Io {
        path: colors_dir.display().to_string(),
        source: err,
    })?;
    download_file(
        ctx,
        "https://raw.githubusercontent.com/tomasr/molokai/master/colors/molokai.vim",
        &colors_dir.join("molokai.vim"),
    )?;

    write_config_with_backup(&ctx.home_dir.join(".vimrc"), VIMRC_CONTENT)?;
    Ok(())
}

// ============================================================================
// FFmpeg
// ============================================================================

fn install_ffmpeg(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => install_with_manager(ctx, "ffmpeg"),
        SupportedOs::Linux => run_ffmpeg_build(ctx),
    }
}

fn update_ffmpeg(ctx: &mut ActionContext) -> Result<()> {
    install_ffmpeg(ctx)
}

fn remove_ffmpeg(ctx: &mut ActionContext) -> Result<()> {
    match ctx.os {
        SupportedOs::Macos => remove_with_manager(ctx, "ffmpeg"),
        SupportedOs::Linux => {
            let prefix = ctx.home_dir.join(".ffbuild");
            if prefix.exists() {
                let _ = fs::remove_dir_all(&prefix);
            }
            remove_home_binary(ctx, "ffmpeg")?;
            remove_home_binary(ctx, "ffprobe")?;
            Ok(())
        }
    }
}

fn run_ffmpeg_build(ctx: &mut ActionContext) -> Result<()> {
    let temp_dir = create_temp_dir(ctx, "ffmpeg-build")?;
    let script_path = temp_dir.join("build_ffmpeg.sh");
    fs::write(&script_path, FFMPEG_BUILD_SCRIPT).map_err(|err| OperationError::Io {
        path: script_path.display().to_string(),
        source: err,
    })?;
    run_command(
        ctx,
        "bash",
        &[script_path.to_str().unwrap_or_default()],
        false,
    )?;
    Ok(())
}
