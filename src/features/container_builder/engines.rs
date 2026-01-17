use crate::core::Result;
use crate::ui::Console;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

use super::types::{BuildContext, BuildResult};

/// Trait for container build engines
pub trait BuildEngine {
    /// Get the engine name
    fn name(&self) -> &'static str;

    /// Check if the engine is available on the system
    fn is_available(&self) -> bool;

    /// Build a container image
    fn build(&self, context: &BuildContext) -> Result<BuildResult>;

    /// Push a container image to registry
    fn push(&self, context: &BuildContext) -> Result<BuildResult>;
}

/// Docker build engine using docker buildx
pub struct DockerEngine;

impl BuildEngine for DockerEngine {
    fn name(&self) -> &'static str {
        "Docker"
    }

    fn is_available(&self) -> bool {
        Command::new("docker")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn build(&self, context: &BuildContext) -> Result<BuildResult> {
        let platforms: Vec<String> = context
            .architecture
            .iter()
            .map(|a| a.platform().to_string())
            .collect();

        let mut args = vec![
            "buildx".to_string(),
            "build".to_string(),
            "--platform".to_string(),
            platforms.join(","),
            "-f".to_string(),
            context.dockerfile.display().to_string(),
            "-t".to_string(),
            context.local_image_ref(),
        ];

        // For Jetson Nano, add specific build args if needed
        if context.architecture.iter().any(|a| a.is_jetson()) {
            args.push("--build-arg".to_string());
            args.push("TARGETPLATFORM=linux/arm64".to_string());
        }

        // Load the image to local docker (for single platform builds)
        args.push("--load".to_string());

        // Do not remove intermediate containers
        args.push("--rm=false".to_string());

        // Context directory
        args.push(context.context_dir.display().to_string());

        execute_command("docker", &args)
    }

    fn push(&self, context: &BuildContext) -> Result<BuildResult> {
        let full_ref = context.full_image_ref();
        let local_ref = context.local_image_ref();

        // Tag for registry if needed
        if context.registry.is_some() {
            let tag_result = execute_command("docker", &["tag", &local_ref, &full_ref])?;
            if !tag_result.success {
                return Ok(tag_result);
            }
        }

        // Push
        execute_command("docker", &["push", &full_ref])
    }
}

/// Buildah build engine
pub struct BuildahEngine;

impl BuildEngine for BuildahEngine {
    fn name(&self) -> &'static str {
        "Buildah"
    }

    fn is_available(&self) -> bool {
        Command::new("buildah")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn build(&self, context: &BuildContext) -> Result<BuildResult> {
        let platforms: Vec<String> = context
            .architecture
            .iter()
            .map(|a| a.platform().to_string())
            .collect();

        let mut args = vec![
            "build".to_string(),
            "--platform".to_string(),
            platforms.join(","),
            "--layers".to_string(),
            "-f".to_string(),
            context.dockerfile.display().to_string(),
            "-t".to_string(),
            context.local_image_ref(),
        ];

        // Do not remove intermediate containers
        args.push("--rm=false".to_string());

        // Context directory
        args.push(context.context_dir.display().to_string());

        execute_command("buildah", &args)
    }

    fn push(&self, context: &BuildContext) -> Result<BuildResult> {
        let full_ref = context.full_image_ref();
        let local_ref = context.local_image_ref();

        // Tag for registry if needed
        if context.registry.is_some() {
            let tag_result = execute_command("buildah", &["tag", &local_ref, &full_ref])?;
            if !tag_result.success {
                return Ok(tag_result);
            }
        }

        // Push using buildah
        execute_command("buildah", &["push", &full_ref])
    }
}

/// Execute a command and stream output in real-time
fn execute_command<S: AsRef<str>>(program: &str, args: &[S]) -> Result<BuildResult> {
    let args_str: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
    let console = Console::new();

    let mut child = Command::new(program)
        .args(&args_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| crate::core::OperationError::Command {
            command: program.to_string(),
            message: err.to_string(),
        })?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        reader
            .lines()
            .for_each(|line| console.raw(&line.unwrap_or_default()));
    });

    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        reader
            .lines()
            .for_each(|line| console.raw(&line.unwrap_or_default()));
    });

    stdout_thread.join().expect("Stdout thread panicked");
    stderr_thread.join().expect("Stderr thread panicked");

    let status = child
        .wait()
        .map_err(|err| crate::core::OperationError::Command {
            command: program.to_string(),
            message: err.to_string(),
        })?;

    Ok(BuildResult {
        success: status.success(),
        exit_code: status.code(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_engine_name() {
        let engine = DockerEngine;
        assert_eq!(engine.name(), "Docker");
    }

    #[test]
    fn test_buildah_engine_name() {
        let engine = BuildahEngine;
        assert_eq!(engine.name(), "Buildah");
    }
}
