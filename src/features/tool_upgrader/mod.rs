mod tools;
mod upgrader;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use tools::AI_TOOLS;
use upgrader::PackageUpgrader;

/// 執行 AI 工具升級功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::TOOL_UPGRADER_HEADER));

    console.info(i18n::t(keys::TOOL_UPGRADER_LIST_TITLE));
    for tool in AI_TOOLS {
        console.list_item("📦", &format!("{} ({})", tool.name, tool.display));
    }
    console.separator();

    if !prompts.confirm(i18n::t(keys::TOOL_UPGRADER_CONFIRM)) {
        console.warning(i18n::t(keys::TOOL_UPGRADER_CANCELLED));
        return;
    }

    console.blank_line();

    let upgrader = PackageUpgrader::new();
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, tool) in AI_TOOLS.iter().enumerate() {
        console.show_progress(
            i + 1,
            AI_TOOLS.len(),
            &crate::tr!(keys::TOOL_UPGRADER_PROGRESS, tool = tool.name),
        );

        match upgrader.upgrade(tool) {
            Ok(output) => {
                console.success_item(&crate::tr!(keys::TOOL_UPGRADER_SUCCESS, tool = tool.name));
                if !output.trim().is_empty() {
                    for line in output.lines().take(3) {
                        console.list_item("  ", line);
                    }
                }
                success_count += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(keys::TOOL_UPGRADER_FAILED, tool = tool.name),
                    &err.to_string(),
                );
                failed_count += 1;
            }
        }
        console.blank_line();
    }

    console.show_summary(
        i18n::t(keys::TOOL_UPGRADER_SUMMARY),
        success_count,
        failed_count,
    );
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
