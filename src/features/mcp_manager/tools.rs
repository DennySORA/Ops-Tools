use super::config::ENV_CONFIG;
use crate::i18n::{self, keys};

/// MCP 工具定義
#[derive(Clone)]
pub struct McpTool {
    pub name: &'static str,
    pub display_name_key: &'static str,
    pub install_args: Vec<String>,
    pub requires_interactive: bool,
}

impl McpTool {
    pub fn display_name(&self) -> &'static str {
        i18n::t(self.display_name_key)
    }
}

#[derive(Clone, Copy)]
pub struct CloudflareTool {
    pub name: &'static str,
    pub display_name_key: &'static str,
    pub url: &'static str,
}

const CLOUDFLARE_TOOLS: &[CloudflareTool] = &[
    CloudflareTool {
        name: "cloudflare-docs",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_DOCS,
        url: "https://docs.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-workers-bindings",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_WORKERS_BINDINGS,
        url: "https://bindings.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-workers-builds",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_WORKERS_BUILDS,
        url: "https://builds.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-observability",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_OBSERVABILITY,
        url: "https://observability.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-radar",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_RADAR,
        url: "https://radar.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-containers",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_CONTAINERS,
        url: "https://containers.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-browser",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_BROWSER,
        url: "https://browser.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-logpush",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_LOGPUSH,
        url: "https://logs.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-ai-gateway",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_AI_GATEWAY,
        url: "https://ai-gateway.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-autorag",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_AUTORAG,
        url: "https://autorag.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-auditlogs",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_AUDITLOGS,
        url: "https://auditlogs.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-dns-analytics",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_DNS_ANALYTICS,
        url: "https://dns-analytics.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-dex",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_DEX,
        url: "https://dex.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-casb",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_CASB,
        url: "https://casb.mcp.cloudflare.com/mcp",
    },
    CloudflareTool {
        name: "cloudflare-graphql",
        display_name_key: keys::MCP_TOOL_CLOUDFLARE_GRAPHQL,
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
            display_name_key: keys::MCP_TOOL_SEQUENTIAL_THINKING,
            install_args: {
                let mut args = vec!["sequential-thinking".to_string()];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "npx".to_string(),
                    "-y".to_string(),
                    "@modelcontextprotocol/server-sequential-thinking@latest".to_string(),
                ]);
                args
            },
            requires_interactive: false,
        },
        McpTool {
            name: "chrome-devtools",
            display_name_key: keys::MCP_TOOL_CHROME_DEVTOOLS,
            install_args: {
                let mut args = vec!["chrome-devtools".to_string()];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "npx".to_string(),
                    "chrome-devtools-mcp@latest".to_string(),
                    "--isolated".to_string(),
                    "--headless".to_string(),
                ]);
                args
            },
            requires_interactive: false,
        },
        McpTool {
            name: "kubernetes",
            display_name_key: keys::MCP_TOOL_KUBERNETES,
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
        McpTool {
            name: "tailwindcss",
            display_name_key: keys::MCP_TOOL_TAILWINDCSS,
            install_args: {
                let mut args = vec!["tailwindcss".to_string()];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "npx".to_string(),
                    "-y".to_string(),
                    "tailwindcss-mcp-server@latest".to_string(),
                ]);
                args
            },
            requires_interactive: false,
        },
        McpTool {
            name: "arxiv-mcp-server",
            display_name_key: keys::MCP_TOOL_ARXIV,
            install_args: {
                let storage_path = ENV_CONFIG.arxiv_storage_path.unwrap_or("~/.arxiv-papers");
                let mut args = vec!["arxiv-mcp-server".to_string()];
                if let Some(sep) = separator {
                    args.push(sep.to_string());
                }
                args.extend(vec![
                    "uv".to_string(),
                    "tool".to_string(),
                    "run".to_string(),
                    "arxiv-mcp-server".to_string(),
                    "--storage-path".to_string(),
                    storage_path.to_string(),
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
            display_name_key: keys::MCP_TOOL_CONTEXT7,
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
                display_name_key: tool.display_name_key,
                install_args: args,
                requires_interactive: true,
            });
        }
    }

    if let Some(token) = ENV_CONFIG.github_token {
        let mode = ENV_CONFIG.github_mcp_mode_value();
        let host = ENV_CONFIG.github_host.unwrap_or("github.com");

        let install_args = if mode == "remote" {
            // 遠端模式：使用 GitHub 託管的 MCP 伺服器（官方推薦）
            // https://github.com/github/github-mcp-server#remote-server-recommended
            match cli_type {
                CliType::Claude => {
                    let mut args = vec![
                        "--transport".to_string(),
                        "http".to_string(),
                        "github".to_string(),
                        "https://api.githubcopilot.com/mcp/".to_string(),
                        "--header".to_string(),
                        format!("Authorization: Bearer {}", token),
                    ];
                    // 如果不是預設的 github.com，加入 X-GitHub-Host header
                    if host != "github.com" {
                        args.push("--header".to_string());
                        args.push(format!("X-GitHub-Host: {}", host));
                    }
                    args
                }
                CliType::Codex => vec![
                    "github".to_string(),
                    "--url".to_string(),
                    "https://api.githubcopilot.com/mcp/".to_string(),
                ],
                CliType::Gemini => {
                    let mut args = vec![
                        "github".to_string(),
                        "https://api.githubcopilot.com/mcp/".to_string(),
                        "--transport".to_string(),
                        "http".to_string(),
                        "--header".to_string(),
                        format!("Authorization: Bearer {}", token),
                    ];
                    if host != "github.com" {
                        args.push("--header".to_string());
                        args.push(format!("X-GitHub-Host: {}", host));
                    }
                    args
                }
            }
        } else {
            // Docker 本地模式
            // https://github.com/github/github-mcp-server#local-server
            let mut args = vec![
                "github".to_string(),
                "--env".to_string(),
                format!("GITHUB_PERSONAL_ACCESS_TOKEN={}", token),
            ];
            // 加入 GITHUB_HOST（用於 GitHub Enterprise）
            if host != "github.com" {
                args.push("--env".to_string());
                args.push(format!("GITHUB_HOST=https://{}", host));
            }
            // 加入 GITHUB_TOOLSETS（功能集）
            if let Some(toolsets) = ENV_CONFIG.github_toolsets {
                args.push("--env".to_string());
                args.push(format!("GITHUB_TOOLSETS={}", toolsets));
            }
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
            ]);
            if host != "github.com" {
                args.push("-e".to_string());
                args.push("GITHUB_HOST".to_string());
            }
            if ENV_CONFIG.github_toolsets.is_some() {
                args.push("-e".to_string());
                args.push("GITHUB_TOOLSETS".to_string());
            }
            args.push("ghcr.io/github/github-mcp-server".to_string());
            args
        };

        tools.push(McpTool {
            name: "github",
            display_name_key: keys::MCP_TOOL_GITHUB,
            install_args,
            requires_interactive: mode == "remote",
        });
    }

    tools
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{self, keys, Language};

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

    #[test]
    fn test_display_name_uses_locale() {
        let _guard = i18n::test_lock();
        let previous = i18n::current_language();
        i18n::set_language(Language::English);

        let tools = get_available_tools(CliType::Claude);
        let tool = tools
            .iter()
            .find(|tool| tool.name == "sequential-thinking")
            .expect("Missing sequential-thinking tool");
        assert_eq!(
            tool.display_name(),
            i18n::t(keys::MCP_TOOL_SEQUENTIAL_THINKING)
        );

        i18n::set_language(previous);
    }
}
