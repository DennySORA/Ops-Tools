use crate::features::system_updater::domain::error::InfrastructureError;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RunLock {
    path: PathBuf,
}

impl RunLock {
    pub fn acquire(path: &Path) -> Result<Self, InfrastructureError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                InfrastructureError::filesystem("INFRA_LOCK_DIR_CREATE", parent, err.to_string())
            })?;
        }

        if path.exists() {
            clear_stale_lock(path)?;
        }

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|err| {
                InfrastructureError::filesystem("INFRA_LOCK_ACQUIRE", path, err.to_string())
            })?;
        let payload = format!("pid={}\nstarted_at_ms={}\n", std::process::id(), now_ms());
        file.write_all(payload.as_bytes()).map_err(|err| {
            InfrastructureError::filesystem("INFRA_LOCK_WRITE", path, err.to_string())
        })?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for RunLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn clear_stale_lock(path: &Path) -> Result<(), InfrastructureError> {
    let content = std::fs::read_to_string(path).map_err(|err| {
        InfrastructureError::filesystem("INFRA_LOCK_READ", path.to_path_buf(), err.to_string())
    })?;
    let pid = content
        .lines()
        .find_map(|line| line.strip_prefix("pid="))
        .and_then(|value| value.parse::<u32>().ok());

    match pid {
        Some(pid) if PathBuf::from(format!("/proc/{pid}")).exists() => {
            Err(InfrastructureError::filesystem(
                "INFRA_LOCK_HELD",
                path.to_path_buf(),
                format!("another update process is running with pid {pid}"),
            ))
        }
        _ => {
            std::fs::remove_file(path).map_err(|err| {
                InfrastructureError::filesystem(
                    "INFRA_LOCK_CLEAR",
                    path.to_path_buf(),
                    err.to_string(),
                )
            })?;
            Ok(())
        }
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::RunLock;
    use std::fs;

    #[test]
    fn acquires_and_releases_lock() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("update.lock");
        {
            let lock = RunLock::acquire(&path).expect("lock");
            assert!(lock.path().exists());
        }
        assert!(!path.exists());
    }

    #[test]
    fn clears_stale_lock_before_acquiring() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("update.lock");
        fs::write(&path, "pid=999999\nstarted_at_ms=1\n").expect("write stale lock");
        let _lock = RunLock::acquire(&path).expect("lock");
        assert!(path.exists());
    }
}
