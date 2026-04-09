use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::InfrastructureError;
use crate::features::system_updater::domain::report::{CommandEvent, CommandMode, CommandStatus};
use crate::features::system_updater::ports::{CommandExecutor, CommandObserver};
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[derive(Clone)]
pub struct ShellCommandExecutor<O> {
    dry_run: bool,
    observer: O,
}

impl<O> ShellCommandExecutor<O> {
    pub fn new(dry_run: bool, observer: O) -> Self {
        Self { dry_run, observer }
    }
}

impl<O> CommandExecutor for ShellCommandExecutor<O>
where
    O: CommandObserver,
{
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn run(&self, command: &CommandSpec) -> Result<(), InfrastructureError> {
        if self.dry_run {
            self.observer.record_command(CommandEvent {
                command_id: None,
                step_id: None,
                command: command.display(),
                mode: CommandMode::Stream,
                status: CommandStatus::DryRun,
                exit_code: None,
                detail: None,
                cwd: command.cwd().map(ToOwned::to_owned),
                timeout_ms: command.timeout_ms(),
                retry_attempt: 0,
                sudo: command.sudo(),
            })?;
            return Ok(());
        }

        for attempt in 0..=command.retry_limit() {
            match execute(command)? {
                ProcessExecution::Completed(output) => {
                    emit_process_output(&output.stdout, &output.stderr);
                    if output.status.success() {
                        self.observer.record_command(CommandEvent {
                            command_id: None,
                            step_id: None,
                            command: command.display(),
                            mode: CommandMode::Stream,
                            status: CommandStatus::Ok,
                            exit_code: output.status.code(),
                            detail: None,
                            cwd: command.cwd().map(ToOwned::to_owned),
                            timeout_ms: command.timeout_ms(),
                            retry_attempt: attempt,
                            sudo: command.sudo(),
                        })?;
                        return Ok(());
                    }

                    let detail = first_non_empty_line(&output.stderr)
                        .or_else(|| first_non_empty_line(&output.stdout))
                        .unwrap_or("command returned non-zero exit status")
                        .to_string();
                    let err = InfrastructureError::command_failed(
                        "INFRA_COMMAND_FAILED",
                        command,
                        output.status.code(),
                        detail.clone(),
                    );
                    self.observer.record_command(CommandEvent {
                        command_id: None,
                        step_id: None,
                        command: command.display(),
                        mode: CommandMode::Stream,
                        status: CommandStatus::Failed,
                        exit_code: output.status.code(),
                        detail: Some(detail),
                        cwd: command.cwd().map(ToOwned::to_owned),
                        timeout_ms: command.timeout_ms(),
                        retry_attempt: attempt,
                        sudo: command.sudo(),
                    })?;
                    if attempt == command.retry_limit() {
                        return Err(err);
                    }
                }
                ProcessExecution::TimedOut(output, timeout_ms) => {
                    emit_process_output(&output.stdout, &output.stderr);
                    let err = InfrastructureError::command_timed_out(
                        "INFRA_COMMAND_TIMEOUT",
                        command,
                        timeout_ms,
                    );
                    self.observer.record_command(CommandEvent {
                        command_id: None,
                        step_id: None,
                        command: command.display(),
                        mode: CommandMode::Stream,
                        status: CommandStatus::TimedOut,
                        exit_code: output.status.code(),
                        detail: Some(err.to_string()),
                        cwd: command.cwd().map(ToOwned::to_owned),
                        timeout_ms: Some(timeout_ms),
                        retry_attempt: attempt,
                        sudo: command.sudo(),
                    })?;
                    if attempt == command.retry_limit() {
                        return Err(err);
                    }
                }
            }
        }

        unreachable!("retry loop returns on terminal attempt")
    }

    fn capture(&self, command: &CommandSpec) -> Result<String, InfrastructureError> {
        if self.dry_run {
            self.observer.record_command(CommandEvent {
                command_id: None,
                step_id: None,
                command: command.display(),
                mode: CommandMode::Capture,
                status: CommandStatus::DryRun,
                exit_code: None,
                detail: None,
                cwd: command.cwd().map(ToOwned::to_owned),
                timeout_ms: command.timeout_ms(),
                retry_attempt: 0,
                sudo: command.sudo(),
            })?;
            return Ok(String::new());
        }

        for attempt in 0..=command.retry_limit() {
            match execute(command)? {
                ProcessExecution::Completed(output) => {
                    if output.status.success() {
                        self.observer.record_command(CommandEvent {
                            command_id: None,
                            step_id: None,
                            command: command.display(),
                            mode: CommandMode::Capture,
                            status: CommandStatus::Ok,
                            exit_code: output.status.code(),
                            detail: None,
                            cwd: command.cwd().map(ToOwned::to_owned),
                            timeout_ms: command.timeout_ms(),
                            retry_attempt: attempt,
                            sudo: command.sudo(),
                        })?;
                        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
                    }

                    let detail = first_non_empty_line(&output.stderr)
                        .unwrap_or("no stderr output")
                        .to_string();
                    let err = InfrastructureError::command_failed(
                        "INFRA_COMMAND_FAILED",
                        command,
                        output.status.code(),
                        detail.clone(),
                    );
                    self.observer.record_command(CommandEvent {
                        command_id: None,
                        step_id: None,
                        command: command.display(),
                        mode: CommandMode::Capture,
                        status: CommandStatus::Failed,
                        exit_code: output.status.code(),
                        detail: Some(detail),
                        cwd: command.cwd().map(ToOwned::to_owned),
                        timeout_ms: command.timeout_ms(),
                        retry_attempt: attempt,
                        sudo: command.sudo(),
                    })?;
                    if attempt == command.retry_limit() {
                        return Err(err);
                    }
                }
                ProcessExecution::TimedOut(output, timeout_ms) => {
                    let err = InfrastructureError::command_timed_out(
                        "INFRA_COMMAND_TIMEOUT",
                        command,
                        timeout_ms,
                    );
                    self.observer.record_command(CommandEvent {
                        command_id: None,
                        step_id: None,
                        command: command.display(),
                        mode: CommandMode::Capture,
                        status: CommandStatus::TimedOut,
                        exit_code: output.status.code(),
                        detail: Some(err.to_string()),
                        cwd: command.cwd().map(ToOwned::to_owned),
                        timeout_ms: Some(timeout_ms),
                        retry_attempt: attempt,
                        sudo: command.sudo(),
                    })?;
                    if attempt == command.retry_limit() {
                        return Err(err);
                    }
                }
            }
        }

        unreachable!("retry loop returns on terminal attempt")
    }
}

