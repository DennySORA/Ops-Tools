use super::config::ENV_CONFIG;

/// MCP 工具定義
#[derive(Clone)]
pub struct McpTool {
    pub name: &'static str,
    pub display_name: &'static str,
    pub install_args: Vec<String>,
}

/// CLI 類型
#[derive(Clone, Copy, PartialEq)]
pub enum CliType {
    Claude,
    Codex,
}

impl CliType {
    pub fn command(&self) -> &'static str {
        match self {
            CliType::Claude => "claude",
            CliType::Codex => "codex",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CliType::Claude => "Anthropic Claude",
            CliType::Codex => "OpenAI Codex",
        }
    }
}

/// 取得可用的 MCP 工具清單
pub fn get_available_tools() -> Vec<McpTool> {
    let github_token = ENV_CONFIG.github_token.unwrap_or("");
    let github_host = ENV_CONFIG.github_host.unwrap_or("");
    let context7_api_key = ENV_CONFIG.context7_api_key.unwrap_or("");

    vec![
        McpTool {
            name: "sequential-thinking",
            display_name: "Sequential Thinking (循序思考)",
            install_args: vec![
                "sequential-thinking".to_string(),
                "--".to_string(),
                "npx".to_string(),
                "-y".to_string(),
                "@modelcontextprotocol/server-sequential-thinking".to_string(),
            ],
        },
        McpTool {
            name: "context7",
            display_name: "Context7 (文檔查詢)",
            install_args: vec![
                "--transport".to_string(),
                "http".to_string(),
                "context7".to_string(),
                "https://mcp.context7.com/mcp".to_string(),
                "--header".to_string(),
                format!("CONTEXT7_API_KEY: {}", context7_api_key),
            ],
        },
        McpTool {
            name: "chrome-devtools",
            display_name: "Chrome DevTools (瀏覽器開發工具)",
            install_args: vec![
                "chrome-devtools".to_string(),
                "--".to_string(),
                "npx".to_string(),
                "chrome-devtools-mcp@latest".to_string(),
                "--isolated=true".to_string(),
            ],
        },
        McpTool {
            name: "kubernetes",
            display_name: "Kubernetes (K8s 管理)",
            install_args: vec![
                "kubernetes".to_string(),
                "--".to_string(),
                "npx".to_string(),
                "-y".to_string(),
                "kubernetes-mcp-server@latest".to_string(),
            ],
        },
        McpTool {
            name: "github",
            display_name: "GitHub (GitHub 整合)",
            install_args: vec![
                "github".to_string(),
                "--env".to_string(),
                format!("GITHUB_PERSONAL_ACCESS_TOKEN={}", github_token),
                "--env".to_string(),
                format!("GITHUB_HOST={}", github_host),
                "--".to_string(),
                "docker".to_string(),
                "run".to_string(),
                "-i".to_string(),
                "--rm".to_string(),
                "-e".to_string(),
                "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                "-e".to_string(),
                "GITHUB_HOST".to_string(),
                "ghcr.io/github/github-mcp-server".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_type_command() {
        assert_eq!(CliType::Claude.command(), "claude");
        assert_eq!(CliType::Codex.command(), "codex");
    }

    #[test]
    fn test_available_tools_not_empty() {
        let tools = get_available_tools();
        assert!(!tools.is_empty());
    }
}
