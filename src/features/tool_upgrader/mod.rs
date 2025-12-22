mod tools;
mod upgrader;

use crate::ui::{Console, Prompts};
use tools::AI_TOOLS;
use upgrader::PackageUpgrader;

/// 執行 AI 工具升級功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header("升級 AI 程式碼助手工具");

    console.info("將升級以下工具：");
    for tool in AI_TOOLS {
        console.list_item("📦", &format!("{} ({})", tool.name, tool.package));
    }
    console.separator();

    if !prompts.confirm("確定要升級這些工具嗎？") {
        console.warning("已取消升級");
        return;
    }

    console.blank_line();

    let upgrader = PackageUpgrader::new();
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, tool) in AI_TOOLS.iter().enumerate() {
        console.show_progress(i + 1, AI_TOOLS.len(), &format!("正在升級 {}...", tool.name));

        match upgrader.upgrade(tool.package) {
            Ok(output) => {
                console.success_item(&format!("{} 升級成功", tool.name));
                if !output.trim().is_empty() {
                    for line in output.lines().take(3) {
                        console.list_item("  ", line);
                    }
                }
                success_count += 1;
            }
            Err(err) => {
                console.error_item(&format!("{} 升級失敗", tool.name), &err.to_string());
                failed_count += 1;
            }
        }
        console.blank_line();
    }

    console.show_summary("升級完成", success_count, failed_count);
}

#[cfg(test)]
mod tests {
    use super::tools::AI_TOOLS;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_ai_tools_list() {
        assert!(!AI_TOOLS.is_empty());
    }
}
