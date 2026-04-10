use crate::features::system_updater::application::maintenance::MaintenanceContext;
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::StepOutcome;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn run_software_update<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.macos.software_update {
        println!("  macOS software updates disabled in config, skipping.");
        return Ok(StepOutcome::skipped(
            "macOS software updates disabled in config",
        ));
    }
    if context.host.command_path("softwareupdate").is_none() {
        println!("  softwareupdate not found, skipping.");
        return Ok(StepOutcome::skipped("softwareupdate not available"));
    }

    let mut args = vec!["--install"];
    if context.config.macos.recommended_only {
        args.push("--recommended");
    } else {
        args.push("--all");
    }
    if context.config.macos.allow_restart && context.config.runtime.auto_reboot {
        args.push("--restart");
    }

    context.executor.run(
        &CommandSpec::new("softwareupdate", args)
            .with_sudo()
            .with_timeout_secs(7200),
    )?;

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("macOS softwareupdate previewed")
    } else {
        StepOutcome::ok()
    })
}

#[cfg(test)]
mod tests {
    use super::run_software_update;
    use crate::features::system_updater::application::maintenance::MaintenanceContext;
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};
    use std::path::PathBuf;

    #[test]
    fn runs_recommended_updates_by_default() {
        let mut host = FakeHost::new();
        host.add_command(
            "softwareupdate",
            PathBuf::from("/usr/sbin/softwareupdate"),
            false,
        );
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let config = Config::default();
        let platform = PlatformInfo::macos(Some("Mac14,15".into()), Some("15.4".into()), None);
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        run_software_update(&context).expect("softwareupdate");
        assert!(
            executor
                .commands()
                .contains(&"sudo softwareupdate --install --recommended".to_string())
        );
    }
}
