use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::StepOutcome;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn install_kernel_driver<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if let Some(outcome) = skip_if_not_gb10(context, "GB10 kernel and driver installation") {
        return Ok(outcome);
    }

    context.executor.run(
        &CommandSpec::new(
            "apt",
            [
                "-y",
                "install",
                context.config.dgx.kernel_meta.as_str(),
                context.config.dgx.headers_meta.as_str(),
                context.config.dgx.driver_pkg.as_str(),
                context.config.dgx.modules_pkg.as_str(),
            ],
        )
        .with_sudo(),
    )?;
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("DGX kernel and driver installation previewed")
    } else {
        StepOutcome::ok()
    })
}

pub fn verify_watchdog<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if let Some(outcome) = skip_if_not_gb10(context, "GB10 watchdog verification") {
        return Ok(outcome);
    }

    if context.executor.is_dry_run() {
        println!("  [dry-run] would check sbsa_gwdt module");
        return Ok(StepOutcome::dry_run("DGX watchdog probe previewed"));
    }

    let output = context
        .executor
        .capture(&CommandSpec::new("lsmod", std::iter::empty::<&str>()))?;
    if output.lines().any(|line| line.starts_with("sbsa_gwdt")) {
        println!("  sbsa_gwdt module loaded -- watchdog OK.");
        return Ok(StepOutcome::ok());
    }

    println!("  !! sbsa_gwdt not loaded, attempting modprobe...");
    let mut warnings = WarningCollector::new();
    warnings.capture(
        "modprobe sbsa_gwdt",
        context
            .executor
            .run(&CommandSpec::new("modprobe", ["sbsa_gwdt"]).with_sudo()),
    );
    Ok(warnings.finish())
}

fn skip_if_not_gb10<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    capability: &str,
) -> Option<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    (!context.platform.supports_gb10_tuning()).then(|| {
        StepOutcome::skipped(format!(
            "{capability} requires a GB10 host; detected {}",
            context.platform.summary()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::install_kernel_driver;
    use crate::features::system_updater::application::maintenance::MaintenanceContext;
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::domain::report::StepStatus;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};

    #[test]
    fn skips_gb10_step_on_generic_linux() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let config = Config::default();
        let platform = PlatformInfo::generic_linux(Some("Dell PowerEdge".into()), None, None);
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = install_kernel_driver(&context).expect("gb10 step");
        assert_eq!(outcome.status, StepStatus::Skipped);
        assert!(executor.commands().is_empty());
    }

    #[test]
    fn skips_gb10_step_on_generic_nvidia_linux() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let config = Config::default();
        let platform =
            PlatformInfo::nvidia_linux(Some("RTX Workstation".into()), None, None, "test");
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = install_kernel_driver(&context).expect("gb10 step");
        assert_eq!(outcome.status, StepStatus::Skipped);
        assert!(
            outcome
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("NVIDIA Linux"))
        );
        assert!(executor.commands().is_empty());
    }
}
