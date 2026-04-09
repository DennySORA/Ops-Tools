use crate::features::system_updater::application::maintenance::tools::common::{
    filter_targets, parse_simple_name_list, self_update_allowed,
};
use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::config::CleanupStrategy;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::{StepOutcome, StepStatus};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn update_pipx<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.tools.pipx.enabled {
        println!("  pipx upgrades disabled in config, skipping.");
        return Ok(StepOutcome::skipped("pipx updates disabled in config"));
    }
    if context.host.command_path("pipx").is_none() {
        println!("  pipx not found, skipping.");
        return Ok(StepOutcome::skipped("pipx not installed"));
    }

    if context.config.tools.pipx.allow.is_empty() && context.config.tools.pipx.deny.is_empty() {
        context.executor.run(
            &CommandSpec::new("pipx", ["upgrade-all", "--include-injected"])
                .with_timeout_secs(1800),
        )?;
    } else {
        let listed = context
            .executor
            .capture(&CommandSpec::new("pipx", ["list", "--short"]))?;
        let packages = filter_targets(
            parse_simple_name_list(&listed),
            &context.config.tools.pipx.allow,
            &context.config.tools.pipx.deny,
        );
        if packages.is_empty() {
            return Ok(StepOutcome::skipped("no pipx apps matched policy filters"));
        }
        for package in packages {
            context.executor.run(
                &CommandSpec::new("pipx", ["upgrade", package.as_str()]).with_timeout_secs(900),
            )?;
        }
    }
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("pipx maintenance previewed")
    } else {
        StepOutcome::ok()
    })
}

pub fn update_conda<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.tools.conda.enabled {
        println!("  Conda updates disabled in config, skipping.");
        return Ok(StepOutcome::skipped("conda updates disabled in config"));
    }
    if context.host.command_path("conda").is_none() {
        println!("  conda not found, skipping.");
        return Ok(StepOutcome::skipped("conda not installed"));
    }
    if context.config.tools.conda.envs.is_empty() {
        println!("  No conda environments configured, skipping.");
        return Ok(StepOutcome::skipped("no conda environments configured"));
    }

    let mut warnings = WarningCollector::new();
    for environment in &context.config.tools.conda.envs {
        warnings.capture(
            format!("conda update --all [{environment}]"),
            context.executor.run(
                &CommandSpec::new(
                    "conda",
                    ["update", "--yes", "--name", environment.as_str(), "--all"],
                )
                .with_timeout_secs(3600),
            ),
        );
    }
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("conda maintenance previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

pub fn update_uv<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.tools.uv.enabled {
        println!("  uv updates disabled in config, skipping.");
        return Ok(StepOutcome::skipped("uv updates disabled in config"));
    }
    if context.host.command_path("uv").is_none() {
        println!("  uv not found, skipping.");
        return Ok(StepOutcome::skipped("uv not installed"));
    }

    let mut warnings = WarningCollector::new();
    if self_update_allowed(context, "uv", "uv") {
        warnings.capture(
            "uv self update",
            context
                .executor
                .run(&CommandSpec::new("uv", ["self", "update"]).with_timeout_secs(900)),
        );
    }

    if context.config.tools.uv.tool_allow.is_empty() && context.config.tools.uv.tool_deny.is_empty()
    {
        warnings.capture(
            "uv tool upgrade --all",
            context
                .executor
                .run(&CommandSpec::new("uv", ["tool", "upgrade", "--all"]).with_timeout_secs(1800)),
        );
    } else {
        let listed = context
            .executor
            .capture(&CommandSpec::new("uv", ["tool", "list"]))?;
        let tools = filter_targets(
            parse_simple_name_list(&listed),
            &context.config.tools.uv.tool_allow,
            &context.config.tools.uv.tool_deny,
        );
        if tools.is_empty() {
            println!("  No uv tools matched policy filters, skipping tool upgrades.");
        } else {
            for tool in tools {
                warnings.capture(
                    format!("uv tool upgrade [{tool}]"),
                    context.executor.run(
                        &CommandSpec::new("uv", ["tool", "upgrade", tool.as_str()])
                            .with_timeout_secs(900),
                    ),
                );
            }
        }
    }

    warnings.capture(
        "uv python upgrade",
        context
            .executor
            .run(&CommandSpec::new("uv", ["python", "upgrade"]).with_timeout_secs(1800)),
    );
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("uv maintenance previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

pub fn cleanup_caches<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let mut warnings = WarningCollector::new();
    let mut any_action = false;

    if context.config.cleanup.flatpak_unused {
        any_action = true;
        if context.host.command_path("flatpak").is_some() {
            warnings.capture(
                "flatpak uninstall --unused",
                context.executor.run(&CommandSpec::new(
                    "flatpak",
                    ["uninstall", "--unused", "-y", "--noninteractive"],
                )),
            );
        } else {
            println!("  flatpak not found, skipping unused runtime cleanup.");
        }
    }

    if context.config.cleanup.uv_cache {
        any_action = true;
        if context.host.command_path("uv").is_some() {
            warnings.capture(
                "uv cache prune",
                context
                    .executor
                    .run(&CommandSpec::new("uv", ["cache", "prune"]).with_timeout_secs(900)),
            );
        } else {
            println!("  uv not found, skipping uv cache cleanup.");
        }
    }

    if context.config.cleanup.pip_cache {
        any_action = true;
        if context.host.command_path("python3").is_some() {
            warnings.capture(
                "python3 -m pip cache purge",
                context.executor.run(&CommandSpec::new(
                    "python3",
                    ["-m", "pip", "cache", "purge"],
                )),
            );
        } else if context.host.command_path("pip3").is_some() {
            warnings.capture(
                "pip3 cache purge",
                context
                    .executor
                    .run(&CommandSpec::new("pip3", ["cache", "purge"])),
            );
        } else {
            println!("  pip not found, skipping pip cache cleanup.");
        }
    }

    if context.config.cleanup.conda_cache {
        any_action = true;
        if context.host.command_path("conda").is_some() {
            warnings.capture(
                "conda clean --all --yes",
                context.executor.run(
                    &CommandSpec::new("conda", ["clean", "--all", "--yes"]).with_timeout_secs(1800),
                ),
            );
        } else {
            println!("  conda not found, skipping conda cache cleanup.");
        }
    }

    if context.config.cleanup.bun_cache {
        any_action = true;
        if context.host.command_path("bun").is_some() {
            warnings.capture(
                "bun pm cache rm",
                context
                    .executor
                    .run(&CommandSpec::new("bun", ["pm", "cache", "rm"]).with_timeout_secs(900)),
            );
        } else {
            println!("  bun not found, skipping Bun cache cleanup.");
        }
    }

    if context.config.cleanup.deno_cache {
        any_action = true;
        if context.host.command_path("deno").is_some() {
            warnings.capture(
                "deno clean",
                context
                    .executor
                    .run(&CommandSpec::new("deno", ["clean"]).with_timeout_secs(900)),
            );
        } else {
            println!("  deno not found, skipping Deno cache cleanup.");
        }
    }

    if matches!(context.config.cleanup.strategy, CleanupStrategy::Aggressive) {
        println!("  Aggressive cleanup strategy enabled.");
    }

    if !any_action {
        println!("  Cache cleanup disabled in config, skipping.");
        return Ok(StepOutcome::skipped("cache cleanup disabled in config"));
    }

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("tool cache cleanup previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}
