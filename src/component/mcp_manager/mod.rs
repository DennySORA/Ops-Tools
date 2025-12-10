use std::process::Command;

use crate::tools::ui::UserInterface;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};

// Compile-time environment variables from .env
const GITHUB_PERSONAL_ACCESS_TOKEN: &str = env!("GITHUB_PERSONAL_ACCESS_TOKEN");
const GITHUB_HOST: &str = env!("GITHUB_HOST");
const CONFLUENCE_URL: &str = env!("CONFLUENCE_URL");
const CONFLUENCE_USERNAME: &str = env!("CONFLUENCE_USERNAME");
const CONFLUENCE_API_TOKEN: &str = env!("CONFLUENCE_API_TOKEN");
const JIRA_URL: &str = env!("JIRA_URL");
const JIRA_USERNAME: &str = env!("JIRA_USERNAME");
const JIRA_API_TOKEN: &str = env!("JIRA_API_TOKEN");
const CONTEXT7_API_KEY: &str = env!("CONTEXT7_API_KEY");

/// MCP 工具的定義
#[derive(Clone)]
struct McpTool {
    name: &'static str,
    display_name: &'static str,
    /// 支援的 CLI 類型
    supported_cli: SupportedCli,
    /// 安裝指令的參數（不包含 mcp add）- 使用 String 支援動態值
    install_args: Vec<String>,
}

/// 支援的 CLI 類型
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum SupportedCli {
    Both,
    ClaudeOnly,
    CodexOnly,
}

impl SupportedCli {
    fn supports(&self, cli: CliType) -> bool {
        match self {
            SupportedCli::Both => true,
            SupportedCli::ClaudeOnly => cli == CliType::Claude,
            SupportedCli::CodexOnly => cli == CliType::Codex,
        }
    }
}

/// 預設可用的 MCP 工具清單
fn get_available_mcps() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "sequential-thinking",
            display_name: "Sequential Thinking (循序思考)",
            supported_cli: SupportedCli::Both,
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
            supported_cli: SupportedCli::Both,
            install_args: vec![
                "--transport".to_string(),
                "http".to_string(),
                "context7".to_string(),
                "https://mcp.context7.com/mcp".to_string(),
                "--header".to_string(),
                format!("CONTEXT7_API_KEY: {}", CONTEXT7_API_KEY),
            ],
        },
        McpTool {
            name: "chrome-devtools",
            display_name: "Chrome DevTools (瀏覽器開發工具)",
            supported_cli: SupportedCli::Both,
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
            supported_cli: SupportedCli::Both,
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
            supported_cli: SupportedCli::Both,
            install_args: vec![
                "github".to_string(),
                "--env".to_string(),
                format!(
                    "GITHUB_PERSONAL_ACCESS_TOKEN={}",
                    GITHUB_PERSONAL_ACCESS_TOKEN
                ),
                "--env".to_string(),
                format!("GITHUB_HOST={}", GITHUB_HOST),
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
        McpTool {
            name: "mcp-atlassian",
            display_name: "Atlassian (Confluence & Jira)",
            supported_cli: SupportedCli::Both,
            install_args: vec![
                "mcp-atlassian".to_string(),
                "--env".to_string(),
                format!("CONFLUENCE_URL={}", CONFLUENCE_URL),
                "--env".to_string(),
                format!("CONFLUENCE_USERNAME={}", CONFLUENCE_USERNAME),
                "--env".to_string(),
                format!("CONFLUENCE_API_TOKEN={}", CONFLUENCE_API_TOKEN),
                "--env".to_string(),
                format!("JIRA_URL={}", JIRA_URL),
                "--env".to_string(),
                format!("JIRA_USERNAME={}", JIRA_USERNAME),
                "--env".to_string(),
                format!("JIRA_API_TOKEN={}", JIRA_API_TOKEN),
                "--".to_string(),
                "docker".to_string(),
                "run".to_string(),
                "-i".to_string(),
                "--rm".to_string(),
                "-e".to_string(),
                "CONFLUENCE_URL".to_string(),
                "-e".to_string(),
                "CONFLUENCE_USERNAME".to_string(),
                "-e".to_string(),
                "CONFLUENCE_API_TOKEN".to_string(),
                "-e".to_string(),
                "JIRA_URL".to_string(),
                "-e".to_string(),
                "JIRA_USERNAME".to_string(),
                "-e".to_string(),
                "JIRA_API_TOKEN".to_string(),
                "ghcr.io/sooperset/mcp-atlassian:latest".to_string(),
            ],
        },
    ]
}

