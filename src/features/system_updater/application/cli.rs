use crate::features::system_updater::application::platform;
use crate::features::system_updater::application::preflight;
use crate::features::system_updater::application::report_cmd;
use crate::features::system_updater::application::scan;
use crate::features::system_updater::application::schedule;
use crate::features::system_updater::application::workflow::{self, StepSelection};
use crate::features::system_updater::domain::config::Config;
use crate::features::system_updater::domain::error::{AppResult, DomainError, InfrastructureError};
use crate::features::system_updater::domain::platform::PlatformInfo;
use crate::features::system_updater::domain::report::{
    RunContext, RunStatus, StepEvent, StepGroup, StepStatus,
};
use crate::features::system_updater::infrastructure::config::load_config;
use crate::features::system_updater::infrastructure::host::HostRuntime;
use crate::features::system_updater::infrastructure::lock::RunLock;
use crate::features::system_updater::infrastructure::reporter::JsonFileReporter;
use crate::features::system_updater::infrastructure::shell::ShellCommandExecutor;
use crate::features::system_updater::ports::{
    CommandExecutor, CommandObserver, RunReporter, SystemProbe,
};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliOptions {
    pub command: CliCommand,
    pub dry_run: bool,
    pub config_path: Option<PathBuf>,
    pub profile: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliCommand {
    Run,
    Scan,
    Cleanup,
    Verify,
    Backup,
    ReportShow {
        selector: Option<String>,
    },
    ReportDiff {
        left: Option<String>,
        right: Option<String>,
    },
    ReportList {
        limit: usize,
    },
    SchedulePrintSystemd {
        job: String,
    },
}

impl CliCommand {
    fn label(&self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Scan => "scan",
            Self::Cleanup => "cleanup",
            Self::Verify => "verify",
            Self::Backup => "backup",
            Self::ReportShow { .. } => "report-show",
            Self::ReportDiff { .. } => "report-diff",
            Self::ReportList { .. } => "report-list",
            Self::SchedulePrintSystemd { .. } => "schedule-print-systemd",
        }
    }
}

