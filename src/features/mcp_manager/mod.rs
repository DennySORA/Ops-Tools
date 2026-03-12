mod config;
mod executor;
mod tools;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use executor::McpExecutor;
use std::collections::HashMap;
use tools::{get_available_tools, CliType, McpTool, McpToolOptions};

/// 執行 MCP 管理功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::MCP_MANAGER_HEADER));

    // 選擇 CLI 類型
    let cli_options = ["Anthropic Claude", "OpenAI Codex", "Google Gemini"];
    let cli_selection = prompts.select(i18n::t(keys::MCP_MANAGER_SELECT_CLI), &cli_options);

    let cli = match cli_selection {
        Some(0) => CliType::Claude,
        Some(1) => CliType::Codex,
        Some(2) => CliType::Gemini,
        _ => {
            console.warning(i18n::t(keys::MCP_MANAGER_CANCELLED));
            return;
        }
    };

    console.blank_line();
    console.info(&crate::tr!(
        keys::MCP_MANAGER_USING_CLI,
        cli = cli.display_name()
    ));

    let executor = McpExecutor::new(cli);

    // 掃描已安裝的 MCP
    console.info(i18n::t(keys::MCP_MANAGER_SCANNING));
    let installed = executor.list_installed().unwrap_or_default();

    if installed.is_empty() {
        console.warning(i18n::t(keys::MCP_MANAGER_NONE_INSTALLED));
    } else {
        console.success(&crate::tr!(
            keys::MCP_MANAGER_FOUND_INSTALLED,
            count = installed.len()
        ));
        for name in &installed {
            console.list_item("✓", name);
        }
    }

    console.blank_line();
    console.separator();

    // 顯示可用工具
    let available_tools = get_available_tools(cli);
    let items: Vec<String> = available_tools
        .iter()
        .map(|mcp| {
            let status = if installed.contains(&mcp.name.to_string()) {
                i18n::t(keys::MCP_MANAGER_STATUS_INSTALLED)
            } else {
                i18n::t(keys::MCP_MANAGER_STATUS_MISSING)
            };
            format!("{} {}", status, mcp.display_name())
        })
        .collect();

    let defaults: Vec<bool> = available_tools
        .iter()
        .map(|mcp| installed.contains(&mcp.name.to_string()))
        .collect();

    console.blank_line();
    console.info(i18n::t(keys::MCP_MANAGER_SELECT_INSTALL));
    console.info(i18n::t(keys::MCP_MANAGER_SELECT_HELP));
    console.blank_line();

    let selections =
        prompts.multi_select(i18n::t(keys::MCP_MANAGER_SELECT_PROMPT), &items, &defaults);

    // 計算需要安裝和移除的項目
    let mut to_install: Vec<&McpTool> = Vec::new();
    let mut to_remove: Vec<&McpTool> = Vec::new();

    for (i, mcp) in available_tools.iter().enumerate() {
        let is_selected = selections.contains(&i);
        let is_installed = installed.contains(&mcp.name.to_string());

        if is_selected && !is_installed {
            to_install.push(mcp);
        } else if !is_selected && is_installed {
            to_remove.push(mcp);
        }
    }

    if to_install.is_empty() && to_remove.is_empty() {
        console.blank_line();
        console.success(i18n::t(keys::MCP_MANAGER_NO_CHANGES));
        return;
    }

    // 顯示變更摘要
    console.blank_line();
    console.separator();
    console.info(i18n::t(keys::MCP_MANAGER_CHANGE_SUMMARY));

    if !to_install.is_empty() {
        console.success(i18n::t(keys::MCP_MANAGER_WILL_INSTALL));
        for mcp in &to_install {
            console.list_item("➕", mcp.display_name());
        }
    }

    if !to_remove.is_empty() {
        console.warning(i18n::t(keys::MCP_MANAGER_WILL_REMOVE));
        for mcp in &to_remove {
            console.list_item("➖", mcp.display_name());
        }
    }

    console.blank_line();
    if !prompts.confirm(i18n::t(keys::MCP_MANAGER_CONFIRM_CHANGES)) {
        console.warning(i18n::t(keys::MCP_MANAGER_CANCELLED));
        return;
    }

    console.blank_line();

    // 為有選項的工具收集配置
    let mut tool_options: HashMap<&str, McpToolOptions> = HashMap::new();
    for mcp in &to_install {
        if mcp.has_options && mcp.name == "chrome-devtools" {
            console.info(&crate::tr!(
                keys::MCP_MANAGER_CONFIGURE_TOOL,
                tool = mcp.display_name()
            ));
            let headless_options = [
                i18n::t(keys::MCP_MANAGER_CHROME_HEADLESS_YES),
                i18n::t(keys::MCP_MANAGER_CHROME_HEADLESS_NO),
            ];
            let selection = prompts.select(
                i18n::t(keys::MCP_MANAGER_CHROME_HEADLESS_PROMPT),
                &headless_options,
            );
            let headless = match selection {
                Some(0) => true,
                Some(1) => false,
                _ => true, // 預設使用 headless
            };
            tool_options.insert(
                mcp.name,
                McpToolOptions {
                    headless: Some(headless),
                },
            );
            console.blank_line();
        }
    }

    if to_install.iter().any(|mcp| mcp.requires_interactive) {
        console.info(i18n::t(keys::MCP_MANAGER_OAUTH_HINT));
        console.info(i18n::t(keys::MCP_MANAGER_WSL_HINT));
        console.blank_line();
    }

    // 執行安裝和移除
    let mut success_count = 0;
    let mut failed_count = 0;
    let total_operations = to_install.len() + to_remove.len();

    for (i, mcp) in to_install.iter().enumerate() {
        console.show_progress(
            i + 1,
            total_operations,
            &crate::tr!(keys::MCP_MANAGER_INSTALLING, tool = mcp.display_name()),
        );

        let options = tool_options.get(mcp.name).cloned().unwrap_or_default();
        match executor.install(mcp, &options) {
            Ok(()) => {
                console.success_item(&crate::tr!(
                    keys::MCP_MANAGER_INSTALL_SUCCESS,
                    tool = mcp.display_name()
                ));
                success_count += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(keys::MCP_MANAGER_INSTALL_FAILED, tool = mcp.display_name()),
                    &err.to_string(),
                );
                failed_count += 1;
            }
        }
    }

    for (i, mcp) in to_remove.iter().enumerate() {
        console.show_progress(
            to_install.len() + i + 1,
            total_operations,
            &crate::tr!(keys::MCP_MANAGER_REMOVING, tool = mcp.display_name()),
        );

        match executor.remove(mcp.name) {
            Ok(()) => {
                console.success_item(&crate::tr!(
                    keys::MCP_MANAGER_REMOVE_SUCCESS,
                    tool = mcp.display_name()
                ));
                success_count += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(keys::MCP_MANAGER_REMOVE_FAILED, tool = mcp.display_name()),
                    &err.to_string(),
                );
                failed_count += 1;
            }
        }
    }

    console.show_summary(
        i18n::t(keys::MCP_MANAGER_SUMMARY),
        success_count,
        failed_count,
    );
}

#[cfg(test)]
mod tests {
    use super::tools::{get_available_tools, CliType};

    #[test]
    fn test_tools_available() {
        let tools = get_available_tools(CliType::Claude);
        assert!(!tools.is_empty());
    }
}
