use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::InfrastructureError;
use crate::features::system_updater::domain::report::{
    CommandEvent, CommandMode, CommandStatus, ReportArtifacts, RunStatus, StepEvent,
};
use crate::features::system_updater::ports::{
    CommandExecutor, CommandObserver, EnvironmentReader, FileSystem, RunReporter, SystemProbe,
    ToolProbe,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone)]
pub struct FakeReporter {
    inner: Rc<RefCell<FakeReporterState>>,
    run_id: String,
    artifact_dir: PathBuf,
}

#[derive(Default)]
struct FakeReporterState {
    notes: Vec<String>,
    steps: Vec<StepEvent>,
    commands: Vec<CommandEvent>,
    finalized: Option<RunStatus>,
    active_step_id: Option<String>,
}

impl FakeReporter {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(FakeReporterState::default())),
            run_id: "test-run".to_string(),
            artifact_dir: PathBuf::from("/tmp/update-test-artifacts"),
        }
    }

    pub fn commands(&self) -> Vec<CommandEvent> {
        self.inner.borrow().commands.clone()
    }

    pub fn steps(&self) -> Vec<StepEvent> {
        self.inner.borrow().steps.clone()
    }

    pub fn notes(&self) -> Vec<String> {
        self.inner.borrow().notes.clone()
    }
}

impl Default for FakeReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl RunReporter for FakeReporter {
    fn run_id(&self) -> &str {
        &self.run_id
    }

    fn note(&self, message: &str) -> Result<(), InfrastructureError> {
        self.inner.borrow_mut().notes.push(message.to_string());
        Ok(())
    }

    fn activate_step(&self, step_id: Option<&str>) -> Result<(), InfrastructureError> {
        self.inner.borrow_mut().active_step_id = step_id.map(ToOwned::to_owned);
        Ok(())
    }

    fn record_step(&self, event: StepEvent) -> Result<(), InfrastructureError> {
        self.inner.borrow_mut().steps.push(event);
        Ok(())
    }

    fn artifact_dir(&self) -> PathBuf {
        self.artifact_dir.clone()
    }

    fn finalize(&self, status: RunStatus) -> Result<ReportArtifacts, InfrastructureError> {
        self.inner.borrow_mut().finalized = Some(status);
        Ok(ReportArtifacts {
            run_id: self.run_id.clone(),
            report_path: self.artifact_dir.join("report.json"),
            log_path: self.artifact_dir.join("session.log"),
            artifact_dir: self.artifact_dir.clone(),
        })
    }
}

impl CommandObserver for FakeReporter {
    fn record_command(&self, mut event: CommandEvent) -> Result<(), InfrastructureError> {
        let mut state = self.inner.borrow_mut();
        if event.step_id.is_none() {
            event.step_id = state.active_step_id.clone();
        }
        if event.command_id.is_none() {
            let next_id = state.commands.len() as u64 + 1;
            event.command_id = Some(next_id);
        }
        state.commands.push(event);
        Ok(())
    }
}

#[derive(Clone)]
pub struct FakeExecutor {
    dry_run: bool,
    reporter: Option<FakeReporter>,
    inner: Rc<RefCell<FakeExecutorState>>,
}

#[derive(Default)]
struct FakeExecutorState {
    commands: Vec<String>,
    run_results: HashMap<String, VecDeque<Result<(), InfrastructureError>>>,
    capture_results: HashMap<String, VecDeque<Result<String, InfrastructureError>>>,
}

impl FakeExecutor {
    pub fn new(dry_run: bool) -> Self {
        Self {
            dry_run,
            reporter: None,
            inner: Rc::new(RefCell::new(FakeExecutorState::default())),
        }
    }

    pub fn with_reporter(dry_run: bool, reporter: FakeReporter) -> Self {
        Self {
            dry_run,
            reporter: Some(reporter),
            inner: Rc::new(RefCell::new(FakeExecutorState::default())),
        }
    }

    pub fn push_run_ok(&self, command: &str) {
        self.push_run_result(command, Ok(()));
    }

    pub fn push_run_error(&self, command: &str, err: InfrastructureError) {
        self.push_run_result(command, Err(err));
    }

