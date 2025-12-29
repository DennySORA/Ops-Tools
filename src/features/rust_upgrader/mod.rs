mod tools;
mod upgrader;

use crate::core::OperationError;
use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use tools::{REQUIRED_CARGO_TOOLS, UPGRADE_STEPS};
use upgrader::RustUpgrader;

/// 執行 Rust 專案升級功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::RUST_UPGRADER_HEADER));

    let upgrader = RustUpgrader::new();

    // 步驟 1: 檢查 Rust 環境
    console.info(i18n::t(keys::RUST_UPGRADER_CHECKING_ENV));
    match upgrader.check_rust_installed() {
        Ok(env) => {
            console.success(i18n::t(keys::RUST_UPGRADER_ENV_INSTALLED));
            console.list_item("🦀", &env.rustc_version);
            console.list_item("📦", &env.cargo_version);
            console.list_item("🔧", &env.rustup_version);
        }
        Err(err) => {
            console.error(&crate::tr!(keys::RUST_UPGRADER_ENV_MISSING, error = err));
            console.info(i18n::t(keys::RUST_UPGRADER_INSTALL_RUST_HINT));
            return;
        }
    }

    console.separator();

    // 步驟 2: 檢查必要的 cargo 工具
    console.info(i18n::t(keys::RUST_UPGRADER_CHECKING_TOOLS));
    let tool_statuses = upgrader.check_tools_status(REQUIRED_CARGO_TOOLS);

    let missing_tools: Vec<_> = tool_statuses.iter().filter(|s| !s.installed).collect();

    for status in &tool_statuses {
        let icon = if status.installed { "✓" } else { "✗" };
        let state = if status.installed {
            i18n::t(keys::RUST_UPGRADER_TOOL_INSTALLED)
        } else {
            i18n::t(keys::RUST_UPGRADER_TOOL_MISSING)
        };
        console.list_item(icon, &format!("{} ({})", status.tool.display_name, state));
    }

    console.separator();

    // 步驟 3: 安裝缺少的工具
    if !missing_tools.is_empty() {
        console.warning(&crate::tr!(
            keys::RUST_UPGRADER_MISSING_TOOLS,
            count = missing_tools.len()
        ));

        if prompts.confirm(i18n::t(keys::RUST_UPGRADER_CONFIRM_INSTALL_TOOLS)) {
            console.blank_line();
            for (i, status) in missing_tools.iter().enumerate() {
                console.show_progress(
                    i + 1,
                    missing_tools.len(),
                    &crate::tr!(
                        keys::RUST_UPGRADER_INSTALLING_TOOL,
                        tool = status.tool.display_name
                    ),
                );

                match upgrader.install_tool(&status.tool) {
                    Ok(_) => {
                        console.success_item(&crate::tr!(
                            keys::RUST_UPGRADER_INSTALL_SUCCESS,
                            tool = status.tool.display_name
                        ));
                    }
                    Err(err) => {
                        console.error_item(
                            &crate::tr!(
                                keys::RUST_UPGRADER_INSTALL_FAILED,
                                tool = status.tool.display_name
                            ),
                            &err.to_string(),
                        );
                    }
                }
            }
            console.separator();
        } else {
            console.warning(i18n::t(keys::RUST_UPGRADER_SKIP_INSTALL));
            console.separator();
        }
    } else {
        console.success(i18n::t(keys::RUST_UPGRADER_ALL_TOOLS_INSTALLED));
        console.separator();
    }

    // 步驟 4: 顯示升級步驟
    console.info(i18n::t(keys::RUST_UPGRADER_UPGRADE_STEPS));
    for step in UPGRADE_STEPS {
        let project_tag = if step.requires_project {
            i18n::t(keys::RUST_UPGRADER_REQUIRES_PROJECT_TAG)
        } else {
            ""
        };
        console.list_item(
            "📋",
            &format!(
                "{}: {}{}",
                step.name,
                i18n::t(step.description_key),
                project_tag
            ),
        );
    }

    console.separator();

    if !prompts.confirm(i18n::t(keys::RUST_UPGRADER_CONFIRM_UPGRADE)) {
        console.warning(i18n::t(keys::RUST_UPGRADER_CANCELLED));
        return;
    }

    console.blank_line();

    // 步驟 5: 執行升級
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;

    for (i, step) in UPGRADE_STEPS.iter().enumerate() {
        console.show_progress(
            i + 1,
            UPGRADE_STEPS.len(),
            &crate::tr!(keys::RUST_UPGRADER_RUNNING_STEP, step = step.name),
        );

        match upgrader.run_upgrade_step(step) {
            Ok(output) => {
                console.success_item(&crate::tr!(keys::RUST_UPGRADER_STEP_DONE, step = step.name));
                display_output(&console, &output);
                success_count += 1;
            }
            Err(OperationError::MissingCargoToml) => {
                console.warning(&crate::tr!(
                    keys::RUST_UPGRADER_STEP_SKIPPED,
                    step = step.name
                ));
                skipped_count += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(keys::RUST_UPGRADER_STEP_FAILED, step = step.name),
                    &err.to_string(),
                );
                failed_count += 1;
            }
        }
        console.blank_line();
    }

    // 步驟 6: 顯示摘要
    console.show_summary(
        i18n::t(keys::RUST_UPGRADER_SUMMARY),
        success_count,
        failed_count,
    );
    if skipped_count > 0 {
        console.info(&crate::tr!(
            keys::RUST_UPGRADER_SKIPPED_COUNT,
            count = skipped_count
        ));
    }
}

/// 顯示命令輸出（限制行數）
fn display_output(console: &Console, output: &str) {
    let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return;
    }

    let display_lines = if lines.len() > 5 {
        &lines[..5]
    } else {
        &lines[..]
    };

    for line in display_lines {
        console.list_item("  ", line);
    }

    if lines.len() > 5 {
        console.list_item(
            "  ",
            &crate::tr!(
                keys::RUST_UPGRADER_OUTPUT_MORE_LINES,
                count = lines.len() - 5
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::tools::{REQUIRED_CARGO_TOOLS, UPGRADE_STEPS};
    use crate::i18n;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_required_tools_list() {
        assert!(!REQUIRED_CARGO_TOOLS.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_upgrade_steps_list() {
        assert!(!UPGRADE_STEPS.is_empty());
    }

    #[test]
    fn test_upgrade_steps_have_descriptions() {
        for step in UPGRADE_STEPS {
            assert!(
                !i18n::t(step.description_key).is_empty(),
                "步驟 {} 應該有描述",
                step.name
            );
        }
    }
}