enum ProcessExecution {
    Completed(Output),
    TimedOut(Output, u64),
}

fn execute(command: &CommandSpec) -> Result<ProcessExecution, InfrastructureError> {
    let (program, args) = effective_command(command);
    let mut process = Command::new(program);
    process.args(args);
    process.stdin(Stdio::null());
    process.stdout(Stdio::piped());
    process.stderr(Stdio::piped());
    if let Some(cwd) = command.cwd() {
        process.current_dir(cwd);
    }
    for (key, value) in command.env() {
        process.env(key, value);
    }

    let mut child = process.spawn().map_err(|err| {
        InfrastructureError::command_spawn("INFRA_COMMAND_SPAWN", command, err.to_string())
    })?;

    if let Some(timeout_ms) = command.timeout_ms() {
        let timeout = Duration::from_millis(timeout_ms);
        match child.wait_timeout(timeout).map_err(|err| {
            InfrastructureError::command_spawn("INFRA_COMMAND_WAIT", command, err.to_string())
        })? {
            Some(_status) => {
                let output = child.wait_with_output().map_err(|err| {
                    InfrastructureError::command_spawn(
                        "INFRA_COMMAND_WAIT",
                        command,
                        err.to_string(),
                    )
                })?;
                Ok(ProcessExecution::Completed(output))
            }
            None => {
                let _ = child.kill();
                let output = child.wait_with_output().map_err(|err| {
                    InfrastructureError::command_spawn(
                        "INFRA_COMMAND_WAIT",
                        command,
                        err.to_string(),
                    )
                })?;
                Ok(ProcessExecution::TimedOut(
                    synthesize_timeout_output(output, timeout_ms),
                    timeout_ms,
                ))
            }
        }
    } else {
        let output = child.wait_with_output().map_err(|err| {
            InfrastructureError::command_spawn("INFRA_COMMAND_WAIT", command, err.to_string())
        })?;
        Ok(ProcessExecution::Completed(output))
    }
}

fn effective_command(command: &CommandSpec) -> (&str, Vec<String>) {
    if command.sudo() {
        let mut args = vec![command.program().to_string()];
        args.extend(command.args().iter().cloned());
        ("sudo", args)
    } else {
        (command.program(), command.args().to_vec())
    }
}

fn emit_process_output(stdout: &[u8], stderr: &[u8]) {
    let stdout = String::from_utf8_lossy(stdout);
    if !stdout.is_empty() {
        print!("{stdout}");
    }

    let stderr = String::from_utf8_lossy(stderr);
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
}

fn first_non_empty_line(bytes: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(bytes).ok()?;
    text.lines().find(|line| !line.trim().is_empty())
}

fn synthesize_timeout_output(mut output: Output, timeout_ms: u64) -> Output {
    if output.stderr.is_empty() {
        output.stderr = format!("timed out after {timeout_ms} ms").into_bytes();
    }
    Output {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    }
}