    pub fn push_capture_ok(&self, command: &str, output: &str) {
        self.push_capture_result(command, Ok(output.to_string()));
    }

    pub fn push_capture_error(&self, command: &str, err: InfrastructureError) {
        self.push_capture_result(command, Err(err));
    }

    pub fn commands(&self) -> Vec<String> {
        self.inner.borrow().commands.clone()
    }

    fn push_run_result(&self, command: &str, result: Result<(), InfrastructureError>) {
        self.inner
            .borrow_mut()
            .run_results
            .entry(command.to_string())
            .or_default()
            .push_back(result);
    }

    fn push_capture_result(&self, command: &str, result: Result<String, InfrastructureError>) {
        self.inner
            .borrow_mut()
            .capture_results
            .entry(command.to_string())
            .or_default()
            .push_back(result);
    }

    fn pop_run_result(&self, command: &str) -> Result<(), InfrastructureError> {
        self.inner
            .borrow_mut()
            .run_results
            .get_mut(command)
            .and_then(VecDeque::pop_front)
            .unwrap_or(Ok(()))
    }

    fn pop_capture_result(&self, command: &str) -> Result<String, InfrastructureError> {
        self.inner
            .borrow_mut()
            .capture_results
            .get_mut(command)
            .and_then(VecDeque::pop_front)
            .unwrap_or_else(|| Ok(String::new()))
    }

    fn record_command(
        &self,
        command: &CommandSpec,
        mode: CommandMode,
        status: CommandStatus,
        exit_code: Option<i32>,
        detail: Option<String>,
        retry_attempt: u8,
    ) -> Result<(), InfrastructureError> {
        if let Some(reporter) = &self.reporter {
            reporter.record_command(CommandEvent {
                command_id: None,
                step_id: None,
                command: command.display(),
                mode,
                status,
                exit_code,
                detail,
                cwd: command.cwd().map(ToOwned::to_owned),
                timeout_ms: command.timeout_ms(),
                retry_attempt,
                sudo: command.sudo(),
            })?;
        }
        Ok(())
    }
}

impl CommandExecutor for FakeExecutor {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn run(&self, command: &CommandSpec) -> Result<(), InfrastructureError> {
        let display = command.display();
        self.inner.borrow_mut().commands.push(display.clone());

        if self.dry_run {
            self.record_command(
                command,
                CommandMode::Stream,
                CommandStatus::DryRun,
                None,
                None,
                0,
            )?;
            return Ok(());
        }

        match self.pop_run_result(&display) {
            Ok(()) => self.record_command(
                command,
                CommandMode::Stream,
                CommandStatus::Ok,
                Some(0),
                None,
                0,
            ),
            Err(err) => {
                self.record_command(
                    command,
                    CommandMode::Stream,
                    CommandStatus::Failed,
                    Some(1),
                    Some(err.to_string()),
                    0,
                )?;
                Err(err)
            }
        }
    }

    fn capture(&self, command: &CommandSpec) -> Result<String, InfrastructureError> {
        let display = command.display();
        self.inner.borrow_mut().commands.push(display.clone());

        if self.dry_run {
            self.record_command(
                command,
                CommandMode::Capture,
                CommandStatus::DryRun,
                None,
                None,
                0,
            )?;
            return Ok(String::new());
        }

        match self.pop_capture_result(&display) {
            Ok(output) => {
                self.record_command(
                    command,
                    CommandMode::Capture,
                    CommandStatus::Ok,
                    Some(0),
                    None,
                    0,
                )?;
                Ok(output)
            }
            Err(err) => {
                self.record_command(
                    command,
                    CommandMode::Capture,
                    CommandStatus::Failed,
                    Some(1),
                    Some(err.to_string()),
                    0,
                )?;
                Err(err)
            }
        }
    }
}

#[derive(Default)]
pub struct FakeHost {
    inner: Rc<RefCell<FakeHostState>>,
}

