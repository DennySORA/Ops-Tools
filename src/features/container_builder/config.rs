use crate::core::{OperationError, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Configuration for Container Builder
/// Stores user preferences and recent values for quick selection
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BuilderConfig {
    /// Recently used image names
    #[serde(default)]
    pub recent_images: Vec<String>,

    /// Recently used tags
    #[serde(default)]
    pub recent_tags: Vec<String>,

    /// Recently used registries
    #[serde(default)]
    pub recent_registries: Vec<String>,
}

/// Get the config file path for container builder
fn config_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|base| base.join("ops-tools").join("container-builder.toml"))
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").map(PathBuf::from).map(|base| {
            base.join("Library")
                .join("Application Support")
                .join("ops-tools")
                .join("container-builder.toml")
        })
    } else if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        Some(
            PathBuf::from(config_home)
                .join("ops-tools")
                .join("container-builder.toml"),
        )
    } else {
        env::var_os("HOME").map(PathBuf::from).map(|base| {
            base.join(".config")
                .join("ops-tools")
                .join("container-builder.toml")
        })
    }
}

/// Load container builder configuration
pub fn load_builder_config() -> Result<BuilderConfig> {
    let Some(path) = config_path() else {
        return Ok(BuilderConfig::default());
    };

    if !path.exists() {
        return Ok(BuilderConfig::default());
    }

    let raw = fs::read_to_string(&path).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;

    let config = toml::from_str(&raw).map_err(|err| OperationError::Config {
        key: path.display().to_string(),
        message: err.to_string(),
    })?;

    Ok(config)
}

/// Save container builder configuration
pub fn save_builder_config(config: &BuilderConfig) -> Result<()> {
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

    #[test]
    fn test_default_config() {
        let config = BuilderConfig::default();
        assert!(config.recent_images.is_empty());
        assert!(config.recent_tags.is_empty());
        assert!(config.recent_registries.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let mut config = BuilderConfig::default();
        config.recent_images.push("myapp".to_string());
        config.recent_tags.push("latest".to_string());
        config
            .recent_registries
            .push("docker.io/myuser".to_string());

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: BuilderConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.recent_images, vec!["myapp"]);
        assert_eq!(deserialized.recent_tags, vec!["latest"]);
        assert_eq!(deserialized.recent_registries, vec!["docker.io/myuser"]);
    }
}
