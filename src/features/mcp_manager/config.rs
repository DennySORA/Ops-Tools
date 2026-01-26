/// 編譯時環境變數
pub struct EnvConfig {
    pub github_token: Option<&'static str>,
    pub github_host: Option<&'static str>,
    pub github_toolsets: Option<&'static str>,
    pub github_mcp_mode: Option<&'static str>,
    pub context7_api_key: Option<&'static str>,
    pub enable_cloudflare_mcp_raw: Option<&'static str>,
    pub arxiv_storage_path: Option<&'static str>,
}

impl EnvConfig {
    pub const fn new() -> Self {
        Self {
            github_token: option_env!("GITHUB_PERSONAL_ACCESS_TOKEN"),
            github_host: option_env!("GITHUB_HOST"),
            github_toolsets: option_env!("GITHUB_TOOLSETS"),
            github_mcp_mode: option_env!("GITHUB_MCP_MODE"),
            context7_api_key: option_env!("CONTEXT7_API_KEY"),
            enable_cloudflare_mcp_raw: first_env(
                option_env!("enable_cloudflare_mcp"),
                option_env!("ENABLE_CLOUDFLARE_MCP"),
            ),
            arxiv_storage_path: option_env!("ARXIV_STORAGE_PATH"),
        }
    }

    /// 取得 GitHub MCP 模式（remote 或 docker）
    pub fn github_mcp_mode_value(&self) -> &'static str {
        match self.github_mcp_mode {
            Some(mode) if mode.eq_ignore_ascii_case("docker") => "docker",
            _ => "remote",
        }
    }

    pub fn enable_cloudflare_mcp(&self) -> bool {
        parse_bool_env(self.enable_cloudflare_mcp_raw)
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 全域配置實例
pub static ENV_CONFIG: EnvConfig = EnvConfig::new();

const fn first_env(
    primary: Option<&'static str>,
    fallback: Option<&'static str>,
) -> Option<&'static str> {
    match primary {
        Some(value) => Some(value),
        None => fallback,
    }
}

fn parse_bool_env(value: Option<&str>) -> bool {
    let Some(raw) = value else {
        return false;
    };
    let trimmed = raw.trim();
    trimmed == "1" || trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("yes")
}
