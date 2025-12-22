/// 編譯時環境變數
pub struct EnvConfig {
    pub github_token: Option<&'static str>,
    pub github_host: Option<&'static str>,
    pub context7_api_key: Option<&'static str>,
}

impl EnvConfig {
    pub const fn new() -> Self {
        Self {
            github_token: option_env!("GITHUB_PERSONAL_ACCESS_TOKEN"),
            github_host: option_env!("GITHUB_HOST"),
            context7_api_key: option_env!("CONTEXT7_API_KEY"),
        }
    }

    /// 檢查必要的環境變數是否已設定
    pub fn check_required(&self) -> Result<(), Vec<&'static str>> {
        let mut missing = Vec::new();

        if self.github_token.is_none() {
            missing.push("GITHUB_PERSONAL_ACCESS_TOKEN");
        }
        if self.github_host.is_none() {
            missing.push("GITHUB_HOST");
        }
        if self.context7_api_key.is_none() {
            missing.push("CONTEXT7_API_KEY");
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 全域配置實例
pub static ENV_CONFIG: EnvConfig = EnvConfig::new();
