use super::config::ENV_CONFIG;

/// MCP 工具定義
#[derive(Clone)]
pub struct McpTool {
    pub name: &'static str,
    pub display_name: &'static str,
    pub install_args: Vec<String>,
    pub requires_interactive: bool,
}

#[derive(Clone, Copy)]
pub struct CloudflareTool {
    pub name: &'static str,
    pub display_name: &'static str,
    pub url: &'static str,
}

const CLOUDFLARE_TOOLS: &[CloudflareTool] = &[
    CloudflareTool {
        name: "cloudflare-docs",
        display_name: "Cloudflare Docs (文件查詢)",
        url: "https://docs.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-workers-bindings",
        display_name: "Cloudflare Workers Bindings",
        url: "https://bindings.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-workers-builds",
        display_name: "Cloudflare Workers Builds",
        url: "https://builds.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-observability",
        display_name: "Cloudflare Observability",
        url: "https://observability.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-radar",
        display_name: "Cloudflare Radar (網路趨勢)",
        url: "https://radar.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-containers",
        display_name: "Cloudflare Containers (Sandbox)",
        url: "https://containers.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-browser",
        display_name: "Cloudflare Browser Rendering",
        url: "https://browser.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-logpush",
        display_name: "Cloudflare Logpush",
        url: "https://logs.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-ai-gateway",
        display_name: "Cloudflare AI Gateway",
        url: "https://ai-gateway.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-autorag",
        display_name: "Cloudflare AutoRAG",
        url: "https://autorag.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-auditlogs",
        display_name: "Cloudflare Audit Logs",
        url: "https://auditlogs.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-dns-analytics",
        display_name: "Cloudflare DNS Analytics",
        url: "https://dns-analytics.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-dex",
        display_name: "Cloudflare DEX",
        url: "https://dex.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-casb",
        display_name: "Cloudflare One CASB",
        url: "https://casb.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-graphql",
        display_name: "Cloudflare GraphQL",
        url: "https://graphql.mcp.cloudflare.com/mcp",
    },
];

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
            requires_interactive: false,
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
            requires_interactive: false,
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
            requires_interactive: false,
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
            requires_interactive: false,
        });
    }

    if ENV_CONFIG.enable_cloudflare_mcp() {
        for tool in CLOUDFLARE_TOOLS {
            let args = match cli_type {
                CliType::Claude => vec![
                    "--transport".to_string(),
                    "http".to_string(),
                    tool.name.to_string(),
                    tool.url.to_string(),
                ],
                CliType::Codex => vec![
                    tool.name.to_string(),
                    "--url".to_string(),
                    tool.url.to_string(),
                ],
                CliType::Gemini => vec![
                    tool.name.to_string(),
                    tool.url.to_string(),
                    "--transport".to_string(),
                    "http".to_string(),
                ],
            };
            tools.push(McpTool {
                name: tool.name,
                display_name: tool.display_name,
                install_args: args,
                requires_interactive: true,
            });
        }
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
            requires_interactive: false,
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
