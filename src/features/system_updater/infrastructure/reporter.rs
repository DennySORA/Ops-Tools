use crate::features::system_updater::domain::error::InfrastructureError;
use crate::features::system_updater::domain::report::{
    CommandEvent, NoteEvent, ReportArtifacts, RunContext, RunRecord, RunStatus, StepEvent,
};
use crate::features::system_updater::ports::{CommandObserver, RunReporter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct JsonFileReporter {
    inner: Arc<Mutex<ReporterState>>,
    run_id: String,
    artifact_dir: PathBuf,
}

struct ReporterState {
    run_record: RunRecord,
    report_path: PathBuf,
    log_path: PathBuf,
    active_step_id: Option<String>,
    next_command_id: u64,
}

impl JsonFileReporter {
    pub fn create(
        base_dir: &Path,
        dry_run: bool,
        scan_only: bool,
        context: RunContext,
    ) -> Result<Self, InfrastructureError> {
        fs::create_dir_all(base_dir).map_err(|err| {
            InfrastructureError::filesystem(
                "INFRA_REPORT_DIR_CREATE",
                base_dir.to_path_buf(),
                err.to_string(),
            )
        })?;

        let run_id = format!("run-{}-{}", now_ms(), std::process::id());
        let artifact_dir = base_dir.join(&run_id);
        fs::create_dir_all(&artifact_dir).map_err(|err| {
            InfrastructureError::filesystem(
                "INFRA_ARTIFACT_DIR_CREATE",
                artifact_dir.clone(),
                err.to_string(),
            )
        })?;

        let log_path = artifact_dir.join("session.log");
        let report_path = artifact_dir.join("report.json");
        fs::File::create(&log_path).map_err(|err| {
            InfrastructureError::filesystem("INFRA_LOG_CREATE", log_path.clone(), err.to_string())
        })?;

        let cwd = std::env::current_dir().map_err(|err| {
            InfrastructureError::probe("INFRA_CWD_READ", "current_dir", err.to_string())
        })?;
        let hostname = read_hostname();
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".into());

        let run_record = RunRecord {
            run_id: run_id.clone(),
            started_at_ms: now_ms(),
            finished_at_ms: None,
            overall_status: None,
            hostname,
            user,
            cwd,
            dry_run,
            scan_only,
            context,
            report_path: report_path.clone(),
            log_path: log_path.clone(),
            artifact_dir: artifact_dir.clone(),
            steps: Vec::new(),
            commands: Vec::new(),
            notes: Vec::new(),
        };

        let reporter = Self {
            inner: Arc::new(Mutex::new(ReporterState {
                run_record,
                report_path,
                log_path,
                active_step_id: None,
                next_command_id: 1,
            })),
            run_id,
            artifact_dir,
        };
        reporter.note("run started")?;
        Ok(reporter)
    }
}

impl RunReporter for JsonFileReporter {
    fn run_id(&self) -> &str {
        &self.run_id
    }

    fn note(&self, message: &str) -> Result<(), InfrastructureError> {
        let timestamp_ms = now_ms();
        let log_path = {
            let mut state = self.inner.lock().expect("report mutex poisoned");
            state.run_record.notes.push(NoteEvent {
                timestamp_ms,
                message: message.to_string(),
            });
            state.log_path.clone()
        };

        append_log_line(&log_path, &format!("NOTE {message}"))?;
        Ok(())
    }

    fn activate_step(&self, step_id: Option<&str>) -> Result<(), InfrastructureError> {
        let mut state = self.inner.lock().expect("report mutex poisoned");
        state.active_step_id = step_id.map(ToOwned::to_owned);
        Ok(())
    }

    fn record_step(&self, event: StepEvent) -> Result<(), InfrastructureError> {
        let log_path = {
            let mut state = self.inner.lock().expect("report mutex poisoned");
            state.run_record.steps.push(event.clone());
            state.log_path.clone()
        };

        let suffix = event
            .detail
            .as_deref()
            .map(|detail| format!(": {detail}"))
            .unwrap_or_default();
        append_log_line(
            &log_path,
            &format!(
                "STEP [{}] {} ({}/{}, {} ms){}",
                event.status.as_str(),
                event.step_id,
                event.index,
                event.total,
                event.duration_ms,
                suffix
            ),
        )?;
        Ok(())
    }

    fn artifact_dir(&self) -> PathBuf {
        self.artifact_dir.clone()
    }

    fn finalize(&self, status: RunStatus) -> Result<ReportArtifacts, InfrastructureError> {
        let (report_path, log_path, artifact_dir, bytes) = {
            let mut state = self.inner.lock().expect("report mutex poisoned");
            state.run_record.finished_at_ms = Some(now_ms());
            state.run_record.overall_status = Some(status.as_str().to_string());
            let bytes = serde_json::to_vec_pretty(&state.run_record).map_err(|err| {
                InfrastructureError::serialization("INFRA_REPORT_SERIALIZE", err.to_string())
            })?;
            (
                state.report_path.clone(),
                state.log_path.clone(),
                state.run_record.artifact_dir.clone(),
                bytes,
            )
        };

        fs::write(&report_path, bytes).map_err(|err| {
            InfrastructureError::filesystem(
                "INFRA_REPORT_WRITE",
                report_path.clone(),
                err.to_string(),
            )
        })?;
        append_log_line(
            &log_path,
            &format!("REPORT [{}] {}", status.as_str(), report_path.display()),
        )?;

        Ok(ReportArtifacts {
            run_id: self.run_id.clone(),
            report_path,
            log_path,
            artifact_dir,
        })
    }
}

impl CommandObserver for JsonFileReporter {
    fn record_command(&self, mut event: CommandEvent) -> Result<(), InfrastructureError> {
        let log_path = {
            let mut state = self.inner.lock().expect("report mutex poisoned");
            if event.command_id.is_none() {
                event.command_id = Some(state.next_command_id);
                state.next_command_id += 1;
            }
            if event.step_id.is_none() {
                event.step_id = state.active_step_id.clone();
            }
            state.run_record.commands.push(event.clone());
            state.log_path.clone()
        };

        let suffix = event
            .detail
            .as_deref()
            .map(|detail| format!(": {detail}"))
            .unwrap_or_default();
        append_log_line(
            &log_path,
            &format!(
                "COMMAND [{}] #{} {}{}",
                event.status.as_str(),
                event.command_id.unwrap_or_default(),
                event.command,
                suffix
            ),
        )?;
        Ok(())
    }
}

fn append_log_line(path: &Path, line: &str) -> Result<(), InfrastructureError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| InfrastructureError::filesystem("INFRA_LOG_OPEN", path, err.to_string()))?;
    writeln!(file, "[{}] {line}", now_ms()).map_err(|err| {
        InfrastructureError::filesystem("INFRA_LOG_APPEND", path.to_path_buf(), err.to_string())
    })?;
    Ok(())
}

fn read_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|value| value.trim().to_string())
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".into())
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
