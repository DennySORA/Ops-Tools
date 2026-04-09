use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::config::CleanupStrategy;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::{StepOutcome, StepStatus};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};
use std::path::Path;

pub fn cleanup_local_artifacts<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let mut warnings = WarningCollector::new();
    let mut any_action = false;

    if let Some(days) = context.config.cleanup.journal_max_days {
        any_action = true;
        warnings.capture(
            format!("journalctl --vacuum-time={days}d"),
            context.executor.run(
                &CommandSpec::new("journalctl", [format!("--vacuum-time={days}d")]).with_sudo(),
            ),
        );
    }
    if let Some(size_mb) = context.config.cleanup.journal_max_size_mb {
        any_action = true;
        warnings.capture(
            format!("journalctl --vacuum-size={size_mb}M"),
            context.executor.run(
                &CommandSpec::new("journalctl", [format!("--vacuum-size={size_mb}M")]).with_sudo(),
            ),
        );
    }
    if context.config.cleanup.crash_reports {
        any_action = true;
        warnings.capture(
            "remove /var/crash/*.crash",
            context.executor.run(
                &CommandSpec::new(
                    "bash",
                    [
                        "-lc",
                        "shopt -s nullglob && rm -f /var/crash/*.crash /var/crash/*.upload",
                    ],
                )
                .with_sudo(),
            ),
        );
    }

    if !any_action {
        return Ok(StepOutcome::skipped(
            "journal and crash cleanup disabled in config",
        ));
    }

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("journal and crash cleanup previewed")
    } else if matches!(context.config.cleanup.strategy, CleanupStrategy::Aggressive) {
        warnings.finish_as(StepStatus::Partial)
    } else {
        warnings.finish()
    })
}

pub fn postflight_verify<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let mut warnings = WarningCollector::new();

    if context.executor.is_dry_run() {
        println!("  [dry-run] would run dpkg --audit");
        println!("  [dry-run] would inspect remaining apt upgrades");
        println!("  [dry-run] would inspect held packages");
        println!("  [dry-run] would inspect failed systemd units");
        if context.platform.expects_nvidia_tooling() {
            println!("  [dry-run] would verify nvidia-smi -L");
        }
        if context.host.command_path("docker").is_some() {
            println!("  [dry-run] would verify sudo docker info");
            println!("  [dry-run] would inspect docker ps");
        }
        return Ok(StepOutcome::dry_run("postflight verification previewed"));
    }

    let audit = context
        .executor
        .capture(&CommandSpec::new("dpkg", ["--audit"]))?;
    if audit.trim().is_empty() {
        println!("  dpkg audit clean.");
    } else {
        warnings.warn(format!(
            "dpkg audit reported issues: {}",
            first_non_empty_line(&audit)
        ));
    }

    let upgradable = context
        .executor
        .capture(&CommandSpec::new("apt", ["list", "--upgradable"]))?;
    let remaining = upgradable
        .lines()
        .filter(|line| line.contains("upgradable"))
        .count();
    if remaining == 0 {
        println!("  No remaining APT upgrades detected.");
    } else {
        warnings.warn(format!(
            "{remaining} APT package(s) still upgradable after maintenance"
        ));
    }

    let holds = context
        .executor
        .capture(&CommandSpec::new("apt-mark", ["showhold"]))?;
    if !holds.trim().is_empty() {
        println!("  Held APT packages:");
        for line in holds.lines().filter(|line| !line.trim().is_empty()) {
            println!("    {line}");
        }
    }

    if context.host.command_path("systemctl").is_some() {
        let failed_units = context
            .executor
            .capture(&CommandSpec::new("systemctl", ["--failed", "--no-legend"]))?;
        if !failed_units.trim().is_empty() {
            warnings.warn(format!(
                "failed systemd units detected: {}",
                first_non_empty_line(&failed_units)
            ));
        }
    }

    if context.platform.expects_nvidia_tooling() {
        if context.host.command_path("nvidia-smi").is_some() {
            warnings.capture(
                "nvidia-smi -L",
                context
                    .executor
                    .run(&CommandSpec::new("nvidia-smi", ["-L"]).with_timeout_secs(30)),
            );
        } else {
            warnings.warn(format!(
                "{} expected NVIDIA tooling, but `nvidia-smi` was not found",
                context.platform.label()
            ));
        }
    }

    if context.host.command_path("docker").is_some() {
        warnings.capture(
            "docker info",
            context.executor.run(
                &CommandSpec::new("docker", ["info"])
                    .with_sudo()
                    .with_timeout_secs(60),
            ),
        );
        warnings.capture(
            "docker ps",
            context.executor.run(
                &CommandSpec::new("docker", ["ps"])
                    .with_sudo()
                    .with_timeout_secs(60),
            ),
        );
    }

    Ok(warnings.finish_as(StepStatus::Partial))
}

