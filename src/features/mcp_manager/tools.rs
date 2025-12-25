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
    Gemini,
}

impl CliType {
    pub fn command(&self) -> &'static str {
        match self {
            CliType::Claude => "claude",
            CliType::Codex => "codex",
            CliType::Gemini => "gemini",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CliType::Claude => "Anthropic Claude",
            CliType::Codex => "OpenAI Codex",
            CliType::Gemini => "Google Gemini",
        }
    }
}

/// 取得可用的 MCP 工具清單
pub fn get_available_tools(cli_type: CliType) -> Vec<McpTool> {
    let separator = if cli_type == CliType::Gemini {
        None
    } else {
        Some("--")
    };

    let mut tools = vec![
        McpTool {
            name: "sequential-thinking",
            display_name: "Sequential Thinking (循序思考)",
            install_args: {
                let mut args = vec!["sequential-thinking".to_string()];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "npx".to_string(),
                    "-y".to_string(),
                    "@modelcontextprotocol/server-sequential-thinking".to_string(),
                ]);
                args
            },
        },
        McpTool {
            name: "chrome-devtools",
            display_name: "Chrome DevTools (瀏覽器開發工具)",
            install_args: {
                let mut args = vec!["chrome-devtools".to_string()];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "npx".to_string(),
                    "chrome-devtools-mcp@latest".to_string(),
                    "--isolated=true".to_string(),
                ]);
                args
            },
        },
        McpTool {
            name: "kubernetes",
            display_name: "Kubernetes (K8s 管理)",
            install_args: {
                let mut args = vec!["kubernetes".to_string()];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "npx".to_string(),
                    "-y".to_string(),
                    "kubernetes-mcp-server@latest".to_string(),
                ]);
                args
            },
        },
    ];

    // 只有在環境變數存在時才加入特定工具
    if let Some(key) = ENV_CONFIG.context7_api_key {
        let context7_args = match cli_type {
            CliType::Claude => vec![
                "--transport".to_string(),
                "http".to_string(),
                "context7".to_string(),
                "https://mcp.context7.com/mcp".to_string(),
                "--header".to_string(),
                format!("CONTEXT7_API_KEY: {}", key),
            ],
            CliType::Codex => vec![
                "context7".to_string(),
                "--url".to_string(),
                "https://mcp.context7.com/mcp".to_string(),
                "--bearer-token-env-var".to_string(),
                "CONTEXT7_API_KEY".to_string(),
            ],
            CliType::Gemini => vec![
                "context7".to_string(),
                "https://mcp.context7.com/mcp".to_string(),
                "--transport".to_string(),
                "http".to_string(),
                "--header".to_string(),
                format!("CONTEXT7_API_KEY: {}", key),
            ],
        };
        tools.push(McpTool {
            name: "context7",
            display_name: "Context7 (文檔查詢)",
            install_args: context7_args,
        });
    }

    if let (Some(token), Some(host)) = (ENV_CONFIG.github_token, ENV_CONFIG.github_host) {
        tools.push(McpTool {
            name: "github",
            display_name: "GitHub (GitHub 整合)",
            install_args: {
                let mut args = vec![
                    "github".to_string(),
                    "--env".to_string(),
                    format!("GITHUB_PERSONAL_ACCESS_TOKEN={}", token),
                    "--env".to_string(),
                    format!("GITHUB_HOST={}", host),
                ];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "docker".to_string(),
                    "run".to_string(),
                    "-i".to_string(),
                    "--rm".to_string(),
                    "-e".to_string(),
                    "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                    "-e".to_string(),
                    "GITHUB_HOST".to_string(),
                    "ghcr.io/github/github-mcp-server".to_string(),
                ]);
                args
            },
        });
    }

    tools
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
        let tools = get_available_tools(CliType::Claude);
        assert!(!tools.is_empty());
    }

    #[test]
    fn test_gemini_tools_no_separator() {
        let tools = get_available_tools(CliType::Gemini);
        for tool in tools {
            if tool.name == "sequential-thinking" {
                // Ensure the second argument (index 1) is NOT "--"
                // install_args[0] is name
                // install_args[1] should be "npx"
                assert_eq!(tool.install_args[1], "npx");
            }
        }
    }
}
