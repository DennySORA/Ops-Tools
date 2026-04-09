use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::StepOutcome;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};
use std::path::PathBuf;

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

pub fn install_cuda_and_configure<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if let Some(outcome) = skip_if_not_gb10(context, "GB10 CUDA toolkit configuration") {
        return Ok(outcome);
    }

    let package = format!("cuda-toolkit-{}", context.config.dgx.cuda_major);
    context
        .executor
        .run(&CommandSpec::new("apt", ["-y", "install", package.as_str()]).with_sudo())?;
    configure_cuda_zshrc(context)
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

fn configure_cuda_zshrc<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let home = context.host.var("HOME").unwrap_or_default();
    let zshrc = PathBuf::from(&home).join(".zshrc");
    let backup = PathBuf::from(&home).join(".zshrc.bak");
    let temp = PathBuf::from(&home).join(".zshrc.tmp");

    let block_start = "# >>> CUDA DGX Spark >>>";
    let block_end = "# <<< CUDA DGX Spark <<<";
    let cuda_block = format!(
        r#"{block_start}
export CUDA_HOME="/usr/local/cuda"
export PATH="$CUDA_HOME/bin${{PATH:+:$PATH}}"
export LD_LIBRARY_PATH="$CUDA_HOME/lib64${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}"
# DGX Spark (GB10) native compute capability
export CUDAARCHS="{arch}-real"
export CMAKE_CUDA_ARCHITECTURES="{arch}-real"
{block_end}"#,
        arch = context.config.dgx.cuda_arch,
    );

    if context.executor.is_dry_run() {
        println!("  [dry-run] would update CUDA block in {}", zshrc.display());
        return Ok(StepOutcome::dry_run(format!(
            "CUDA shell block would be updated in {}",
            zshrc.display()
        )));
    }

    let existing = if context.host.exists(&zshrc) {
        context.host.read_to_string(&zshrc)?
    } else {
        String::new()
    };

    if context.host.exists(&zshrc) {
        context.host.copy_file(&zshrc, &backup)?;
        println!("  Backed up .zshrc -> .zshrc.bak");
    }

    let new_content = rewrite_cuda_block(&existing, block_start, block_end, &cuda_block);
    context.host.write_string(&temp, &new_content)?;
    context.host.rename(&temp, &zshrc)?;
    println!("  CUDA environment configured in .zshrc");
    Ok(StepOutcome::ok())
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

fn rewrite_cuda_block(
    existing: &str,
    block_start: &str,
    block_end: &str,
    replacement: &str,
) -> String {
    if existing.contains(block_start) {
        let mut lines = Vec::new();
        let mut in_block = false;
        for line in existing.lines() {
            if line.contains(block_start) {
                in_block = true;
                continue;
            }
            if line.contains(block_end) {
                in_block = false;
                continue;
            }
            if !in_block {
                lines.push(line);
            }
        }
        while lines.last() == Some(&"") {
            lines.pop();
        }
        format!("{}\n\n{}\n", lines.join("\n"), replacement)
    } else if existing.trim().is_empty() {
        format!("{replacement}\n")
    } else {
        format!("{}\n\n{}\n", existing.trim_end(), replacement)
    }
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
        let platform = PlatformInfo::generic_linux(Some("Dell PowerEdge".into()));
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
        let platform = PlatformInfo::nvidia_linux(Some("RTX Workstation".into()), "test");
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
