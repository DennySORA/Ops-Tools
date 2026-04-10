use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::{StepOutcome, StepStatus};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn update_homebrew<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.brew.enabled {
        println!("  Homebrew updates disabled in config, skipping.");
        return Ok(StepOutcome::skipped("homebrew updates disabled in config"));
    }
    if context.host.command_path("brew").is_none() {
        println!("  Homebrew not found, skipping.");
        return Ok(StepOutcome::skipped("homebrew not installed"));
    }

    let mut warnings = WarningCollector::new();
    warnings.capture(
        "brew update",
        context
            .executor
            .run(&CommandSpec::new("brew", ["update"]).with_timeout_secs(1800)),
    );
    warnings.capture(
        "brew upgrade --formula",
        context
            .executor
            .run(&CommandSpec::new("brew", ["upgrade", "--formula"]).with_timeout_secs(3600)),
    );

    let cask_args = if context.config.brew.greedy_casks {
        vec!["upgrade", "--cask", "--greedy"]
    } else {
        vec!["upgrade", "--cask"]
    };
    warnings.capture(
        format!("brew {}", cask_args.join(" ")),
        context
            .executor
            .run(&CommandSpec::new("brew", cask_args).with_timeout_secs(3600)),
    );

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("homebrew update and upgrade previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

pub fn cleanup_homebrew<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("brew").is_none() {
        println!("  Homebrew not found, skipping cleanup.");
        return Ok(StepOutcome::skipped("homebrew not installed"));
    }
    if !context.config.brew.autoremove && !context.config.brew.cleanup {
        return Ok(StepOutcome::skipped("homebrew cleanup disabled in config"));
    }

    let mut warnings = WarningCollector::new();
    if context.config.brew.autoremove {
        warnings.capture(
            "brew autoremove",
            context
                .executor
                .run(&CommandSpec::new("brew", ["autoremove"]).with_timeout_secs(1800)),
        );
    }
    if context.config.brew.cleanup {
        warnings.capture(
            "brew cleanup --prune=all",
            context
                .executor
                .run(&CommandSpec::new("brew", ["cleanup", "--prune=all"]).with_timeout_secs(1800)),
        );
    }

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("homebrew cleanup previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

#[cfg(test)]
mod tests {
    use super::{cleanup_homebrew, update_homebrew};
    use crate::features::system_updater::application::maintenance::MaintenanceContext;
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};
    use std::path::PathBuf;

    #[test]
    fn skips_update_when_brew_is_missing() {
        let host = FakeHost::new();
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

        let outcome = update_homebrew(&context).expect("brew step");
        assert_eq!(outcome.status.as_str(), "skipped");
    }

    #[test]
    fn runs_brew_upgrade_and_cleanup() {
        let mut host = FakeHost::new();
        host.add_command("brew", PathBuf::from("/opt/homebrew/bin/brew"), false);
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

        update_homebrew(&context).expect("brew update");
        cleanup_homebrew(&context).expect("brew cleanup");

        let commands = executor.commands();
        assert!(commands.contains(&"brew update".to_string()));
        assert!(commands.contains(&"brew upgrade --formula".to_string()));
        assert!(commands.contains(&"brew upgrade --cask".to_string()));
        assert!(commands.contains(&"brew autoremove".to_string()));
        assert!(commands.contains(&"brew cleanup --prune=all".to_string()));
    }
}
