mod apt;
mod backup;
mod brew;
mod cuda;
mod dgx;
mod macos;
mod services;
mod system;
mod tools;

use crate::features::system_updater::application::workflow::{PlatformSupport, StepDefinition};
use crate::features::system_updater::domain::config::Config;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::platform::PlatformInfo;
use crate::features::system_updater::domain::report::{StepGroup, StepOutcome, StepStatus};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};

pub struct MaintenanceContext<'a, H, E, R>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    pub config: &'a Config,
    pub platform: &'a PlatformInfo,
    pub host: &'a H,
    pub executor: &'a E,
    pub reporter: &'a R,
}

pub fn plan<H, E, R>() -> Vec<StepDefinition<H, E, R>>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    vec![
        StepDefinition {
            id: "backup.snapshot",
            name: "Create maintenance snapshot",
            group: StepGroup::Backup,
            support: PlatformSupport::Any,
            run: backup::create_snapshot::<H, E, R>,
        },
        StepDefinition {
            id: "system-packages.apt-upgrade",
            name: "APT update & full-upgrade",
            group: StepGroup::SystemPackages,
            support: PlatformSupport::Linux,
            run: apt::update_and_upgrade::<H, E, R>,
        },
        StepDefinition {
            id: "system-packages.homebrew-upgrade",
            name: "Homebrew update & upgrade",
            group: StepGroup::SystemPackages,
            support: PlatformSupport::Macos,
            run: brew::update_homebrew::<H, E, R>,
        },
        StepDefinition {
            id: "system-packages.macos-software-update",
            name: "macOS softwareupdate",
            group: StepGroup::SystemPackages,
            support: PlatformSupport::Macos,
            run: macos::run_software_update::<H, E, R>,
        },
        StepDefinition {
            id: "dgx.kernel-driver",
            name: "GB10 kernel & driver",
            group: StepGroup::Dgx,
            support: PlatformSupport::Linux,
            run: dgx::install_kernel_driver::<H, E, R>,
        },
        StepDefinition {
            id: "dgx.cuda-shell",
            name: "CUDA Toolkit + .zshrc",
            group: StepGroup::Dgx,
            support: PlatformSupport::Linux,
            run: cuda::upgrade_toolkit_and_configure::<H, E, R>,
        },
        StepDefinition {
            id: "system-packages.apt-maintenance-tools",
            name: "System maintenance tools",
            group: StepGroup::SystemPackages,
            support: PlatformSupport::Linux,
            run: apt::install_maintenance_tools::<H, E, R>,
        },
        StepDefinition {
            id: "cleanup.apt-residual-configs",
            name: "Clean rc residual configs",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Linux,
            run: apt::clean_rc_packages::<H, E, R>,
        },
        StepDefinition {
            id: "cleanup.apt-old-kernels",
            name: "Purge old kernels",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Linux,
            run: apt::purge_old_kernels::<H, E, R>,
        },
        StepDefinition {
            id: "services.snap",
            name: "Snap packages",
            group: StepGroup::Services,
            support: PlatformSupport::Linux,
            run: services::update_snap::<H, E, R>,
        },
        StepDefinition {
            id: "services.flatpak",
            name: "Flatpak packages",
            group: StepGroup::Services,
            support: PlatformSupport::Any,
            run: services::update_flatpak::<H, E, R>,
        },
        StepDefinition {
            id: "services.docker-compose",
            name: "Docker / Compose",
            group: StepGroup::Services,
            support: PlatformSupport::Any,
            run: services::update_docker::<H, E, R>,
        },
        StepDefinition {
            id: "tools.node",
            name: "nvm / Node / npm",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_nvm_node::<H, E, R>,
        },
        StepDefinition {
            id: "tools.bun",
            name: "Bun",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_bun::<H, E, R>,
        },
        StepDefinition {
            id: "tools.deno",
            name: "Deno",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_deno::<H, E, R>,
        },
        StepDefinition {
            id: "tools.pipx",
            name: "pipx",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_pipx::<H, E, R>,
        },
        StepDefinition {
            id: "tools.conda",
            name: "Conda",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_conda::<H, E, R>,
        },
        StepDefinition {
            id: "tools.pnpm",
            name: "pnpm",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_pnpm::<H, E, R>,
        },
        StepDefinition {
            id: "tools.rust",
            name: "Rust / Cargo",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_rust::<H, E, R>,
        },
        StepDefinition {
            id: "tools.uv",
            name: "uv / Python",
            group: StepGroup::Tooling,
            support: PlatformSupport::Any,
            run: tools::update_uv::<H, E, R>,
        },
        StepDefinition {
            id: "cleanup.apt-autoremove",
            name: "APT cleanup / autoremove --purge",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Linux,
            run: apt::autoremove_purge_and_clean::<H, E, R>,
        },
        StepDefinition {
            id: "brew.cleanup",
            name: "Homebrew cleanup / autoremove",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Macos,
            run: brew::cleanup_homebrew::<H, E, R>,
        },
        StepDefinition {
            id: "services.docker-cleanup",
            name: "Docker cleanup",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Any,
            run: services::cleanup_docker::<H, E, R>,
        },
        StepDefinition {
            id: "tools.cache-cleanup",
            name: "Tool / cache cleanup",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Any,
            run: tools::cleanup_caches::<H, E, R>,
        },
        StepDefinition {
            id: "system.local-cleanup",
            name: "Journal / crash cleanup",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Linux,
            run: system::cleanup_local_artifacts::<H, E, R>,
        },
        StepDefinition {
            id: "services.needrestart",
            name: "needrestart",
            group: StepGroup::Verify,
            support: PlatformSupport::Linux,
            run: services::run_needrestart::<H, E, R>,
        },
        StepDefinition {
            id: "system.postflight",
            name: "Postflight verification",
            group: StepGroup::Verify,
            support: PlatformSupport::Any,
            run: system::postflight_verify::<H, E, R>,
        },
        StepDefinition {
            id: "dgx.watchdog",
            name: "GB10 watchdog verification",
            group: StepGroup::Verify,
            support: PlatformSupport::Linux,
            run: dgx::verify_watchdog::<H, E, R>,
        },
        StepDefinition {
            id: "system.reboot",
            name: "Reboot decision",
            group: StepGroup::Reboot,
            support: PlatformSupport::Linux,
            run: system::check_reboot::<H, E, R>,
        },
    ]
}

