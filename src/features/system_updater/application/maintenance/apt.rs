use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::config::CleanupStrategy;
use crate::features::system_updater::domain::error::{AppResult, DomainError};
use crate::features::system_updater::domain::report::StepOutcome;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub fn update_and_upgrade<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.apt.hold_packages.is_empty() {
        let mut hold_args = vec!["apt-mark".to_string(), "hold".into()];
        hold_args.extend(context.config.apt.hold_packages.iter().cloned());
        context.executor.run(&CommandSpec::new("sudo", hold_args))?;
    }

    context
        .executor
        .run(&CommandSpec::new("apt", ["update"]).with_sudo())?;
    context
        .executor
        .run(&CommandSpec::new("apt", ["-y", "full-upgrade"]).with_sudo())?;
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("APT update and full-upgrade previewed")
    } else {
        StepOutcome::ok()
    })
}

pub fn install_maintenance_tools<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.config.apt.maintenance_packages.is_empty() {
        return Ok(StepOutcome::skipped(
            "no maintenance packages configured for installation",
        ));
    }

    let mut args = vec!["apt".to_string(), "-y".to_string(), "install".to_string()];
    args.extend(context.config.apt.maintenance_packages.iter().cloned());
    context.executor.run(&CommandSpec::new("sudo", args))?;
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("maintenance packages would be installed")
    } else {
        StepOutcome::ok()
    })
}

pub fn clean_rc_packages<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let output = context
        .executor
        .capture(&CommandSpec::new("dpkg", ["-l"]))?;
    let packages: Vec<String> = output
        .lines()
        .filter(|line| line.starts_with("rc "))
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(ToOwned::to_owned)
        .collect();

    if packages.is_empty() {
        println!("  No residual configs to clean.");
        return Ok(StepOutcome::skipped("no residual rc packages found"));
    }

    println!("  Purging {} residual config(s)", packages.len());
    for package in &packages {
        if !package
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_.+-:".contains(character))
        {
            return Err(DomainError::validation(
                "DOMAIN_PACKAGE_NAME_INVALID",
                format!("suspicious package name in dpkg output: {package}"),
            )
            .into());
        }
    }

    let mut args = vec!["apt".to_string(), "-y".to_string(), "purge".to_string()];
    args.extend(packages);
    context.executor.run(&CommandSpec::new("sudo", args))?;
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("rc package purge previewed")
    } else {
        StepOutcome::ok()
    })
}

pub fn purge_old_kernels<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("purge-old-kernels").is_none() {
        println!("  purge-old-kernels not found, skipping.");
        return Ok(StepOutcome::skipped("purge-old-kernels not installed"));
    }

    let mut warnings = WarningCollector::new();
    context
        .executor
        .run(&CommandSpec::new("purge-old-kernels", ["--keep", "2", "-qy"]).with_sudo())?;
    warnings.capture(
        "update-grub",
        context
            .executor
            .run(&CommandSpec::new("update-grub", std::iter::empty::<&str>()).with_sudo()),
    );
    let outcome = warnings.finish();
    Ok(
        if context.executor.is_dry_run()
            && outcome.status == crate::features::system_updater::domain::report::StepStatus::Ok
        {
            StepOutcome::dry_run("old kernel cleanup previewed")
        } else {
            outcome
        },
    )
}

pub fn autoremove_purge_and_clean<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.executor.is_dry_run() {
        println!("  [dry-run] would simulate apt-get -s autoremove before cleanup");
        context
            .executor
            .run(&CommandSpec::new("apt", ["-y", "autoremove", "--purge"]).with_sudo())?;
        context
            .executor
            .run(&CommandSpec::new("apt", ["-y", "autoclean"]).with_sudo())?;
        context
            .executor
            .run(&CommandSpec::new("apt", ["-y", "clean"]).with_sudo())?;
        return Ok(StepOutcome::dry_run(
            "APT autoremove/autoclean/clean previewed",
        ));
    }

    let simulation = context
        .executor
        .capture(&CommandSpec::new("apt-get", ["-s", "autoremove"]))?;
    let removable_count = autoremove_removal_count(&simulation);
    let protected_hits =
        protected_autoremove_hits(&simulation, &context.config.apt.protected_keywords);
    let denied_hits = denied_autoremove_hits(&simulation, &context.config.apt.deny_remove_packages);
    let mut warnings = WarningCollector::new();

    if protected_hits.is_empty() && denied_hits.is_empty() {
        if removable_count == 0 {
            println!("  No auto-removable packages detected.");
        } else {
            println!(
                "  Running sudo apt autoremove --purge for {removable_count} package(s) (-y enabled)."
            );
        }
        context
            .executor
            .run(&CommandSpec::new("apt", ["-y", "autoremove", "--purge"]).with_sudo())?;
    } else {
        warnings.warn("autoremove would remove protected or denied packages -- skipping!");
        for line in protected_hits.iter().chain(denied_hits.iter()) {
            println!("    {line}");
        }
        println!("  Please review and remove manually if safe.");
    }

    println!(
        "  Clearing APT caches with strategy {:?}",
        context.config.cleanup.strategy
    );
    warnings.capture(
        "apt autoclean",
        context
            .executor
            .run(&CommandSpec::new("apt", ["-y", "autoclean"]).with_sudo()),
    );
    if matches!(
        context.config.cleanup.strategy,
        CleanupStrategy::Normal | CleanupStrategy::Aggressive
    ) {
        warnings.capture(
            "apt clean",
            context
                .executor
                .run(&CommandSpec::new("apt", ["-y", "clean"]).with_sudo()),
        );
    }
    Ok(warnings.finish_as(crate::features::system_updater::domain::report::StepStatus::Partial))
}

fn autoremove_removal_count(simulation: &str) -> usize {
    simulation
        .lines()
        .filter(|line| line.starts_with("Remv "))
        .count()
}

fn protected_autoremove_hits<'a>(simulation: &'a str, keywords: &[String]) -> Vec<&'a str> {
    simulation
        .lines()
        .filter(|line| line.starts_with("Remv "))
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            keywords
                .iter()
                .any(|keyword| lower.contains(&keyword.to_ascii_lowercase()))
        })
        .collect()
}

fn denied_autoremove_hits<'a>(simulation: &'a str, packages: &[String]) -> Vec<&'a str> {
    simulation
        .lines()
        .filter(|line| line.starts_with("Remv "))
        .filter(|line| packages.iter().any(|package| line.contains(package)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{autoremove_removal_count, denied_autoremove_hits, protected_autoremove_hits};

    #[test]
    fn counts_autoremove_candidates() {
        let simulation = "Remv libc6 [1.0]\nRemv libssl [3.0]\nKeep something [1.0]\n";
        assert_eq!(autoremove_removal_count(simulation), 2);
    }

    #[test]
    fn detects_protected_packages_using_configured_keywords() {
        let simulation = "\
Remv libc6 [1.0]\n\
Remv nvidia-driver-580-open [580.1]\n\
Remv cuda-toolkit-13-0 [13.0]\n\
Remv nccl-tests [2.0]\n";
        let keywords = vec![
            "nvidia".to_string(),
            "dgx".to_string(),
            "cuda".to_string(),
            "nccl".to_string(),
        ];

        let hits = protected_autoremove_hits(simulation, &keywords);
        assert_eq!(hits.len(), 3);
    }

    #[test]
    fn detects_denied_package_removals() {
        let simulation = "Remv libc6 [1.0]\nRemv package-a [3.0]\n";
        let hits = denied_autoremove_hits(simulation, &["package-a".into()]);
        assert_eq!(hits.len(), 1);
    }
}
