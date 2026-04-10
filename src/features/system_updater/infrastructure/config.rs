use crate::features::system_updater::domain::config::Config;
use crate::features::system_updater::domain::error::InfrastructureError;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigLoad {
    pub config: Config,
    pub source: Option<PathBuf>,
}

pub fn load_config(explicit_path: Option<&Path>) -> Result<ConfigLoad, InfrastructureError> {
    if let Some(path) = explicit_path {
        let expanded = expand_path(path);
        let config = load_config_file(&expanded)?;
        return Ok(ConfigLoad {
            config,
            source: Some(expanded),
        });
    }

    for candidate in default_config_paths() {
        if candidate.is_file() {
            let config = load_config_file(&candidate)?;
            return Ok(ConfigLoad {
                config,
                source: Some(candidate),
            });
        }
    }

    let mut config = Config::default();
    expand_config_paths(&mut config);
    Ok(ConfigLoad {
        config,
        source: None,
    })
}

fn load_config_file(path: &Path) -> Result<Config, InfrastructureError> {
    let raw = std::fs::read_to_string(path).map_err(|err| {
        InfrastructureError::filesystem("INFRA_CONFIG_READ", path.to_path_buf(), err.to_string())
    })?;

    let mut config: Config = toml::from_str(&raw)
        .map_err(|err| InfrastructureError::serialization("INFRA_CONFIG_PARSE", err.to_string()))?;
    expand_config_paths(&mut config);
    Ok(config)
}

fn default_config_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("update.toml"),
        PathBuf::from("update.config.toml"),
        config_home().join("update").join("config.toml"),
    ]
    .into_iter()
    .map(|path| expand_path(&path))
    .collect()
}

fn config_home() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| {
        if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(dir)
        } else {
            home_dir().join(".config")
        }
    })
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn expand_config_paths(config: &mut Config) {
    config.report.dir = expand_path(&config.report.dir);
    config.runtime.lock_path = expand_path(&config.runtime.lock_path);
    config.docker.compose_projects = config
        .docker
        .compose_projects
        .iter()
        .map(|path| expand_path(path))
        .collect();
}

fn expand_path(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if raw == "~" {
        home_dir()
    } else if let Some(rest) = raw.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::load_config;
    use std::fs;

    #[test]
    fn loads_explicit_config_and_expands_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("update.toml");
        fs::write(
            &path,
            r#"
[report]
dir = "~/custom-reports"

[docker]
compose_projects = ["~/stack-a"]

[tools.conda]
envs = ["base", "ml"]
"#,
        )
        .expect("write config");

        let loaded = load_config(Some(&path)).expect("load config");

        assert_eq!(loaded.source.as_deref(), Some(path.as_path()));
        assert!(
            loaded
                .config
                .report
                .dir
                .to_string_lossy()
                .contains("custom-reports")
        );
        assert_eq!(loaded.config.tools.conda.envs, vec!["base", "ml"]);
        assert_eq!(loaded.config.docker.compose_projects.len(), 1);
    }

    #[test]
    fn rejects_invalid_config_toml() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("invalid.toml");
        fs::write(&path, "[report\n").expect("write invalid config");

        let err = load_config(Some(&path)).unwrap_err().to_string();
        assert!(err.contains("INFRA_CONFIG_PARSE"));
    }
}