impl FakeHost {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(FakeHostState {
                hostname: "test-host".to_string(),
                ..FakeHostState::default()
            })),
        }
    }

    pub fn set_env(&mut self, key: &str, value: &str) {
        self.inner
            .borrow_mut()
            .env
            .insert(key.to_string(), value.to_string());
    }

    pub fn set_free_space(&mut self, path: &str, gib: u64) {
        self.inner
            .borrow_mut()
            .free_space
            .insert(PathBuf::from(path), gib);
    }

    pub fn set_dns(&mut self, host: &str, result: Result<bool, InfrastructureError>) {
        self.inner.borrow_mut().dns.insert(host.to_string(), result);
    }

    pub fn add_command(&mut self, binary: &str, path: PathBuf, writable: bool) {
        if writable {
            self.inner.borrow_mut().writable_paths.insert(path.clone());
        }
        self.inner
            .borrow_mut()
            .commands
            .insert(binary.to_string(), path);
    }

    pub fn add_file(&mut self, path: impl AsRef<Path>, contents: &str) {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            self.inner.borrow_mut().dirs.insert(parent.to_path_buf());
        }
        self.inner
            .borrow_mut()
            .files
            .insert(path, contents.to_string());
    }

    pub fn add_dir(&mut self, path: impl AsRef<Path>) {
        self.inner
            .borrow_mut()
            .dirs
            .insert(path.as_ref().to_path_buf());
    }
}

impl EnvironmentReader for FakeHost {
    fn var(&self, key: &str) -> Option<String> {
        self.inner.borrow().env.get(key).cloned()
    }

    fn current_dir(&self) -> Result<PathBuf, InfrastructureError> {
        Ok(PathBuf::from("/tmp/update-test"))
    }
}

impl ToolProbe for FakeHost {
    fn command_path(&self, name: &str) -> Option<PathBuf> {
        self.inner.borrow().commands.get(name).cloned()
    }

    fn is_writable(&self, path: &Path) -> bool {
        self.inner.borrow().writable_paths.contains(path)
    }
}

impl FileSystem for FakeHost {
    fn exists(&self, path: &Path) -> bool {
        let state = self.inner.borrow();
        state.files.contains_key(path)
            || state.dirs.contains(path)
            || state.commands.values().any(|candidate| candidate == path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.inner.borrow().dirs.contains(path)
    }

    fn read_to_string(&self, path: &Path) -> Result<String, InfrastructureError> {
        self.inner.borrow().files.get(path).cloned().ok_or_else(|| {
            InfrastructureError::filesystem("INFRA_FILE_READ", path, "file not found")
        })
    }

    fn write_string(&self, path: &Path, contents: &str) -> Result<(), InfrastructureError> {
        let mut state = self.inner.borrow_mut();
        if let Some(parent) = path.parent() {
            state.dirs.insert(parent.to_path_buf());
        }
        state.files.insert(path.to_path_buf(), contents.to_string());
        Ok(())
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<(), InfrastructureError> {
        let content = self.read_to_string(from)?;
        self.write_string(to, &content)?;
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), InfrastructureError> {
        let content = self.read_to_string(from)?;
        let mut state = self.inner.borrow_mut();
        state.files.remove(from);
        if let Some(parent) = to.parent() {
            state.dirs.insert(parent.to_path_buf());
        }
        state.files.insert(to.to_path_buf(), content);
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), InfrastructureError> {
        self.inner.borrow_mut().dirs.insert(path.to_path_buf());
        Ok(())
    }
}

impl SystemProbe for FakeHost {
    fn hostname(&self) -> Result<String, InfrastructureError> {
        Ok(self.inner.borrow().hostname.clone())
    }

    fn free_space_gib(&self, path: &Path) -> Result<u64, InfrastructureError> {
        Ok(*self.inner.borrow().free_space.get(path).unwrap_or(&100))
    }

    fn dns_resolves(&self, host: &str) -> Result<bool, InfrastructureError> {
        self.inner
            .borrow()
            .dns
            .get(host)
            .cloned()
            .unwrap_or(Ok(true))
    }
}

#[derive(Default)]
struct FakeHostState {
    env: HashMap<String, String>,
    commands: HashMap<String, PathBuf>,
    writable_paths: HashSet<PathBuf>,
    files: HashMap<PathBuf, String>,
    dirs: HashSet<PathBuf>,
    free_space: HashMap<PathBuf, u64>,
    dns: HashMap<String, Result<bool, InfrastructureError>>,
    hostname: String,
}
