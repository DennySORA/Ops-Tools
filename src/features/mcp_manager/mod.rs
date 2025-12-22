mod config;
mod executor;
mod tools;

use crate::ui::{Console, Prompts};
use config::ENV_CONFIG;
use executor::McpExecutor;
use tools::{get_available_tools, CliType, McpTool};

/// 執行 MCP 管理功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header("MCP 工具管理器");

    // 檢查環境變數
    if let Err(missing) = ENV_CONFIG.check_required() {
        console.error("此功能需要以下環境變數，請在編譯時設定：");
        for var in &missing {
            console.list_item("✗", var);
        }
        console.blank_line();
        console.info("提示：請在 .env 檔案中設定這些變數，然後重新編譯。");
        return;
    }

    // 選擇 CLI 類型
    let cli_options = ["Anthropic Claude", "OpenAI Codex"];
    let cli_selection = prompts.select("請選擇要管理的 CLI", &cli_options);

    let cli = match cli_selection {
        Some(0) => CliType::Claude,
        Some(1) => CliType::Codex,
        _ => {
            console.warning("已取消操作");
            return;
        }
    };

    console.info(&format!("\n正在使用 {} CLI...", cli.display_name()));

    let executor = McpExecutor::new(cli);

    // 掃描已安裝的 MCP
    console.info("正在掃描已安裝的 MCP...");
    let installed = executor.list_installed().unwrap_or_default();

    if installed.is_empty() {
        console.warning("目前沒有已安裝的 MCP");
    } else {
        console.success(&format!("找到 {} 個已安裝的 MCP：", installed.len()));
        for name in &installed {
            console.list_item("✓", name);
        }
    }

    console.blank_line();
    console.separator();

    // 顯示可用工具
    let available_tools = get_available_tools();
    let items: Vec<String> = available_tools
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

    let defaults: Vec<bool> = available_tools
        .iter()
        .map(|mcp| installed.contains(&mcp.name.to_string()))
        .collect();

    console.info("\n請選擇要安裝的 MCP（已勾選的會保留，取消勾選會移除）：");
    console.info("使用空白鍵勾選/取消，Enter 確認\n");

    let selections = prompts.multi_select("選擇 MCP 工具", &items, &defaults);

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
        console.success("\n沒有需要變更的項目");
        return;
    }

    // 顯示變更摘要
    console.blank_line();
    console.separator();
    console.info("\n變更摘要：");

    if !to_install.is_empty() {
        console.success("將安裝：");
        for mcp in &to_install {
            console.list_item("➕", mcp.display_name);
        }
    }

    if !to_remove.is_empty() {
        console.warning("將移除：");
        for mcp in &to_remove {
            console.list_item("➖", mcp.display_name);
        }
    }

    console.blank_line();
    if !prompts.confirm("確定要執行這些變更嗎？") {
        console.warning("已取消操作");
        return;
    }

    console.blank_line();

    // 執行安裝和移除
    let mut success_count = 0;
    let mut failed_count = 0;
    let total_operations = to_install.len() + to_remove.len();

    for (i, mcp) in to_install.iter().enumerate() {
        console.show_progress(
            i + 1,
            total_operations,
            &format!("正在安裝 {}...", mcp.display_name),
        );

        match executor.install(mcp) {
            Ok(()) => {
                console.success_item(&format!("{} 安裝成功", mcp.display_name));
                success_count += 1;
            }
            Err(err) => {
                console.error_item(&format!("{} 安裝失敗", mcp.display_name), &err.to_string());
                failed_count += 1;
            }
        }
    }

    for (i, mcp) in to_remove.iter().enumerate() {
        console.show_progress(
            to_install.len() + i + 1,
            total_operations,
            &format!("正在移除 {}...", mcp.display_name),
        );

        match executor.remove(mcp.name) {
            Ok(()) => {
                console.success_item(&format!("{} 移除成功", mcp.display_name));
                success_count += 1;
            }
            Err(err) => {
                console.error_item(&format!("{} 移除失敗", mcp.display_name), &err.to_string());
                failed_count += 1;
            }
        }
    }

    console.show_summary("MCP 管理完成", success_count, failed_count);
}

#[cfg(test)]
mod tests {
    use super::tools::get_available_tools;

    #[test]
    fn test_tools_available() {
        let tools = get_available_tools();
        assert!(!tools.is_empty());
    }
}
