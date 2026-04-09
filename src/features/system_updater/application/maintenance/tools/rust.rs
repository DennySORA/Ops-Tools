use crate::features::system_updater::application::maintenance::tools::common::{
    filter_targets, parse_cargo_install_list,
};
use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::{StepOutcome, StepStatus};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn update_rust<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let mut warnings = WarningCollector::new();
    if context.host.command_path("rustup").is_some() {
        warnings.capture(
            "rustup update",
            context
                .executor
                .run(&CommandSpec::new("rustup", ["update"]).with_timeout_secs(3600)),
        );
    } else {
        println!("  rustup not found, skipping toolchain update.");
    }

    if context.host.command_path("cargo").is_some() {
        if context
            .executor
            .capture(&CommandSpec::new("cargo", ["install-update", "-V"]))
            .is_ok()
        {
            println!("  Updating cargo-installed binaries...");
            if context.config.tools.rust.cargo_allow.is_empty()
                && context.config.tools.rust.cargo_deny.is_empty()
            {
                warnings.capture(
                    "cargo install-update -a",
                    context.executor.run(
                        &CommandSpec::new("cargo", ["install-update", "-a"])
                            .with_timeout_secs(3600),
                    ),
                );
            } else {
                let listed = context
                    .executor
                    .capture(&CommandSpec::new("cargo", ["install", "--list"]))?;
                let crates = filter_targets(
                    parse_cargo_install_list(&listed),
                    &context.config.tools.rust.cargo_allow,
                    &context.config.tools.rust.cargo_deny,
                );
                if crates.is_empty() {
                    println!(
                        "  No cargo binaries matched policy filters, skipping binary upgrades."
                    );
                } else {
                    let mut args = vec!["install-update".to_string()];
                    args.extend(crates);
                    warnings.capture(
                        "cargo install-update [filtered]",
                        context
                            .executor
                            .run(&CommandSpec::new("cargo", args).with_timeout_secs(3600)),
                    );
                }
            }
        } else {
            println!("  cargo-update not installed, skipping binary upgrades.");
            println!("  Install with: cargo install cargo-update");
        }
    }

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("rust maintenance previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}
