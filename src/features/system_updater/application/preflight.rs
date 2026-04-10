use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::config::Config;
use crate::features::system_updater::domain::error::{AppResult, DomainError, InfrastructureError};
use crate::features::system_updater::domain::platform::PlatformInfo;
use crate::features::system_updater::ports::{CommandExecutor, HostServices};
use std::path::Path;
use std::time::Instant;

#[derive(Debug)]
pub struct PreflightSummary {
    pub warnings: Vec<String>,
    pub duration_ms: u128,
}

pub fn run<H, E>(
    config: &Config,
    platform: &PlatformInfo,
    host: &H,
    executor: &E,
) -> AppResult<PreflightSummary>
where
    H: HostServices,
    E: CommandExecutor,
{
    let started = Instant::now();
    let mut warnings = Vec::new();

    println!();
    println!("  Running preflight checks...");

    if host.var("USER").unwrap_or_default() == "root" {
        let message =
            "running as root. Tools like nvm, pipx, bun, and uv are safer as a regular user.";
        eprintln!("  !! {message}");
        warnings.push(message.to_string());
    }

    enforce_disk_floor(host, Path::new("/"), config.preflight.min_root_free_gb)?;
    enforce_disk_floor(host, Path::new("/var"), config.preflight.min_var_free_gb)?;

    if config.preflight.check_network {
        record_dns_result(host, primary_registry(platform), &mut warnings);
        record_dns_result(host, "pypi.org", &mut warnings);
    }

    if executor.is_dry_run() {
        println!("  Dry-run mode: skipping sudo credential refresh.");
        return Ok(PreflightSummary {
            warnings,
            duration_ms: started.elapsed().as_millis(),
        });
    }

    println!("  Verifying sudo access...");
    executor.run(&CommandSpec::new("sudo", ["-v"]))?;

    if platform.supports_apt() {
        match executor.capture(&CommandSpec::new("fuser", ["/var/lib/dpkg/lock-frontend"])) {
            Ok(output) if !output.trim().is_empty() => {
                return Err(DomainError::safety(
                    "DOMAIN_APT_LOCK_HELD",
                    "another package manager is running (dpkg lock held)",
                )
                .into());
            }
            Ok(_) => {}
            Err(err) => match err {
                InfrastructureError::CommandFailed {
                    exit_code: Some(1), ..
                } => {}
                other => return Err(other.into()),
            },
        }
    }

    Ok(PreflightSummary {
        warnings,
        duration_ms: started.elapsed().as_millis(),
    })
}

fn primary_registry(platform: &PlatformInfo) -> &'static str {
    if platform.supports_homebrew() {
        "formulae.brew.sh"
    } else {
        "archive.ubuntu.com"
    }
}

fn enforce_disk_floor(host: &impl HostServices, path: &Path, min_gib: u64) -> AppResult<()> {
    let free_gib = host.free_space_gib(path)?;
    println!("  Free space on {}: {free_gib} GiB", path.display());
    if free_gib < min_gib {
        return Err(DomainError::safety(
            "DOMAIN_DISK_LOW",
            format!(
                "not enough free space on {}: {free_gib} GiB available, need at least {min_gib} GiB",
                path.display()
            ),
        )
        .into());
    }
    Ok(())
}

fn record_dns_result(host: &impl HostServices, host_name: &str, warnings: &mut Vec<String>) {
    match host.dns_resolves(host_name) {
        Ok(true) => println!("  DNS check OK: {host_name}"),
        Ok(false) => {
            let message = format!("DNS check failed for {host_name}: lookup returned no records");
            eprintln!("  !! {message}");
            warnings.push(message);
        }
        Err(err) => {
            let message = format!("DNS check failed for {host_name}: {err}");
            eprintln!("  !! {message}");
            warnings.push(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::error::InfrastructureError;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};

    #[test]
    fn fails_when_disk_floor_is_not_met() {
        let mut host = FakeHost::new();
        host.set_free_space("/", 1);
        host.set_free_space("/var", 10);
        host.set_env("USER", "tester");
        host.set_dns("archive.ubuntu.com", Ok(true));
        host.set_dns("pypi.org", Ok(true));

        let executor = FakeExecutor::new(false);
        let config = Config::default();

        let err = run(&config, &PlatformInfo::default(), &host, &executor)
            .unwrap_err()
            .to_string();
        assert!(err.contains("DOMAIN_DISK_LOW"));
    }

    #[test]
    fn returns_warnings_for_dns_failures_instead_of_failing() {
        let mut host = FakeHost::new();
        host.set_free_space("/", 10);
        host.set_free_space("/var", 10);
        host.set_env("USER", "tester");
        host.set_dns(
            "archive.ubuntu.com",
            Err(InfrastructureError::probe(
                "INFRA_DNS_FAILED",
                "archive.ubuntu.com",
                "timeout",
            )),
        );
        host.set_dns("pypi.org", Ok(true));

        let executor = FakeExecutor::new(true);
        let config = Config::default();

        let summary = run(&config, &PlatformInfo::default(), &host, &executor).expect("preflight");
        assert_eq!(summary.warnings.len(), 1);
        assert!(summary.warnings[0].contains("archive.ubuntu.com"));
    }

    #[test]
    fn treats_fuser_exit_code_one_as_unlocked() {
        let mut host = FakeHost::new();
        host.set_free_space("/", 10);
        host.set_free_space("/var", 10);
        host.set_env("USER", "tester");
        host.set_dns("archive.ubuntu.com", Ok(true));
        host.set_dns("pypi.org", Ok(true));

        let reporter = FakeReporter::new();
        let executor = FakeExecutor::with_reporter(false, reporter);
        executor.push_capture_error(
            "fuser /var/lib/dpkg/lock-frontend",
            InfrastructureError::command_failed(
                "INFRA_COMMAND_FAILED",
                &crate::features::system_updater::domain::command::CommandSpec::new(
                    "fuser",
                    ["/var/lib/dpkg/lock-frontend"],
                ),
                Some(1),
                "not found",
            ),
        );
        executor.push_run_ok("sudo -v");

        let summary = run(
            &Config::default(),
            &PlatformInfo::default(),
            &host,
            &executor,
        )
        .expect("preflight");
        assert!(summary.warnings.is_empty());
    }

    #[test]
    fn macos_preflight_uses_homebrew_dns_target_and_skips_dpkg_lock() {
        let mut host = FakeHost::new();
        host.set_free_space("/", 10);
        host.set_free_space("/var", 10);
        host.set_env("USER", "tester");
        host.set_dns("formulae.brew.sh", Ok(true));
        host.set_dns("pypi.org", Ok(true));

        let reporter = FakeReporter::new();
        let executor = FakeExecutor::with_reporter(false, reporter);
        executor.push_run_ok("sudo -v");

        let platform = PlatformInfo::macos(Some("Mac14,15".into()), Some("15.4".into()), None);
        let summary = run(&Config::default(), &platform, &host, &executor).expect("preflight");

        assert!(summary.warnings.is_empty());
        assert_eq!(executor.commands(), vec!["sudo -v".to_string()]);
    }
}