pub fn check_reboot<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let reboot_marker = Path::new("/var/run/reboot-required");
    if !context.host.exists(reboot_marker) {
        println!("  No reboot required.");
        return Ok(StepOutcome::skipped("reboot-required marker not present"));
    }

    println!("  Reboot required marker detected.");
    if !context.config.runtime.auto_reboot {
        println!("  Auto-reboot disabled in config.");
        return Ok(StepOutcome::skipped("auto reboot disabled by config"));
    }
    if context.platform.expects_nvidia_tooling()
        && context.host.command_path("nvidia-smi").is_none()
    {
        let mut warnings = WarningCollector::new();
        warnings.warn(format!(
            "{} requires `nvidia-smi` for safe workload-aware reboot checks",
            context.platform.label()
        ));
        println!("  Skipping auto-reboot because NVIDIA workload state cannot be determined.");
        return Ok(warnings.finish_as(StepStatus::Blocked));
    }
    if context.executor.is_dry_run() {
        println!("  [dry-run] would evaluate GPU workload state before auto-reboot.");
        if context.platform.expects_nvidia_tooling() {
            println!("  [dry-run] would reboot only if no active GPU workload is detected.");
        } else {
            println!("  [dry-run] no NVIDIA workload gate required on this platform.");
        }
        return Ok(StepOutcome::dry_run(
            "auto reboot decision previewed in dry-run mode",
        ));
    }

    match gpu_workloads_running(context) {
        Ok(true) => {
            let mut warnings = WarningCollector::new();
            warnings.warn("GPU workloads detected -- skipping auto-reboot.");
            println!("  Active GPU processes:");
            warnings.capture(
                "nvidia-smi active GPU process listing",
                context.executor.run(
                    &CommandSpec::new(
                        "nvidia-smi",
                        [
                            "--query-compute-apps=pid,process_name,used_memory",
                            "--format=csv",
                        ],
                    )
                    .with_timeout_secs(30),
                ),
            );
            println!("  Run `sudo reboot` after workloads complete.");
            Ok(warnings.finish_as(StepStatus::Blocked))
        }
        Ok(false) => {
            println!("  System will reboot now.");
            context
                .executor
                .run(&CommandSpec::new("reboot", std::iter::empty::<&str>()).with_sudo())?;
            Ok(StepOutcome::ok())
        }
        Err(err) => {
            let mut warnings = WarningCollector::new();
            warnings.warn(format!("GPU workload detection failed: {err}"));
            println!("  Skipping auto-reboot because GPU workload status could not be determined.");
            Ok(warnings.finish_as(StepStatus::Blocked))
        }
    }
}

fn gpu_workloads_running<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<bool>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.platform.expects_nvidia_tooling() {
        return Ok(false);
    }

    let output = context.executor.capture(
        &CommandSpec::new(
            "nvidia-smi",
            ["--query-compute-apps=pid", "--format=csv,noheader"],
        )
        .with_timeout_secs(30),
    )?;
    Ok(!output.trim().is_empty())
}

fn first_non_empty_line(text: &str) -> &str {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("no details")
}

