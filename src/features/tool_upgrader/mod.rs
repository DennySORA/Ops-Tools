mod tools;
mod upgrader;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use tools::AI_TOOLS;
use upgrader::{PackageUpgrader, SourceBuildExecutor};

/// Codex source build 的固定參數
const CODEX_CARGO_PACKAGE: &str = "codex-cli";
const CODEX_BINARY_NAME: &str = "codex";

/// 執行 AI 工具升級功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::TOOL_UPGRADER_HEADER));

    // 預先偵測 Codex source path
    let codex_source_dir = SourceBuildExecutor::resolve_source_dir();

    console.info(i18n::t(keys::TOOL_UPGRADER_LIST_TITLE));
    for tool in AI_TOOLS {
        let mode = if tool.name == "OpenAI Codex" && codex_source_dir.is_some() {
            "source build"
        } else {
            tool.display
        };
        console.list_item("📦", &format!("{} ({})", tool.name, mode));
    }
    console.separator();

    if !prompts.confirm(i18n::t(keys::TOOL_UPGRADER_CONFIRM)) {
        console.warning(i18n::t(keys::TOOL_UPGRADER_CANCELLED));
        return;
    }

    console.blank_line();

    let package_upgrader = PackageUpgrader::new();
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, tool) in AI_TOOLS.iter().enumerate() {
        console.show_progress(
            i + 1,
            AI_TOOLS.len(),
            &crate::tr!(keys::TOOL_UPGRADER_PROGRESS, tool = tool.name),
        );

        // Codex: 有設 source path → source build，沒有 → 一般升級
        let result = if tool.name == "OpenAI Codex" {
            if let Some(ref source_dir) = codex_source_dir {
                SourceBuildExecutor::execute_source_build(
                    source_dir,
                    CODEX_CARGO_PACKAGE,
                    CODEX_BINARY_NAME,
                )
            } else {
                package_upgrader.upgrade(tool)
            }
        } else {
            package_upgrader.upgrade(tool)
        };

        match result {
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
