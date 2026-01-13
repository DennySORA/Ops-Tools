use crate::core::{OperationError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub language: Option<String>,
    /// Menu usage statistics for sorting by frequency
    #[serde(default)]
    pub menu_usage: HashMap<String, u32>,
}

impl AppConfig {
    /// Increment usage count for a menu item
    pub fn increment_usage(&mut self, key: &str) {
        *self.menu_usage.entry(key.to_string()).or_insert(0) += 1;
    }

    /// Get usage count for a menu item
    pub fn get_usage(&self, key: &str) -> u32 {
        self.menu_usage.get(key).copied().unwrap_or(0)
    }
}

pub fn config_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|base| base.join("ops-tools").join("config.toml"))
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").map(PathBuf::from).map(|base| {
            base.join("Library")
                .join("Application Support")
                .join("ops-tools")
                .join("config.toml")
        })
    } else if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        Some(
            PathBuf::from(config_home)
                .join("ops-tools")
                .join("config.toml"),
        )
    } else {
        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|base| base.join(".config").join("ops-tools").join("config.toml"))
    }
}

pub fn load_config() -> Result<Option<AppConfig>> {
    let Some(path) = config_path() else {
        return Ok(None);
    };

    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;

    let config = toml::from_str(&raw).map_err(|err| OperationError::Config {
        key: path.display().to_string(),
        message: err.to_string(),
    })?;

    Ok(Some(config))
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let Some(path) = config_path() else {
        return Err(OperationError::Config {
            key: "config_path".to_string(),
            message: "Unable to resolve config directory".to_string(),
        });
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| OperationError::Io {
            path: parent.display().to_string(),
            source: err,
        })?;
    }

    let content = toml::to_string(config).map_err(|err| OperationError::Config {
        key: path.display().to_string(),
        message: err.to_string(),
    })?;

    fs::write(&path, content).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Env lock")
    }

    fn set_env(key: &str, value: &std::path::Path) {
        env::set_var(key, value);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn remove_env(key: &str) {
        env::remove_var(key);
    }

    fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
    }

    #[test]
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn test_config_path_uses_xdg() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let old_xdg = env::var_os("XDG_CONFIG_HOME");
        let old_home = env::var_os("HOME");
        set_env("XDG_CONFIG_HOME", temp.path());
        remove_env("HOME");

        let path = config_path().expect("Expected config path");
        assert!(path.starts_with(temp.path()));
        assert!(path.ends_with("ops-tools/config.toml"));

        restore_env("XDG_CONFIG_HOME", old_xdg);
        restore_env("HOME", old_home);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_config_path_uses_appdata() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let old_appdata = env::var_os("APPDATA");
        set_env("APPDATA", temp.path());

        let path = config_path().expect("Expected config path");
        assert!(path.starts_with(temp.path()));
        assert!(
            path.ends_with("ops-tools\\config.toml") || path.ends_with("ops-tools/config.toml")
        );

        restore_env("APPDATA", old_appdata);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_config_path_uses_app_support() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let old_home = env::var_os("HOME");
        set_env("HOME", temp.path());

        let path = config_path().expect("Expected config path");
        assert!(path.starts_with(temp.path()));
        assert!(path
            .to_string_lossy()
            .contains("Library/Application Support/ops-tools/config.toml"));

        restore_env("HOME", old_home);
    }

    #[test]
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn test_save_and_load_config() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let old_xdg = env::var_os("XDG_CONFIG_HOME");
        let old_home = env::var_os("HOME");
        set_env("XDG_CONFIG_HOME", temp.path());
        remove_env("HOME");

        let config = AppConfig {
            language: Some("en".to_string()),
            ..Default::default()
        };
        save_config(&config).unwrap();

        let loaded = load_config().unwrap().expect("Expected config");
        assert_eq!(loaded.language.as_deref(), Some("en"));

        restore_env("XDG_CONFIG_HOME", old_xdg);
        restore_env("HOME", old_home);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_save_and_load_config() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let old_home = env::var_os("HOME");
        set_env("HOME", temp.path());

        let config = AppConfig {
            language: Some("en".to_string()),
            ..Default::default()
        };
        save_config(&config).unwrap();

        let loaded = load_config().unwrap().expect("Expected config");
        assert_eq!(loaded.language.as_deref(), Some("en"));

        restore_env("HOME", old_home);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_save_and_load_config() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let old_appdata = env::var_os("APPDATA");
        set_env("APPDATA", temp.path());

        let config = AppConfig {
            language: Some("en".to_string()),
            ..Default::default()
        };
        save_config(&config).unwrap();

        let loaded = load_config().unwrap().expect("Expected config");
        assert_eq!(loaded.language.as_deref(), Some("en"));

        restore_env("APPDATA", old_appdata);
    }
}
