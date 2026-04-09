use crate::features::system_updater::domain::error::InfrastructureError;
use crate::features::system_updater::ports::{
    EnvironmentReader, FileSystem, SystemProbe, ToolProbe,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Default)]
pub struct HostRuntime;

impl HostRuntime {
    pub fn extend_path_for_common_tools() {
        let home = std::env::var("HOME").unwrap_or_default();
        let current_path = std::env::var("PATH").unwrap_or_default();
        let extra = [
            format!("{home}/.cargo/bin"),
            format!("{home}/.local/bin"),
            "/usr/local/cuda/bin".to_string(),
            "/snap/bin".to_string(),
        ];

        let mut prepend = Vec::new();
        for directory in &extra {
            if Path::new(directory).is_dir() && !current_path.contains(directory) {
                prepend.push(directory.clone());
            }
        }

        if !prepend.is_empty() {
            prepend.push(current_path);
            // SAFETY: invoked at startup before background threads exist.
            unsafe { std::env::set_var("PATH", prepend.join(":")) };
        }
    }
}

impl EnvironmentReader for HostRuntime {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn current_dir(&self) -> Result<PathBuf, InfrastructureError> {
        std::env::current_dir().map_err(|err| {
            InfrastructureError::probe("INFRA_CWD_READ", "current_dir", err.to_string())
        })
    }
}

impl ToolProbe for HostRuntime {
    fn command_path(&self, name: &str) -> Option<PathBuf> {
        if name.contains('/') {
            let path = PathBuf::from(name);
            return path.is_file().then_some(path);
        }

        let path_var = std::env::var("PATH").unwrap_or_default();
        path_var
            .split(':')
            .filter(|entry| !entry.is_empty())
            .map(|entry| Path::new(entry).join(name))
            .find(|candidate| candidate.is_file())
    }

    fn is_writable(&self, path: &Path) -> bool {
        Command::new("test")
            .arg("-w")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}

impl FileSystem for HostRuntime {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn read_to_string(&self, path: &Path) -> Result<String, InfrastructureError> {
        std::fs::read_to_string(path).map_err(|err| {
            InfrastructureError::filesystem("INFRA_FILE_READ", path.to_path_buf(), err.to_string())
        })
    }

    fn write_string(&self, path: &Path, contents: &str) -> Result<(), InfrastructureError> {
        std::fs::write(path, contents).map_err(|err| {
            InfrastructureError::filesystem("INFRA_FILE_WRITE", path.to_path_buf(), err.to_string())
        })
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<(), InfrastructureError> {
        std::fs::copy(from, to).map(|_| ()).map_err(|err| {
            InfrastructureError::filesystem(
                "INFRA_FILE_COPY",
                format!("{} -> {}", from.display(), to.display()),
                err.to_string(),
            )
        })
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), InfrastructureError> {
        std::fs::rename(from, to).map_err(|err| {
            InfrastructureError::filesystem(
                "INFRA_FILE_RENAME",
                format!("{} -> {}", from.display(), to.display()),
                err.to_string(),
            )
        })
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), InfrastructureError> {
        std::fs::create_dir_all(path).map_err(|err| {
            InfrastructureError::filesystem("INFRA_DIR_CREATE", path.to_path_buf(), err.to_string())
        })
    }
}

impl SystemProbe for HostRuntime {
    fn hostname(&self) -> Result<String, InfrastructureError> {
        std::fs::read_to_string("/etc/hostname")
            .map(|value| value.trim().to_string())
            .map_err(|err| {
                InfrastructureError::filesystem(
                    "INFRA_HOSTNAME_READ",
                    "/etc/hostname",
                    err.to_string(),
                )
            })
    }

    fn free_space_gib(&self, path: &Path) -> Result<u64, InfrastructureError> {
        let command = format!("df -Pk {}", path.display());
        let output = Command::new("df")
            .args(["-Pk", &path.to_string_lossy()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| {
                InfrastructureError::probe("INFRA_DF_SPAWN", command.clone(), err.to_string())
            })?;

        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr)
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("no stderr output")
                .to_string();
            return Err(InfrastructureError::probe(
                "INFRA_DF_FAILED",
                command,
                detail,
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let available_kib = stdout
            .lines()
            .nth(1)
            .and_then(|line| line.split_whitespace().nth(3))
            .ok_or_else(|| {
                InfrastructureError::probe(
                    "INFRA_DF_PARSE",
                    command.clone(),
                    "unexpected df output format",
                )
            })?
            .parse::<u64>()
            .map_err(|err| {
                InfrastructureError::probe(
                    "INFRA_DF_PARSE",
                    command,
                    format!("failed to parse available space: {err}"),
                )
            })?;

        Ok(available_kib / 1024 / 1024)
    }

    fn dns_resolves(&self, host: &str) -> Result<bool, InfrastructureError> {
        let output = Command::new("getent")
            .args(["hosts", host])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| {
                InfrastructureError::probe(
                    "INFRA_DNS_SPAWN",
                    format!("getent hosts {host}"),
                    err.to_string(),
                )
            })?;

        if output.status.success() {
            Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
        } else {
            let detail = String::from_utf8_lossy(&output.stderr)
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("lookup returned no output")
                .to_string();
            Err(InfrastructureError::probe(
                "INFRA_DNS_FAILED",
                format!("getent hosts {host}"),
                detail,
            ))
        }
    }
}
