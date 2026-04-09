use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    program: String,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    env: BTreeMap<String, String>,
    timeout_ms: Option<u64>,
    retry_limit: u8,
    sudo: bool,
}

impl CommandSpec {
    pub fn new(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            cwd: None,
            env: BTreeMap::new(),
            timeout_ms: None,
            retry_limit: 0,
            sudo: false,
        }
    }

    pub fn with_cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_ms = Some(timeout_secs.saturating_mul(1000));
        self
    }

    pub fn with_retry_limit(mut self, retry_limit: u8) -> Self {
        self.retry_limit = retry_limit;
        self
    }

    pub fn with_sudo(mut self) -> Self {
        self.sudo = true;
        self
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn cwd(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    pub fn env(&self) -> &BTreeMap<String, String> {
        &self.env
    }

    pub fn timeout_ms(&self) -> Option<u64> {
        self.timeout_ms
    }

    pub fn retry_limit(&self) -> u8 {
        self.retry_limit
    }

    pub fn sudo(&self) -> bool {
        self.sudo
    }

    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.sudo {
            parts.push("sudo".to_string());
        }
        parts.push(self.program.clone());
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }
}
