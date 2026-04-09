use crate::features::system_updater::application::maintenance::MaintenanceContext;
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};
use std::path::Path;

pub fn self_update_allowed<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    binary: &str,
    label: &str,
) -> bool
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    match context.host.command_path(binary) {
        Some(path) if context.host.is_writable(&path) => true,
        Some(path) => {
            println!(
                "  {label} binary is not writable ({}), skipping self-update.",
                path.display()
            );
            false
        }
        None => false,
    }
}

pub fn filter_targets(all: Vec<String>, allow: &[String], deny: &[String]) -> Vec<String> {
    all.into_iter()
        .filter(|item| allow.is_empty() || allow.iter().any(|candidate| candidate == item))
        .filter(|item| !deny.iter().any(|candidate| candidate == item))
        .collect()
}

pub fn npm_prefix<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<String>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    Ok(context
        .executor
        .capture(&CommandSpec::new("npm", ["config", "get", "prefix"]))?
        .trim()
        .to_string())
}

pub fn run_npm_update<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    packages: &[String],
) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let prefix = npm_prefix(context)?;
    let base_command = if packages.is_empty() {
        CommandSpec::new("npm", ["update", "-g"])
    } else {
        let mut args = vec!["update".to_string(), "-g".into()];
        args.extend(packages.iter().cloned());
        CommandSpec::new("npm", args)
    };

    if !prefix.is_empty() && context.host.is_writable(Path::new(&prefix)) {
        context.executor.run(&base_command)?;
    } else {
        context.executor.run(&base_command.with_sudo())?;
    }
    Ok(())
}

pub fn parse_parseable_package_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.rsplit('/').next())
        .filter(|line| !line.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn parse_cargo_install_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| !line.starts_with(' ') && !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .map(|name| name.trim_end_matches(':').to_string())
        .collect()
}

pub fn parse_simple_name_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|line| !line.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