pub struct WarningCollector {
    warnings: Vec<String>,
}

impl WarningCollector {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        let message = message.into();
        eprintln!("  !! {message}");
        self.warnings.push(message);
    }

    pub fn capture<T, E>(&mut self, context: impl Into<String>, result: Result<T, E>) -> Option<T>
    where
        crate::features::system_updater::domain::error::ApplicationError: From<E>,
    {
        match result {
            Ok(value) => Some(value),
            Err(err) => {
                let err =
                    crate::features::system_updater::domain::error::ApplicationError::from(err);
                self.warn(format!("{}: {}", context.into(), err));
                None
            }
        }
    }

    pub fn finish(self) -> StepOutcome {
        if self.warnings.is_empty() {
            StepOutcome::ok()
        } else {
            StepOutcome::warning(format!(
                "{} issue(s): {}",
                self.warnings.len(),
                self.warnings.join(" | ")
            ))
        }
    }

    pub fn finish_as(self, status: StepStatus) -> StepOutcome {
        if self.warnings.is_empty() {
            StepOutcome::ok()
        } else {
            StepOutcome::new(
                status,
                Some(format!(
                    "{} issue(s): {}",
                    self.warnings.len(),
                    self.warnings.join(" | ")
                )),
            )
        }
    }
}

impl Default for WarningCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn step_ok() -> AppResult<StepOutcome> {
    Ok(StepOutcome::ok())
}