pub fn run_from_env() -> ExitCode {
    match execute_from_env() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

/// Execute with pre-built options (called from interactive menu).
pub fn execute(options: CliOptions) {
    HostRuntime::extend_path_for_common_tools();
    if let Err(err) = execute_runtime_command(options) {
        eprintln!("{err}");
    }
}

fn execute_from_env() -> AppResult<()> {
    let options = parse_args(std::env::args().skip(1))?;
    HostRuntime::extend_path_for_common_tools();

    match &options.command {
        CliCommand::ReportShow { selector } => execute_report_show(&options, selector.as_deref()),
        CliCommand::ReportDiff { left, right } => {
            execute_report_diff(&options, left.as_deref(), right.as_deref())
        }
        CliCommand::ReportList { limit } => execute_report_list(&options, *limit),
        CliCommand::SchedulePrintSystemd { job } => execute_schedule_print(&options, job),
        _ => execute_runtime_command(options),
    }
}

fn execute_runtime_command(options: CliOptions) -> AppResult<()> {
    println!();
    println!("  System Update & Scanner");
    println!("  =======================");
    if options.dry_run {
        println!();
        println!("  *** DRY RUN MODE -- no changes will be made ***");
    }

    let mut loaded = load_config(options.config_path.as_deref())?;
    if let Some(profile) = &options.profile {
        loaded
            .config
            .apply_profile(profile)
            .map_err(|message| DomainError::validation("DOMAIN_PROFILE_UNKNOWN", message))?;
    }

    let lock = RunLock::acquire(&loaded.config.runtime.lock_path)?;
    let host = HostRuntime;
    let reporter = JsonFileReporter::create(
        &loaded.config.report.dir,
        options.dry_run,
        matches!(options.command, CliCommand::Scan),
        RunContext {
            subcommand: options.command.label().to_string(),
            profile: options.profile.clone(),
            config_path: loaded.source.clone(),
            lock_path: Some(lock.path().to_path_buf()),
        },
    )?;
    let scan_executor = ShellCommandExecutor::new(false, reporter.clone());
    let executor = ShellCommandExecutor::new(options.dry_run, reporter.clone());
    let platform = platform::detect(&host, &scan_executor);

    if platform.supports_gb10_tuning() || platform.expects_nvidia_tooling() {
        super::dgx_detect::detect_and_merge(&host, &scan_executor, &mut loaded.config.dgx);
    }

    if let Err(err) = print_runtime_banner(
        &reporter,
        &loaded.source,
        &host,
        &options.profile,
        &platform,
    ) {
        best_effort_finalize(&reporter, RunStatus::Failed);
        return Err(err.into());
    }

    if matches!(options.command, CliCommand::Run | CliCommand::Scan) {
        scan::run_scan(&host, &scan_executor, &platform);
    }

    match options.command {
        CliCommand::Scan => {
            let artifacts = finalize(&reporter, RunStatus::ScanOnly)?;
            maybe_notify(
                &loaded.config,
                &artifacts,
                RunStatus::ScanOnly,
                "scan",
                options.profile.as_deref(),
                options.dry_run,
            )?;
            Ok(())
        }
        CliCommand::Backup => {
            let summary = workflow::run(
                &loaded.config,
                &platform,
                &host,
                &executor,
                &reporter,
                &StepSelection::default().include_groups([StepGroup::Backup]),
            )?;
            let artifacts = finalize(&reporter, summary.overall_status)?;
            maybe_notify(
                &loaded.config,
                &artifacts,
                summary.overall_status,
                "backup",
                options.profile.as_deref(),
                options.dry_run,
            )?;
            Ok(())
        }
        CliCommand::Run | CliCommand::Cleanup | CliCommand::Verify => {
            let preflight = match preflight::run(&loaded.config, &platform, &host, &executor) {
                Ok(summary) => summary,
                Err(err) => {
                    let _ = reporter.record_step(StepEvent {
                        step_id: "preflight".into(),
                        name: "Preflight checks".into(),
                        group: StepGroup::Verify.as_str().into(),
                        status: StepStatus::Failed,
                        index: 0,
                        total: 0,
                        duration_ms: 0,
                        detail: Some(err.to_string()),
                    });
                    let artifacts = finalize(&reporter, RunStatus::Failed)?;
                    maybe_notify(
                        &loaded.config,
                        &artifacts,
                        RunStatus::Failed,
                        "preflight",
                        options.profile.as_deref(),
                        options.dry_run,
                    )?;
                    return Err(err);
                }
            };
            reporter.record_step(StepEvent {
                step_id: "preflight".into(),
                name: "Preflight checks".into(),
                group: StepGroup::Verify.as_str().into(),
                status: if preflight.warnings.is_empty() {
                    StepStatus::Ok
                } else {
                    StepStatus::Warning
                },
                index: 0,
                total: 0,
                duration_ms: preflight.duration_ms,
                detail: (!preflight.warnings.is_empty()).then(|| preflight.warnings.join(" | ")),
            })?;

            let selection =
                build_selection(&options.command, options.profile.as_deref(), &loaded.config)?;
            let workflow_summary = workflow::run(
                &loaded.config,
                &platform,
                &host,
                &executor,
                &reporter,
                &selection,
            )?;
            let run_status = combine_status(&preflight.warnings, workflow_summary.overall_status);
            let artifacts = finalize(&reporter, run_status)?;
            maybe_notify(
                &loaded.config,
                &artifacts,
                run_status,
                options.command.label(),
                options.profile.as_deref(),
                options.dry_run,
            )?;
            Ok(())
        }
        CliCommand::ReportShow { .. }
        | CliCommand::ReportDiff { .. }
        | CliCommand::ReportList { .. }
        | CliCommand::SchedulePrintSystemd { .. } => unreachable!("handled earlier"),
    }
}

fn execute_report_show(options: &CliOptions, selector: Option<&str>) -> AppResult<()> {
    let loaded = load_config(options.config_path.as_deref())?;
    report_cmd::show(&loaded.config.report.dir, selector)
}

fn execute_report_diff(
    options: &CliOptions,
    left: Option<&str>,
    right: Option<&str>,
) -> AppResult<()> {
    let loaded = load_config(options.config_path.as_deref())?;
    report_cmd::diff(&loaded.config.report.dir, left, right)
}

fn execute_report_list(options: &CliOptions, limit: usize) -> AppResult<()> {
    let loaded = load_config(options.config_path.as_deref())?;
    report_cmd::list(&loaded.config.report.dir, limit)
}

fn execute_schedule_print(options: &CliOptions, job: &str) -> AppResult<()> {
    let mut loaded = load_config(options.config_path.as_deref())?;
    if let Some(profile) = &options.profile {
        loaded
            .config
            .apply_profile(profile)
            .map_err(|message| DomainError::validation("DOMAIN_PROFILE_UNKNOWN", message))?;
    }
    let executable = schedule::current_executable()?;
    schedule::print_systemd_templates(
        &executable,
        loaded.source.as_deref(),
        options.profile.as_deref(),
        job,
        &loaded.config.scheduling,
    )
}

fn build_selection(
    command: &CliCommand,
    profile: Option<&str>,
    config: &Config,
) -> AppResult<StepSelection> {
    let base_groups = match command {
        CliCommand::Run => vec![
            StepGroup::Backup,
            StepGroup::SystemPackages,
            StepGroup::Dgx,
            StepGroup::Services,
            StepGroup::Tooling,
            StepGroup::Cleanup,
            StepGroup::Verify,
            StepGroup::Reboot,
        ],
        CliCommand::Cleanup => vec![StepGroup::Backup, StepGroup::Cleanup],
        CliCommand::Verify => vec![StepGroup::Verify],
        CliCommand::Backup => vec![StepGroup::Backup],
        _ => Vec::new(),
    };

    let mut allowed: BTreeSet<StepGroup> = base_groups.into_iter().collect();
    let mut selection = StepSelection::default();

    if let Some(profile_name) = profile {
        let profile_config = config
            .builtin_profiles()
            .remove(profile_name)
            .or_else(|| config.profiles.get(profile_name).cloned())
            .unwrap_or_default();

        if !profile_config.include_groups.is_empty() {
            let profile_groups = parse_step_groups(&profile_config.include_groups)?;
            allowed = allowed.intersection(&profile_groups).copied().collect();
        }
        if !profile_config.exclude_groups.is_empty() {
            let excluded = parse_step_groups(&profile_config.exclude_groups)?;
            allowed = allowed.difference(&excluded).copied().collect();
        }
        if !profile_config.skip_steps.is_empty() {
            selection = selection.exclude_steps(profile_config.skip_steps);
        }
    }

    Ok(selection.include_groups(allowed))
}

fn parse_step_groups(values: &[String]) -> AppResult<BTreeSet<StepGroup>> {
    let mut groups = BTreeSet::new();
    for value in values {
        let group = match value.as_str() {
            "backup" => StepGroup::Backup,
            "system-packages" | "system_packages" | "packages" | "apt" => StepGroup::SystemPackages,
            "dgx" => StepGroup::Dgx,
            "services" => StepGroup::Services,
            "tooling" => StepGroup::Tooling,
            "cleanup" => StepGroup::Cleanup,
            "verify" => StepGroup::Verify,
            "reboot" => StepGroup::Reboot,
            other => {
                return Err(DomainError::validation(
                    "DOMAIN_PROFILE_GROUP_INVALID",
                    format!("unknown step group in profile: {other}"),
                )
                .into());
            }
        };
        groups.insert(group);
    }
    Ok(groups)
}

fn combine_status(preflight_warnings: &[String], workflow_status: RunStatus) -> RunStatus {
    if matches!(workflow_status, RunStatus::Failed | RunStatus::Partial) {
        workflow_status
    } else if !preflight_warnings.is_empty() {
        RunStatus::Warning
    } else {
        workflow_status
    }
}

fn maybe_notify(
    config: &Config,
    artifacts: &crate::features::system_updater::domain::report::ReportArtifacts,
    status: RunStatus,
    subcommand: &str,
    profile: Option<&str>,
    dry_run: bool,
) -> AppResult<()> {
    if dry_run {
        return Ok(());
    }

    let hook = match status {
        RunStatus::Ok | RunStatus::ScanOnly => config.notifications.on_success_command.as_deref(),
        RunStatus::Warning | RunStatus::Partial => {
            config.notifications.on_warning_command.as_deref()
        }
        RunStatus::Failed => config.notifications.on_failure_command.as_deref(),
    };
    let Some(hook) = hook else {
        return Ok(());
    };

    let executor = ShellCommandExecutor::new(false, NoopObserver);
    executor.run(
        &crate::features::system_updater::domain::command::CommandSpec::new("bash", ["-lc", hook])
            .with_timeout_secs(30)
            .with_env("UPDATE_RUN_ID", artifacts.run_id.clone())
            .with_env("UPDATE_STATUS", status.as_str())
            .with_env(
                "UPDATE_REPORT_PATH",
                artifacts.report_path.display().to_string(),
            )
            .with_env("UPDATE_LOG_PATH", artifacts.log_path.display().to_string())
            .with_env("UPDATE_SUBCOMMAND", subcommand.to_string())
            .with_env("UPDATE_PROFILE", profile.unwrap_or_default().to_string()),
    )?;
    Ok(())
}

fn best_effort_finalize(reporter: &impl RunReporter, status: RunStatus) {
    let _ = reporter.finalize(status);
}

fn finalize(
    reporter: &impl RunReporter,
    status: RunStatus,
) -> AppResult<crate::features::system_updater::domain::report::ReportArtifacts> {
    let artifacts = reporter.finalize(status)?;
    println!();
    println!("  Run ID:  {}", artifacts.run_id);
    println!("  Report:  {}", artifacts.report_path.display());
    println!("  Log:     {}", artifacts.log_path.display());
    println!("  Artifacts: {}", artifacts.artifact_dir.display());
    Ok(artifacts)
}

fn print_runtime_banner(
    reporter: &impl RunReporter,
    config_path: &Option<PathBuf>,
    host: &impl SystemProbe,
    profile: &Option<String>,
    platform: &PlatformInfo,
) -> Result<(), InfrastructureError> {
    println!();
    println!("  Run ID: {}", reporter.run_id());
    match config_path {
        Some(path) => {
            println!("  Config: {}", path.display());
            reporter.note(&format!("config source: {}", path.display()))?;
        }
        None => {
            println!("  Config: built-in defaults");
            reporter.note("config source: built-in defaults")?;
        }
    }
    if let Some(profile) = profile {
        println!("  Profile: {profile}");
        reporter.note(&format!("profile: {profile}"))?;
    }

    let hostname = host.hostname().unwrap_or_else(|_| "unknown".into());
    println!("  Host:   {hostname}");
    println!("  Platform: {platform}", platform = platform.summary());
    reporter.note(&format!("host: {hostname}"))?;
    reporter.note(&format!("platform: {}", platform.detection_note()))?;
    reporter.note(&format!("run id: {}", reporter.run_id()))?;
    Ok(())
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> AppResult<CliOptions> {
    let mut dry_run = false;
    let mut config_path = None;
    let mut profile = None;
    let mut positionals = Vec::new();
    let mut iter = args.into_iter();

    while let Some(argument) = iter.next() {
        match argument.as_str() {
            "--dry-run" | "-n" => dry_run = true,
            "--scan" => positionals.push("scan".to_string()),
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--config" => {
                let value = iter.next().ok_or_else(|| {
                    DomainError::validation("DOMAIN_CLI_CONFIG_PATH", "--config requires a path")
                })?;
                config_path = Some(PathBuf::from(value));
            }
            "--profile" => {
                let value = iter.next().ok_or_else(|| {
                    DomainError::validation("DOMAIN_CLI_PROFILE", "--profile requires a value")
                })?;
                profile = Some(value);
            }
            _ if argument.starts_with("--config=") => {
                let value = argument.trim_start_matches("--config=");
                if value.is_empty() {
                    return Err(DomainError::validation(
                        "DOMAIN_CLI_CONFIG_PATH",
                        "--config requires a path",
                    )
                    .into());
                }
                config_path = Some(PathBuf::from(value));
            }
            _ if argument.starts_with("--profile=") => {
                let value = argument.trim_start_matches("--profile=");
                if value.is_empty() {
                    return Err(DomainError::validation(
                        "DOMAIN_CLI_PROFILE",
                        "--profile requires a value",
                    )
                    .into());
                }
                profile = Some(value.to_string());
            }
            _ if argument.starts_with('-') => {
                return Err(DomainError::validation(
                    "DOMAIN_CLI_UNKNOWN_OPTION",
                    format!("unknown option: {argument}"),
                )
                .into());
            }
            _ => positionals.push(argument),
        }
    }

    let command = parse_command(positionals)?;
    Ok(CliOptions {
        command,
        dry_run,
        config_path,
        profile,
    })
}

fn parse_command(positionals: Vec<String>) -> AppResult<CliCommand> {
    if positionals.is_empty() {
        return Ok(CliCommand::Run);
    }

    let mut iter = positionals.into_iter();
    let first = iter.next().expect("positionals not empty");
    let command = match first.as_str() {
        "run" => CliCommand::Run,
        "scan" => CliCommand::Scan,
        "cleanup" => CliCommand::Cleanup,
        "verify" => CliCommand::Verify,
        "backup" => CliCommand::Backup,
        "report" => match iter.next().as_deref() {
            Some("show") | None => CliCommand::ReportShow {
                selector: iter.next(),
            },
            Some("diff") => CliCommand::ReportDiff {
                left: iter.next(),
                right: iter.next(),
            },
            Some("list") => {
                let limit = iter
                    .next()
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(10);
                CliCommand::ReportList { limit }
            }
            Some(other) => {
                return Err(DomainError::validation(
                    "DOMAIN_CLI_REPORT_SUBCOMMAND",
                    format!("unknown report subcommand: {other}"),
                )
                .into());
            }
        },
        "schedule" => match iter.next().as_deref() {
            Some("print-systemd") | None => CliCommand::SchedulePrintSystemd {
                job: iter.next().unwrap_or_else(|| "run".into()),
            },
            Some(other) => {
                return Err(DomainError::validation(
                    "DOMAIN_CLI_SCHEDULE_SUBCOMMAND",
                    format!("unknown schedule subcommand: {other}"),
                )
                .into());
            }
        },
        other => {
            return Err(DomainError::validation(
                "DOMAIN_CLI_UNKNOWN_COMMAND",
                format!("unknown command: {other}"),
            )
            .into());
        }
    };

    Ok(command)
}

pub fn print_usage() {
    println!("System Update & Scanner");
    println!();
    println!("Usage: update [GLOBAL OPTIONS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  run                    Full maintenance workflow (default)");
    println!("  scan                   Read-only system scan");
    println!("  cleanup                Backup + cleanup groups only");
    println!("  verify                 Verification groups only");
    println!("  backup                 Create maintenance snapshot only");
    println!("  report show [RUN]      Show latest or selected report");
    println!("  report diff [A] [B]    Diff two runs, or latest two by default");
    println!("  report list [N]        List latest N reports");
    println!("  schedule print-systemd [JOB]");
    println!();
    println!("Global Options:");
    println!("  --dry-run, -n          Preview mutating commands without executing");
    println!("  --config PATH          Load config from PATH");
    println!("  --profile NAME         Apply built-in or configured profile");
    println!("  --help, -h             Show this help");
}

struct NoopObserver;

impl CommandObserver for NoopObserver {
    fn record_command(
        &self,
        _event: crate::features::system_updater::domain::report::CommandEvent,
    ) -> Result<(), InfrastructureError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CliCommand, parse_args, parse_step_groups};
    use crate::features::system_updater::domain::report::StepGroup;

    #[test]
    fn parses_known_flags() {
        let options = parse_args(vec![
            "cleanup".to_string(),
            "--dry-run".to_string(),
            "--profile=safe".to_string(),
            "--config=custom.toml".to_string(),
        ])
        .expect("parse args");

        assert!(matches!(options.command, CliCommand::Cleanup));
        assert!(options.dry_run);
        assert_eq!(options.profile.as_deref(), Some("safe"));
        assert_eq!(
            options.config_path.expect("config"),
            std::path::PathBuf::from("custom.toml")
        );
    }

    #[test]
    fn parses_report_subcommands() {
        let options = parse_args(vec![
            "report".to_string(),
            "diff".to_string(),
            "run-a".to_string(),
            "run-b".to_string(),
        ])
        .expect("parse args");

        assert!(matches!(
            options.command,
            CliCommand::ReportDiff {
                left: Some(_),
                right: Some(_)
            }
        ));
    }

    #[test]
    fn rejects_unknown_option() {
        let err = parse_args(vec!["--wat".to_string()])
            .unwrap_err()
            .to_string();
        assert!(err.contains("DOMAIN_CLI_UNKNOWN_OPTION"));
    }

    #[test]
    fn accepts_legacy_and_new_system_package_group_names() {
        let groups = parse_step_groups(&[
            "apt".to_string(),
            "system-packages".to_string(),
            "packages".to_string(),
        ])
        .expect("parse groups");

        assert_eq!(groups.len(), 1);
        assert!(groups.contains(&StepGroup::SystemPackages));
    }
}