#[cfg(test)]
mod tests {
    use super::{check_reboot, postflight_verify};
    use crate::features::system_updater::application::maintenance::MaintenanceContext;
    use crate::features::system_updater::domain::command::CommandSpec;
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::error::InfrastructureError;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};
    use std::path::PathBuf;

    #[test]
    fn does_not_reboot_without_marker() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let platform = PlatformInfo::default();
        let context = MaintenanceContext {
            config: &Config::default(),
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = check_reboot(&context).expect("no reboot required");
        assert_eq!(outcome.status.as_str(), "skipped");
        assert!(!executor.commands().contains(&"sudo reboot".to_string()));
    }

    #[test]
    fn reboots_when_marker_exists_and_no_gpu_workload_is_running() {
        let mut host = FakeHost::new();
        host.add_file("/var/run/reboot-required", "");

        let executor = FakeExecutor::new(false);
        executor.push_run_ok("sudo reboot");
        let reporter = FakeReporter::new();
        let platform = PlatformInfo::default();
        let context = MaintenanceContext {
            config: &Config::default(),
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = check_reboot(&context).expect("reboot should be triggered");
        assert_eq!(outcome.status.as_str(), "ok");
        assert!(executor.commands().contains(&"sudo reboot".to_string()));
    }

    #[test]
    fn skips_auto_reboot_when_gpu_probe_fails() {
        let mut host = FakeHost::new();
        host.add_file("/var/run/reboot-required", "");
        host.add_command("nvidia-smi", PathBuf::from("/usr/bin/nvidia-smi"), false);

        let executor = FakeExecutor::new(false);
        executor.push_capture_error(
            "nvidia-smi --query-compute-apps=pid --format=csv,noheader",
            InfrastructureError::command_failed(
                "INFRA_COMMAND_FAILED",
                &CommandSpec::new(
                    "nvidia-smi",
                    ["--query-compute-apps=pid", "--format=csv,noheader"],
                ),
                Some(1),
                "driver unavailable",
            ),
        );
        let reporter = FakeReporter::new();
        let platform = PlatformInfo::nvidia_linux(Some("NVIDIA Workstation".into()), "test");
        let context = MaintenanceContext {
            config: &Config::default(),
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = check_reboot(&context).expect("blocked instead of reboot");
        assert_eq!(outcome.status.as_str(), "blocked");
        assert!(!executor.commands().contains(&"sudo reboot".to_string()));
    }

    #[test]
    fn blocks_auto_reboot_when_nvidia_platform_lacks_nvidia_smi() {
        let mut host = FakeHost::new();
        host.add_file("/var/run/reboot-required", "");

        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let platform = PlatformInfo::nvidia_linux(Some("Rack GPU Host".into()), "test");
        let context = MaintenanceContext {
            config: &Config::default(),
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = check_reboot(&context).expect("blocked without nvidia-smi");
        assert_eq!(outcome.status.as_str(), "blocked");
        assert!(!executor.commands().contains(&"sudo reboot".to_string()));
    }

    #[test]
    fn postflight_warns_when_upgradable_packages_remain() {
        let mut host = FakeHost::new();
        host.add_command("apt-mark", PathBuf::from("/usr/bin/apt-mark"), false);
        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("dpkg --audit", "");
        executor.push_capture_ok(
            "apt list --upgradable",
            "Listing...\nlinux-generic/now 1.2 upgradable from: 1.1\n",
        );
        executor.push_capture_ok("apt-mark showhold", "");
        let reporter = FakeReporter::new();
        let platform = PlatformInfo::default();
        let context = MaintenanceContext {
            config: &Config::default(),
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = postflight_verify(&context).expect("postflight");
        assert_eq!(outcome.status.as_str(), "partial");
    }

    #[test]
    fn postflight_warns_when_nvidia_tooling_is_missing_on_nvidia_linux() {
        let mut host = FakeHost::new();
        host.add_command("apt-mark", PathBuf::from("/usr/bin/apt-mark"), false);
        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("dpkg --audit", "");
        executor.push_capture_ok("apt list --upgradable", "Listing...\n");
        executor.push_capture_ok("apt-mark showhold", "");
        let reporter = FakeReporter::new();
        let platform = PlatformInfo::nvidia_linux(Some("GPU Server".into()), "test");
        let context = MaintenanceContext {
            config: &Config::default(),
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = postflight_verify(&context).expect("postflight");
        assert_eq!(outcome.status.as_str(), "partial");
        assert!(
            outcome
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("nvidia-smi"))
        );
    }
}
