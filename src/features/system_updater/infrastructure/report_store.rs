use crate::features::system_updater::domain::error::InfrastructureError;
use crate::features::system_updater::domain::report::RunRecord;
use std::path::{Path, PathBuf};

pub fn list_run_reports(
    base_dir: &Path,
    limit: usize,
) -> Result<Vec<PathBuf>, InfrastructureError> {
    let mut reports = Vec::new();
    if !base_dir.exists() {
        return Ok(reports);
    }

    for entry in std::fs::read_dir(base_dir).map_err(|err| {
        InfrastructureError::filesystem(
            "INFRA_REPORT_LIST",
            base_dir.to_path_buf(),
            err.to_string(),
        )
    })? {
        let entry = entry.map_err(|err| {
            InfrastructureError::filesystem(
                "INFRA_REPORT_LIST",
                base_dir.to_path_buf(),
                err.to_string(),
            )
        })?;
        let path = entry.path().join("report.json");
        if path.is_file() {
            reports.push(path);
        }
    }

    reports.sort();
    reports.reverse();
    reports.truncate(limit);
    Ok(reports)
}

pub fn load_run_record(path: &Path) -> Result<RunRecord, InfrastructureError> {
    let raw = std::fs::read_to_string(path).map_err(|err| {
        InfrastructureError::filesystem("INFRA_REPORT_READ", path.to_path_buf(), err.to_string())
    })?;
    serde_json::from_str(&raw)
        .map_err(|err| InfrastructureError::serialization("INFRA_REPORT_PARSE", err.to_string()))
}

pub fn resolve_run_record(
    base_dir: &Path,
    selector: Option<&str>,
) -> Result<RunRecord, InfrastructureError> {
    match selector {
        Some(selector) => {
            let candidate = PathBuf::from(selector);
            if candidate.is_file() {
                return load_run_record(&candidate);
            }

            let path = base_dir.join(selector).join("report.json");
            load_run_record(&path)
        }
        None => {
            let latest = list_run_reports(base_dir, 1)?
                .into_iter()
                .next()
                .ok_or_else(|| {
                    InfrastructureError::filesystem(
                        "INFRA_REPORT_NOT_FOUND",
                        base_dir.to_path_buf(),
                        "no reports found",
                    )
                })?;
            load_run_record(&latest)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{list_run_reports, resolve_run_record};
    use std::fs;

    #[test]
    fn lists_reports_newest_first() {
        let temp = tempfile::tempdir().expect("tempdir");
        let a = temp.path().join("run-a");
        let b = temp.path().join("run-b");
        fs::create_dir_all(&a).expect("mkdir a");
        fs::create_dir_all(&b).expect("mkdir b");
        fs::write(a.join("report.json"), sample_report_json("run-a")).expect("report a");
        fs::write(b.join("report.json"), sample_report_json("run-b")).expect("report b");

        let reports = list_run_reports(temp.path(), 10).expect("list");
        assert_eq!(reports.len(), 2);
        assert!(reports[0].display().to_string().contains("run-b"));
    }

    #[test]
    fn resolves_latest_record_by_default() {
        let temp = tempfile::tempdir().expect("tempdir");
        let latest = temp.path().join("run-z");
        fs::create_dir_all(&latest).expect("mkdir");
        fs::write(latest.join("report.json"), sample_report_json("run-z")).expect("report");

        let record = resolve_run_record(temp.path(), None).expect("resolve");
        assert_eq!(record.run_id, "run-z");
    }

    fn sample_report_json(run_id: &str) -> String {
        format!(
            r#"{{
  "run_id": "{run_id}",
  "started_at_ms": 1,
  "finished_at_ms": 2,
  "overall_status": "ok",
  "hostname": "test-host",
  "user": "tester",
  "cwd": "/tmp",
  "dry_run": false,
  "scan_only": false,
  "context": {{
    "subcommand": "run",
    "profile": null,
    "config_path": null,
    "lock_path": null
  }},
  "report_path": "/tmp/report.json",
  "log_path": "/tmp/session.log",
  "artifact_dir": "/tmp",
  "steps": [],
  "commands": [],
  "notes": []
}}"#
        )
    }
}