/// CLI 類型
#[derive(Clone, Copy, PartialEq)]
enum CliType {
    Codex,
    Claude,
}

impl CliType {
    fn command(&self) -> &'static str {
        match self {
            CliType::Codex => "codex",
            CliType::Claude => "claude",
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            CliType::Codex => "OpenAI Codex",
            CliType::Claude => "Anthropic Claude",
        }
    }
}

/// 取得已安裝的 MCP 清單
fn get_installed_mcps(cli: CliType) -> Vec<String> {
    let output = Command::new(cli.command()).args(["mcp", "list"]).output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_mcp_list(&stdout)
        }
        _ => Vec::new(),
    }
}

/// 解析 mcp list 的輸出，提取 MCP 名稱
fn parse_mcp_list(output: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        // 跳過空行和標題行
        if trimmed.is_empty() || trimmed.starts_with("MCP") || trimmed.starts_with("---") {
            continue;
        }
        // 提取 MCP 名稱（通常是第一個欄位）
        if let Some(name) = trimmed.split_whitespace().next() {
            // 過濾掉可能的裝飾字符
            let clean_name =
                name.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
            if !clean_name.is_empty() {
                names.push(clean_name.to_string());
            }
        }
    }

    names
}

/// 安裝 MCP
fn install_mcp(cli: CliType, mcp: &McpTool) -> Result<String, String> {
    let mut args: Vec<&str> = vec!["mcp", "add"];
    let string_refs: Vec<&str> = mcp.install_args.iter().map(|s| s.as_str()).collect();
    args.extend(string_refs);

    let output = Command::new(cli.command())
        .args(&args)
        .output()
        .map_err(|e| format!("無法執行 {}: {}", cli.command(), e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(stderr.lines().next().unwrap_or("未知錯誤").to_string())
    }
}

/// 移除 MCP
fn remove_mcp(cli: CliType, mcp_name: &str) -> Result<String, String> {
    let output = Command::new(cli.command())
        .args(["mcp", "remove", mcp_name])
        .output()
        .map_err(|e| format!("無法執行 {}: {}", cli.command(), e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(stderr.lines().next().unwrap_or("未知錯誤").to_string())
    }
}

/// MCP 管理器主函數
pub fn manage_mcp() {
    let ui = UserInterface::new();
    ui.header("MCP 工具管理器");

    // 步驟 1: 選擇 CLI 類型
    let cli_options = vec!["Anthropic Claude", "OpenAI Codex"];
    let cli_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("請選擇要管理的 CLI")
        .items(&cli_options)
        .default(0)
        .interact();

    let cli = match cli_selection {
        Ok(0) => CliType::Claude,
        Ok(1) => CliType::Codex,
        _ => {
            ui.warning("已取消操作");
            return;
        }
    };

    ui.info(&format!("\n正在使用 {} CLI...", cli.display_name()));

    // 步驟 2: 掃描已安裝的 MCP
    ui.info("正在掃描已安裝的 MCP...");
    let installed = get_installed_mcps(cli);

    if installed.is_empty() {
        ui.warning("目前沒有已安裝的 MCP");
    } else {
        ui.success(&format!("找到 {} 個已安裝的 MCP：", installed.len()));
        for name in &installed {
            ui.list_item("✓", name);
        }
    }

    println!();
    ui.separator();

    // 步驟 3: 過濾出支援當前 CLI 的 MCP
    let all_mcps = get_available_mcps();
    let available_mcps: Vec<McpTool> = all_mcps
        .into_iter()
        .filter(|mcp| mcp.supported_cli.supports(cli))
        .collect();

    let items: Vec<String> = available_mcps
        .iter()
        .map(|mcp| {
            let status = if installed.contains(&mcp.name.to_string()) {
                "[已安裝]"
            } else {
                "[未安裝]"
            };
            format!("{} {}", status, mcp.display_name)
        })
        .collect();

    // 預設選擇已安裝的項目
    let defaults: Vec<bool> = available_mcps
        .iter()
        .map(|mcp| installed.contains(&mcp.name.to_string()))
        .collect();

    ui.info("\n請選擇要安裝的 MCP（已勾選的會保留，取消勾選會移除）：");
    ui.info("使用空白鍵勾選/取消，Enter 確認\n");

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("選擇 MCP 工具")
        .items(&items)
        .defaults(&defaults)
        .interact();

    let selected_indices = match selections {
        Ok(indices) => indices,
        Err(_) => {
            ui.warning("已取消操作");
            return;
        }
    };

    // 步驟 4: 計算需要安裝和移除的項目
    let mut to_install: Vec<&McpTool> = Vec::new();
    let mut to_remove: Vec<&McpTool> = Vec::new();

    for (i, mcp) in available_mcps.iter().enumerate() {
        let is_selected = selected_indices.contains(&i);
        let is_installed = installed.contains(&mcp.name.to_string());

        if is_selected && !is_installed {
            to_install.push(mcp);
        } else if !is_selected && is_installed {
            to_remove.push(mcp);
        }
    }

    // 顯示變更摘要
    if to_install.is_empty() && to_remove.is_empty() {
        ui.success("\n沒有需要變更的項目");
        return;
    }

    println!();
    ui.separator();
    ui.info("\n變更摘要：");

    if !to_install.is_empty() {
        ui.success("將安裝：");
        for mcp in &to_install {
            ui.list_item("➕", mcp.display_name);
        }
    }

    if !to_remove.is_empty() {
        ui.warning("將移除：");
        for mcp in &to_remove {
            ui.list_item("➖", mcp.display_name);
        }
    }

    println!();
    if !ui.confirm("確定要執行這些變更嗎？") {
        ui.warning("已取消操作");
        return;
    }

    println!();

    // 步驟 5: 執行安裝和移除
    let mut success_count = 0;
    let mut failed_count = 0;
    let total_operations = to_install.len() + to_remove.len();

    // 安裝新的 MCP
    for (i, mcp) in to_install.iter().enumerate() {
        ui.show_progress(
            i + 1,
            total_operations,
            &format!("正在安裝 {}...", mcp.display_name),
        );

        match install_mcp(cli, mcp) {
            Ok(_) => {
                ui.success_item(&format!("{} 安裝成功", mcp.display_name));
                success_count += 1;
            }
            Err(err) => {
                ui.error_item(&format!("{} 安裝失敗", mcp.display_name), &err);
                failed_count += 1;
            }
        }
    }

    // 移除 MCP
    for (i, mcp) in to_remove.iter().enumerate() {
        ui.show_progress(
            to_install.len() + i + 1,
            total_operations,
            &format!("正在移除 {}...", mcp.display_name),
        );

        match remove_mcp(cli, mcp.name) {
            Ok(_) => {
                ui.success_item(&format!("{} 移除成功", mcp.display_name));
                success_count += 1;
            }
            Err(err) => {
                ui.error_item(&format!("{} 移除失敗", mcp.display_name), &err);
                failed_count += 1;
            }
        }
    }

    ui.show_summary("MCP 管理完成", success_count, failed_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mcp_list_empty() {
        let output = "";
        let result = parse_mcp_list(output);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_mcp_list_with_names() {
        let output = "MCP Servers\n---\nsequential-thinking  running\ncontext7  running";
        let result = parse_mcp_list(output);
        assert!(result.contains(&"sequential-thinking".to_string()));
        assert!(result.contains(&"context7".to_string()));
    }

    #[test]
    fn test_available_mcps_not_empty() {
        let mcps = get_available_mcps();
        assert!(!mcps.is_empty());
    }

    #[test]
    fn test_cli_type_command() {
        assert_eq!(CliType::Claude.command(), "claude");
        assert_eq!(CliType::Codex.command(), "codex");
    }

    #[test]
    fn test_supported_cli_filter() {
        let mcps = get_available_mcps();

        // GitHub should support both
        let github = mcps.iter().find(|m| m.name == "github").unwrap();
        assert!(github.supported_cli.supports(CliType::Claude));
        assert!(github.supported_cli.supports(CliType::Codex));

        // Atlassian should support both
        let atlassian = mcps.iter().find(|m| m.name == "mcp-atlassian").unwrap();
        assert!(atlassian.supported_cli.supports(CliType::Claude));
        assert!(atlassian.supported_cli.supports(CliType::Codex));

        // Sequential thinking should support both
        let seq = mcps
            .iter()
            .find(|m| m.name == "sequential-thinking")
            .unwrap();
        assert!(seq.supported_cli.supports(CliType::Claude));
        assert!(seq.supported_cli.supports(CliType::Codex));
    }
}
