use crate::features::system_updater::application::maintenance;
use crate::features::system_updater::domain::config::Config;
use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::platform::PlatformInfo;
use crate::features::system_updater::domain::report::{
    RunStatus, StepEvent, StepGroup, StepOutcome, StepStatus,
};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

pub struct WorkflowSummary {
    pub counts: BTreeMap<StepStatus, usize>,
    pub overall_status: RunStatus,
    pub executed_steps: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlatformSupport {
    #[default]
    Any,
    Linux,
    Macos,
}

impl PlatformSupport {
    fn skip_reason(self, platform: &PlatformInfo) -> Option<String> {
        match self {
            Self::Any => None,
            Self::Linux if !platform.is_linux() => {
                Some(format!("requires Linux; detected {}", platform.summary()))
            }
            Self::Macos if !platform.is_macos() => {
                Some(format!("requires macOS; detected {}", platform.summary()))
            }
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct StepSelection {
    include_groups: Option<BTreeSet<StepGroup>>,
    exclude_groups: BTreeSet<StepGroup>,
    include_steps: Option<BTreeSet<String>>,
    exclude_steps: BTreeSet<String>,
}

impl StepSelection {
    pub fn include_groups(mut self, groups: impl IntoIterator<Item = StepGroup>) -> Self {
        self.include_groups = Some(groups.into_iter().collect());
        self
    }

    pub fn exclude_groups(mut self, groups: impl IntoIterator<Item = StepGroup>) -> Self {
        self.exclude_groups.extend(groups);
        self
    }

    pub fn include_steps(mut self, steps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.include_steps = Some(steps.into_iter().map(Into::into).collect());
        self
    }

    pub fn exclude_steps(mut self, steps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.exclude_steps.extend(steps.into_iter().map(Into::into));
        self
    }

    pub fn allows<H, E, R>(&self, step: &StepDefinition<H, E, R>) -> bool
    where
        H: HostServices,
        E: CommandExecutor,
        R: RunReporter,
    {
        if self.exclude_groups.contains(&step.group)
            || selection_matches_step_id(&self.exclude_steps, step.id)
        {
            return false;
        }
        if let Some(include_groups) = &self.include_groups
            && !include_groups.contains(&step.group)
        {
            return false;
        }
        if let Some(include_steps) = &self.include_steps
            && !selection_matches_step_id(include_steps, step.id)
        {
            return false;
        }
        true
    }
}

fn selection_matches_step_id(selection: &BTreeSet<String>, step_id: &str) -> bool {
    selection.contains(step_id)
        || legacy_step_ids(step_id)
            .iter()
            .any(|legacy_id| selection.contains(*legacy_id))
}

fn legacy_step_ids(step_id: &str) -> &'static [&'static str] {
    match step_id {
        "system-packages.apt-upgrade" => &["apt.update-upgrade"],
        "system-packages.homebrew-upgrade" => &["brew.update-upgrade"],
        "system-packages.macos-software-update" => &["macos.software-update"],
        "system-packages.apt-maintenance-tools" => &["apt.maintenance-tools"],
        "cleanup.apt-residual-configs" => &["apt.rc-configs"],
        "cleanup.apt-old-kernels" => &["apt.old-kernels"],
        "cleanup.apt-autoremove" => &["apt.cleanup"],
        _ => &[],
    }
}

pub fn run<H, E, R>(
    config: &Config,
    platform: &PlatformInfo,
    host: &H,
    executor: &E,
    reporter: &R,
    selection: &StepSelection,
) -> AppResult<WorkflowSummary>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    println!();
    println!("  Starting System Update");
    println!("  ======================");

    let context = maintenance::MaintenanceContext {
        config,
        platform,
        host,
        executor,
        reporter,
    };
    let steps: Vec<_> = maintenance::plan()
        .into_iter()
        .filter(|step| selection.allows(step))
        .collect();
    let total = steps.len();
    let mut counts = BTreeMap::new();

    for (index, step) in steps.iter().enumerate() {
        println!();
        println!("==> [{}/{}] {}", index + 1, total, step.name);
        let started = Instant::now();
        reporter.activate_step(Some(step.id))?;
        let outcome = match step.support.skip_reason(platform) {
            Some(reason) => {
                println!("  Skipping: {reason}");
                StepOutcome::skipped(reason)
            }
            None => match (step.run)(&context) {
                Ok(outcome) => outcome,
                Err(err) => StepOutcome::new(StepStatus::Failed, Some(err.to_string())),
            },
        };
        reporter.activate_step(None)?;

        *counts.entry(outcome.status).or_insert(0) += 1;
        reporter.record_step(StepEvent {
            step_id: step.id.to_string(),
            name: step.name.to_string(),
            group: step.group.as_str().to_string(),
            status: outcome.status,
            index: index + 1,
            total,
            duration_ms: started.elapsed().as_millis(),
            detail: outcome.detail.clone(),
        })?;

        if matches!(outcome.status, StepStatus::Failed | StepStatus::Blocked)
            && let Some(detail) = outcome.detail
        {
            eprintln!("  !! {}: {}", step.name, detail);
        }
    }

    let executed_steps = total;
    let overall_status = overall_status(&counts);
    print_summary(&counts, executed_steps);
    print_manual_steps(platform);

    Ok(WorkflowSummary {
        counts,
        overall_status,
        executed_steps,
    })
}

fn overall_status(counts: &BTreeMap<StepStatus, usize>) -> RunStatus {
    if counts.contains_key(&StepStatus::Failed) || counts.contains_key(&StepStatus::Blocked) {
        RunStatus::Partial
    } else if counts.contains_key(&StepStatus::Warning) || counts.contains_key(&StepStatus::Partial)
    {
        RunStatus::Warning
    } else {
        RunStatus::Ok
    }
}

fn print_summary(counts: &BTreeMap<StepStatus, usize>, total: usize) {
    println!();
    if counts.get(&StepStatus::Failed).copied().unwrap_or_default() == 0
        && counts
            .get(&StepStatus::Blocked)
            .copied()
            .unwrap_or_default()
            == 0
        && counts
            .get(&StepStatus::Warning)
            .copied()
            .unwrap_or_default()
            == 0
        && counts
            .get(&StepStatus::Partial)
            .copied()
            .unwrap_or_default()
            == 0
    {
        println!("  All {total} selected steps completed without warnings.");
    } else {
        println!("  Completed {total} selected steps with status summary:");
        for (status, count) in counts {
            println!("    {:<10} {}", status.as_str(), count);
        }
    }
}

fn print_manual_steps(platform: &PlatformInfo) {
    println!();
    println!("-- Manual Steps ----------------------------------------");
    println!();
    if platform.is_macos() {
        println!("  App Store apps and major macOS installer flows remain manual.");
        println!("  Review manually if needed:");
        println!();
        println!("    softwareupdate --list-full-installers");
    } else {
        println!("  Firmware updates require interactive confirmation.");
        println!("  Run manually:");
        println!();
        println!("    sudo fwupdmgr refresh");
        println!("    sudo fwupdmgr upgrade");
    }
    println!();
}

#[derive(Clone, Copy)]
pub struct StepDefinition<H, E, R>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    pub id: &'static str,
    pub name: &'static str,
    pub group: StepGroup,
    pub support: PlatformSupport,
    pub run: fn(&maintenance::MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>,
}

#[cfg(test)]
mod tests {
    use super::{PlatformSupport, StepDefinition, StepSelection, run};
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::domain::report::{StepGroup, StepOutcome, StepStatus};
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};

