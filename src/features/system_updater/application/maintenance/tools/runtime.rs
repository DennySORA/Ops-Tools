use crate::features::system_updater::application::maintenance::MaintenanceContext;
use crate::features::system_updater::application::maintenance::tools::common::self_update_allowed;
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::StepOutcome;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn update_bun<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.tools.bun.enabled {
        println!("  Bun updates disabled in config, skipping.");
        return Ok(StepOutcome::skipped("bun updates disabled in config"));
    }
    if context.host.command_path("bun").is_none() {
        println!("  bun not found, skipping.");
        return Ok(StepOutcome::skipped("bun not installed"));
    }
    if !self_update_allowed(context, "bun", "Bun") {
        return Ok(StepOutcome::skipped("bun binary is not writable"));
    }
    context
        .executor
        .run(&CommandSpec::new("bun", ["upgrade"]).with_timeout_secs(600))?;
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("bun upgrade previewed")
    } else {
        StepOutcome::ok()
    })
}

pub fn update_deno<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.tools.deno.enabled {
        println!("  Deno updates disabled in config, skipping.");
        return Ok(StepOutcome::skipped("deno updates disabled in config"));
    }
    if context.host.command_path("deno").is_none() {
        println!("  deno not found, skipping.");
        return Ok(StepOutcome::skipped("deno not installed"));
    }
    if !self_update_allowed(context, "deno", "Deno") {
        return Ok(StepOutcome::skipped("deno binary is not writable"));
    }
    context
        .executor
        .run(&CommandSpec::new("deno", ["upgrade"]).with_timeout_secs(600))?;
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("deno upgrade previewed")
    } else {
        StepOutcome::ok()
    })
}

#[cfg(test)]
mod tests {
    use super::update_bun;
    use crate::features::system_updater::application::maintenance::MaintenanceContext;
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};
    use std::path::PathBuf;

    #[test]
    fn skips_self_update_when_binary_is_not_writable() {
        let mut host = FakeHost::new();
        host.add_command("bun", PathBuf::from("/usr/bin/bun"), false);
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

        let outcome = update_bun(&context).expect("bun update");
        assert_eq!(outcome.status.as_str(), "skipped");
        assert!(executor.commands().is_empty());
    }
}
