use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::config::CleanupStrategy;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::{StepOutcome, StepStatus};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn update_snap<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("snap").is_none() {
        println!("  snap not found, skipping.");
        return Ok(StepOutcome::skipped("snap not installed"));
    }

    let mut warnings = WarningCollector::new();
    warnings.capture(
        "snap refresh",
        context
            .executor
            .run(&CommandSpec::new("snap", ["refresh"]).with_sudo()),
    );
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("snap refresh previewed")
    } else {
        warnings.finish()
    })
}

pub fn update_flatpak<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("flatpak").is_none() {
        println!("  flatpak not found, skipping.");
        return Ok(StepOutcome::skipped("flatpak not installed"));
    }

    let mut warnings = WarningCollector::new();
    warnings.capture(
        "flatpak update",
        context.executor.run(&CommandSpec::new(
            "flatpak",
            ["update", "-y", "--noninteractive"],
        )),
    );
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("flatpak update previewed")
    } else {
        warnings.finish()
    })
}

pub fn update_docker<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("docker").is_none() {
        println!("  Docker not found, skipping.");
        return Ok(StepOutcome::skipped("docker not installed"));
    }

    let mut warnings = WarningCollector::new();
    println!("  Docker Engine upgrades are covered by APT full-upgrade.");

    if context.config.docker.compose_projects.is_empty() {
        println!("  No compose projects configured, skipping Compose updates.");
        return Ok(StepOutcome::skipped(
            "docker installed but no compose projects configured",
        ));
    }

    for directory in &context.config.docker.compose_projects {
        if !context.host.is_dir(directory) {
            warnings.warn(format!(
                "Compose dir does not exist: {}",
                directory.display()
            ));
            continue;
        }

        println!("  Updating Compose project: {}", directory.display());
        warnings.capture(
            format!("docker compose pull failed for {}", directory.display()),
            context.executor.run(
                &CommandSpec::new("docker", ["compose", "pull", "--include-deps"])
                    .with_sudo()
                    .with_cwd(directory.clone()),
            ),
        );
        warnings.capture(
            format!("docker compose up -d failed for {}", directory.display()),
            context.executor.run(
                &CommandSpec::new("docker", ["compose", "up", "-d"])
                    .with_sudo()
                    .with_cwd(directory.clone()),
            ),
        );
    }

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("docker compose maintenance previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

pub fn cleanup_docker<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("docker").is_none() {
        println!("  Docker not found, skipping cleanup.");
        return Ok(StepOutcome::skipped("docker not installed"));
    }
    if !context.config.docker.prune
        && !context.config.cleanup.docker_system_prune
        && !context.config.cleanup.docker_builder_prune
        && !context.config.cleanup.docker_volume_prune
    {
        return Ok(StepOutcome::skipped("docker cleanup disabled in config"));
    }

    let mut warnings = WarningCollector::new();
    if context.config.docker.prune || context.config.cleanup.docker_system_prune {
        warnings.capture(
            "docker system prune",
            context
                .executor
                .run(&CommandSpec::new("docker", ["system", "prune", "-f"]).with_sudo()),
        );
    }
    if matches!(context.config.cleanup.strategy, CleanupStrategy::Aggressive)
        || context.config.cleanup.docker_builder_prune
    {
        warnings.capture(
            "docker builder prune",
            context
                .executor
                .run(&CommandSpec::new("docker", ["builder", "prune", "-af"]).with_sudo()),
        );
    }
    if matches!(context.config.cleanup.strategy, CleanupStrategy::Aggressive)
        || context.config.cleanup.docker_volume_prune
    {
        warnings.capture(
            "docker volume prune",
            context
                .executor
                .run(&CommandSpec::new("docker", ["volume", "prune", "-f"]).with_sudo()),
        );
    }

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("docker cleanup previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

pub fn run_needrestart<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("needrestart").is_none() {
        println!("  needrestart not found, skipping.");
        return Ok(StepOutcome::skipped("needrestart not installed"));
    }

    let mut args = vec!["needrestart".to_string(), "-r".into(), "a".into()];
    for service in &context.config.runtime.needrestart_reject {
        args.push("-b".into());
        args.push(format!("/{service}/"));
    }
    let mut warnings = WarningCollector::new();
    warnings.capture(
        "needrestart",
        context.executor.run(&CommandSpec::new("sudo", args)),
    );
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("needrestart previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}