    #[test]
    fn selection_filters_groups() {
        let selection = StepSelection::default().include_groups([StepGroup::Cleanup]);
        let step = StepDefinition::<FakeHost, FakeExecutor, FakeReporter> {
            id: "cleanup.test",
            name: "Cleanup",
            group: StepGroup::Cleanup,
            support: PlatformSupport::Any,
            run: |_context| Ok(StepOutcome::ok()),
        };

        assert!(selection.allows(&step));
    }

    #[test]
    fn workflow_runs_selected_backup_step() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let config = Config::default();
        let selection = StepSelection::default().include_groups([StepGroup::Backup]);

        let platform = PlatformInfo::default();
        let summary =
            run(&config, &platform, &host, &executor, &reporter, &selection).expect("workflow");
        assert_eq!(summary.executed_steps, 1);
        assert_eq!(reporter.steps()[0].step_id, "backup.snapshot");
    }

    #[test]
    fn workflow_marks_gb10_steps_as_skipped_on_generic_linux() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let config = Config::default();
        let platform = PlatformInfo::generic_linux(Some("Generic VM".into()), None, None);
        let selection = StepSelection::default().include_groups([StepGroup::Dgx]);

        let summary =
            run(&config, &platform, &host, &executor, &reporter, &selection).expect("workflow");

        assert_eq!(summary.executed_steps, 2);
        assert_eq!(summary.counts.get(&StepStatus::Skipped).copied(), Some(2));
        assert!(
            reporter
                .steps()
                .iter()
                .all(|step| step.status == StepStatus::Skipped)
        );
    }

    #[test]
    fn selection_accepts_legacy_step_id_aliases() {
        let selection = StepSelection::default().exclude_steps(["apt.update-upgrade"]);
        let step = StepDefinition::<FakeHost, FakeExecutor, FakeReporter> {
            id: "system-packages.apt-upgrade",
            name: "APT update & full-upgrade",
            group: StepGroup::SystemPackages,
            support: PlatformSupport::Linux,
            run: |_context| Ok(StepOutcome::ok()),
        };

        assert!(!selection.allows(&step));
    }
}
