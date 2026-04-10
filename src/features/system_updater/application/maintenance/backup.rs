use crate::features::system_updater::application::maintenance::MaintenanceContext;
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::StepOutcome;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};
use std::path::Path;

pub fn create_snapshot<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let snapshot_dir = context.reporter.artifact_dir().join("backup");
    context.host.create_dir_all(&snapshot_dir)?;

    write_capture(
        context,
        CommandSpec::new("uname", ["-a"]),
        &snapshot_dir.join("uname.txt"),
    )?;
    if context.platform.is_linux() && context.host.command_path("dpkg").is_some() {
        write_capture(
            context,
            CommandSpec::new("dpkg", ["--get-selections"]),
            &snapshot_dir.join("dpkg-selections.txt"),
        )?;
    }
    if context.platform.is_linux() && context.host.command_path("apt-mark").is_some() {
        write_capture(
            context,
            CommandSpec::new("apt-mark", ["showmanual"]),
            &snapshot_dir.join("apt-manual.txt"),
        )?;
        write_capture(
            context,
            CommandSpec::new("apt-mark", ["showhold"]),
            &snapshot_dir.join("apt-hold.txt"),
        )?;
    }

    if context.platform.is_linux() && context.host.command_path("systemctl").is_some() {
        write_capture(
            context,
            CommandSpec::new("systemctl", ["--failed", "--no-legend"]),
            &snapshot_dir.join("systemd-failed.txt"),
        )?;
    }

    let os_release = Path::new("/etc/os-release");
    if context.host.exists(os_release) {
        let content = context.host.read_to_string(os_release)?;
        context
            .host
            .write_string(&snapshot_dir.join("os-release.txt"), &content)?;
    }

    if context.platform.is_linux() && context.host.exists(Path::new("/etc/apt/sources.list")) {
        let content = context
            .host
            .read_to_string(Path::new("/etc/apt/sources.list"))?;
        context
            .host
            .write_string(&snapshot_dir.join("apt-sources.list"), &content)?;
    }

    if context.platform.is_macos() && context.host.command_path("sw_vers").is_some() {
        write_capture(
            context,
            CommandSpec::new("sw_vers", std::iter::empty::<&str>()),
            &snapshot_dir.join("sw_vers.txt"),
        )?;
    }
    if context.platform.is_macos() && context.host.command_path("brew").is_some() {
        write_capture(
            context,
            CommandSpec::new("brew", ["list", "--versions"]),
            &snapshot_dir.join("brew-list.txt"),
        )?;
        write_capture(
            context,
            CommandSpec::new("brew", ["outdated", "--verbose"]),
            &snapshot_dir.join("brew-outdated.txt"),
        )?;
    }
    if context.platform.is_macos() && context.host.command_path("softwareupdate").is_some() {
        write_capture(
            context,
            CommandSpec::new("softwareupdate", ["--list"]),
            &snapshot_dir.join("softwareupdate.txt"),
        )?;
    }

    if context.host.command_path("nvidia-smi").is_some() {
        write_capture(
            context,
            CommandSpec::new("nvidia-smi", ["-L"]),
            &snapshot_dir.join("nvidia-smi.txt"),
        )?;
    }

    context
        .reporter
        .note(&format!("snapshot written to {}", snapshot_dir.display()))?;
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run(format!(
            "snapshot would be written to {}",
            snapshot_dir.display()
        ))
    } else {
        StepOutcome::ok()
    })
}

fn write_capture<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    command: CommandSpec,
    target: &Path,
) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let content = context.executor.capture(&command)?;
    context.host.write_string(target, &content)?;
    Ok(())
}
