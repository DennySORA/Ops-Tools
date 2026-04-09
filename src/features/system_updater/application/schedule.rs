use crate::features::system_updater::domain::config::SchedulingConfig;
use crate::features::system_updater::domain::error::{AppResult, InfrastructureError};
use std::path::Path;

pub fn print_systemd_templates(
    executable: &Path,
    config_path: Option<&Path>,
    profile: Option<&str>,
    job: &str,
    scheduling: &SchedulingConfig,
) -> AppResult<()> {
    let config_arg = config_path
        .map(|path| format!(" --config {}", path.display()))
        .unwrap_or_default();
    let profile_arg = profile
        .map(|value| format!(" --profile {value}"))
        .unwrap_or_default();
    let command = format!(
        "{} {}{}{}",
        executable.display(),
        job,
        config_arg,
        profile_arg
    );

    let service_name = format!("update-{}", job.replace(' ', "-"));
    println!("[Unit]");
    println!("Description=Update maintenance job ({job})");
    println!("After=network-online.target");
    println!();
    println!("[Service]");
    println!("Type=oneshot");
    println!("ExecStart={command}");
    println!();
    println!("# {}.timer", service_name);
    println!("[Unit]");
    println!("Description=Schedule update maintenance job ({job})");
    println!();
    println!("[Timer]");
    println!("OnCalendar={}", scheduling.on_calendar);
    println!(
        "RandomizedDelaySec={}",
        scheduling.randomized_delay_minutes.saturating_mul(60)
    );
    println!(
        "Persistent={}",
        if scheduling.persistent {
            "true"
        } else {
            "false"
        }
    );
    println!();
    println!("[Install]");
    println!("WantedBy=timers.target");
    Ok(())
}

pub fn current_executable() -> Result<std::path::PathBuf, InfrastructureError> {
    std::env::current_exe().map_err(|err| {
        InfrastructureError::probe("INFRA_CURRENT_EXE", "current_exe", err.to_string())
    })
}
