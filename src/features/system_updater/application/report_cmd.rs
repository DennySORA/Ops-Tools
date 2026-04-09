use crate::features::system_updater::domain::error::AppResult;
use crate::features::system_updater::domain::report::{RunRecord, StepEvent, StepStatus};
use crate::features::system_updater::infrastructure::report_store::{
    list_run_reports, load_run_record, resolve_run_record,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn show(base_dir: &Path, selector: Option<&str>) -> AppResult<()> {
    let record = resolve_run_record(base_dir, selector)?;
    print_record(&record);
    Ok(())
}

pub fn list(base_dir: &Path, limit: usize) -> AppResult<()> {
    let reports = list_run_reports(base_dir, limit)?;
    if reports.is_empty() {
        println!("No reports found in {}", base_dir.display());
        return Ok(());
    }

    for report in reports {
        let record = load_run_record(&report)?;
        println!(
            "{}  {:<10}  {:<8}  {}",
            record.run_id,
            record.overall_status.as_deref().unwrap_or("unknown"),
            subcommand_label(&record),
            report.display()
        );
    }
    Ok(())
}

pub fn diff(base_dir: &Path, left: Option<&str>, right: Option<&str>) -> AppResult<()> {
    let (left_record, right_record) = match (left, right) {
        (Some(left), Some(right)) => (
            resolve_run_record(base_dir, Some(left))?,
            resolve_run_record(base_dir, Some(right))?,
        ),
        _ => {
            let reports = list_run_reports(base_dir, 2)?;
            if reports.len() < 2 {
                println!("Need at least two reports to diff.");
                return Ok(());
            }
            (load_run_record(&reports[1])?, load_run_record(&reports[0])?)
        }
    };

    println!("Left:  {}", left_record.run_id);
    println!("Right: {}", right_record.run_id);
    println!(
        "Overall: {} -> {}",
        left_record.overall_status.as_deref().unwrap_or("unknown"),
        right_record.overall_status.as_deref().unwrap_or("unknown")
    );
    println!(
        "Commands: {} -> {}",
        left_record.commands.len(),
        right_record.commands.len()
    );

    let left_steps = build_step_map(&left_record);
    let right_steps = build_step_map(&right_record);

    println!();
    println!("Changed steps:");
    let mut any_change = false;
    for step_id in merged_step_ids(&left_steps, &right_steps) {
        let left_step = left_steps.get(&step_id);
        let right_step = right_steps.get(&step_id);
        if let Some(change_line) =
            describe_step_change(&step_id, left_step.copied(), right_step.copied())
        {
            any_change = true;
            println!("{change_line}");
        }
    }
    if !any_change {
        println!("  no step status changes");
    }

    Ok(())
}

fn print_record(record: &RunRecord) {
    println!("Run ID:     {}", record.run_id);
    println!(
        "Status:     {}",
        record.overall_status.as_deref().unwrap_or("unknown")
    );
    println!("Subcommand: {}", subcommand_label(record));
    if let Some(profile) = &record.context.profile {
        println!("Profile:    {profile}");
    }
    println!("Report:     {}", record.report_path.display());
    println!("Log:        {}", record.log_path.display());
    println!("Commands:   {}", record.commands.len());
    println!("Steps:      {}", record.steps.len());

    let mut counts = BTreeMap::new();
    for step in &record.steps {
        *counts.entry(step.status).or_insert(0usize) += 1;
    }

    println!();
    println!("Step Summary:");
    for status in [
        StepStatus::Ok,
        StepStatus::Warning,
        StepStatus::Partial,
        StepStatus::Failed,
        StepStatus::Blocked,
        StepStatus::Skipped,
        StepStatus::DryRun,
    ] {
        if let Some(count) = counts.get(&status) {
            println!("  {:<10} {}", status.as_str(), count);
        }
    }

    let warnings: Vec<_> = record
        .steps
        .iter()
        .filter(|step| {
            !matches!(
                step.status,
                StepStatus::Ok | StepStatus::Skipped | StepStatus::DryRun
            )
        })
        .collect();
    if !warnings.is_empty() {
        println!();
        println!("Attention:");
        for step in warnings {
            println!(
                "  {} [{}] {}",
                step_label(step, None),
                step.status.as_str(),
                step.detail.as_deref().unwrap_or("no detail")
            );
        }
    }
}

fn build_step_map(record: &RunRecord) -> BTreeMap<String, &StepEvent> {
    let mut steps = BTreeMap::new();
    for (index, step) in record.steps.iter().enumerate() {
        steps.entry(step_identity(step, index)).or_insert(step);
    }
    steps
}

fn merged_step_ids(
    left_steps: &BTreeMap<String, &StepEvent>,
    right_steps: &BTreeMap<String, &StepEvent>,
) -> BTreeSet<String> {
    left_steps
        .keys()
        .chain(right_steps.keys())
        .cloned()
        .collect()
}

fn describe_step_change(
    step_id: &str,
    left_step: Option<&StepEvent>,
    right_step: Option<&StepEvent>,
) -> Option<String> {
    let left_status = left_step.map(|step| step.status);
    let right_status = right_step.map(|step| step.status);
    if left_status != right_status {
        return Some(format!(
            "  {:<24} {} -> {}",
            step_id,
            left_step
                .map(|step| step.status.as_str())
                .unwrap_or("missing"),
            right_step
                .map(|step| step.status.as_str())
                .unwrap_or("missing")
        ));
    }

    let left_detail = left_step.and_then(|step| step.detail.as_deref());
    let right_detail = right_step.and_then(|step| step.detail.as_deref());
    if left_detail != right_detail {
        return Some(format!("  {:<24} detail changed", step_id));
    }

    None
}

fn subcommand_label(record: &RunRecord) -> &str {
    if record.context.subcommand.trim().is_empty() {
        "legacy"
    } else {
        record.context.subcommand.as_str()
    }
}

fn step_identity(step: &StepEvent, index: usize) -> String {
    if !step.step_id.trim().is_empty() {
        step.step_id.clone()
    } else if !step.name.trim().is_empty() {
        step.name.clone()
    } else {
        format!("step-{index}")
    }
}

fn step_label(step: &StepEvent, index: Option<usize>) -> String {
    if !step.step_id.trim().is_empty() {
        step.step_id.clone()
    } else if !step.name.trim().is_empty() {
        step.name.clone()
    } else {
        format!("step-{}", index.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_step_map, describe_step_change, merged_step_ids, step_identity, subcommand_label,
    };
    use crate::features::system_updater::domain::report::{
        RunContext, RunRecord, StepEvent, StepStatus,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn list_uses_legacy_label_when_subcommand_missing() {
        let mut record = RunRecord {
            context: RunContext::default(),
            ..RunRecord::default()
        };
        assert_eq!(subcommand_label(&record), "legacy");

        record.context.subcommand = "cleanup".into();
        assert_eq!(subcommand_label(&record), "cleanup");
    }

    #[test]
    fn step_identity_falls_back_to_name_and_index() {
        let named = StepEvent {
            name: "APT cleanup".into(),
            ..StepEvent::default()
        };
        assert_eq!(step_identity(&named, 3), "APT cleanup");

        let unnamed = StepEvent::default();
        assert_eq!(step_identity(&unnamed, 7), "step-7");
    }

    #[test]
    fn build_step_map_preserves_legacy_named_steps() {
        let mut record = RunRecord {
            report_path: PathBuf::from("/tmp/report.json"),
            log_path: PathBuf::from("/tmp/session.log"),
            artifact_dir: PathBuf::from("/tmp"),
            ..RunRecord::default()
        };
        record.steps = vec![
            StepEvent {
                step_id: String::new(),
                name: "APT cleanup".into(),
                status: StepStatus::Warning,
                ..StepEvent::default()
            },
            StepEvent {
                step_id: "verify-postflight".into(),
                name: "Postflight verification".into(),
                status: StepStatus::Ok,
                ..StepEvent::default()
            },
        ];

        let map = build_step_map(&record);
        assert!(map.contains_key("APT cleanup"));
        assert!(map.contains_key("verify-postflight"));
    }

    #[test]
    fn merged_step_ids_deduplicates_shared_keys() {
        let shared = StepEvent::default();
        let duplicate = StepEvent::default();
        let postflight = StepEvent::default();

        let mut left = BTreeMap::new();
        left.insert("backup.snapshot".to_string(), &shared);

        let mut right = BTreeMap::new();
        right.insert("backup.snapshot".to_string(), &duplicate);
        right.insert("system.postflight".to_string(), &postflight);

        let ids = merged_step_ids(&left, &right);
        let collected: Vec<_> = ids.into_iter().collect();
        assert_eq!(collected, vec!["backup.snapshot", "system.postflight"]);
    }

    #[test]
    fn describe_step_change_marks_detail_only_changes() {
        let left = StepEvent {
            status: StepStatus::DryRun,
            detail: Some("snapshot path A".into()),
            ..StepEvent::default()
        };
        let right = StepEvent {
            status: StepStatus::DryRun,
            detail: Some("snapshot path B".into()),
            ..StepEvent::default()
        };

        let change = describe_step_change("backup.snapshot", Some(&left), Some(&right));
        assert_eq!(
            change,
            Some("  backup.snapshot          detail changed".into())
        );
    }
}
