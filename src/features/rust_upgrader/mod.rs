mod tools;
mod upgrader;

use crate::ui::{Console, Prompts};
use tools::{REQUIRED_CARGO_TOOLS, UPGRADE_STEPS};
use upgrader::RustUpgrader;

/// 執行 Rust 專案升級功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header("升級 Rust 專案與工具鏈");

    let upgrader = RustUpgrader::new();

    // 步驟 1: 檢查 Rust 環境
    console.info("正在檢查 Rust 環境...");
    match upgrader.check_rust_installed() {
        Ok(env) => {
            console.success("Rust 環境已安裝:");
            console.list_item("🦀", &env.rustc_version);
            console.list_item("📦", &env.cargo_version);
            console.list_item("🔧", &env.rustup_version);
        }
        Err(err) => {
            console.error(&format!("Rust 未安裝: {}", err));
            console.info("請先安裝 Rust: https://rustup.rs");
            return;
        }
    }

    console.separator();

    // 步驟 2: 檢查必要的 cargo 工具
    console.info("正在檢查必要的 Cargo 工具...");
    let tool_statuses = upgrader.check_tools_status(REQUIRED_CARGO_TOOLS);

    let missing_tools: Vec<_> = tool_statuses.iter().filter(|s| !s.installed).collect();

    for status in &tool_statuses {
        let icon = if status.installed { "✓" } else { "✗" };
        let state = if status.installed {
            "已安裝"
        } else {
            "未安裝"
        };
        console.list_item(icon, &format!("{} ({})", status.tool.display_name, state));
    }

    console.separator();

    // 步驟 3: 安裝缺少的工具
    if !missing_tools.is_empty() {
        console.warning(&format!("發現 {} 個缺少的工具", missing_tools.len()));

        if prompts.confirm("是否要安裝缺少的工具？") {
            console.blank_line();
            for (i, status) in missing_tools.iter().enumerate() {
                console.show_progress(
                    i + 1,
                    missing_tools.len(),
                    &format!("正在安裝 {}...", status.tool.display_name),
                );

                match upgrader.install_tool(&status.tool) {
                    Ok(_) => {
                        console.success_item(&format!("{} 安裝成功", status.tool.display_name));
                    }
                    Err(err) => {
                        console.error_item(
                            &format!("{} 安裝失敗", status.tool.display_name),
                            &err.to_string(),
                        );
                    }
                }
            }
            console.separator();
        } else {
            console.warning("跳過工具安裝，部分升級功能可能無法使用");
            console.separator();
        }
    } else {
        console.success("所有必要工具都已安裝");
        console.separator();
    }

    // 步驟 4: 顯示升級步驟
    console.info("將執行以下升級步驟：");
    for step in UPGRADE_STEPS {
        let project_tag = if step.requires_project {
            " [需要專案]"
        } else {
            ""
        };
        console.list_item(
            "📋",
            &format!("{}: {}{}", step.name, step.description, project_tag),
        );
    }

    console.separator();

    if !prompts.confirm("確定要執行升級嗎？") {
        console.warning("已取消升級");
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
            &format!("正在執行 {}...", step.name),
        );

        match upgrader.run_upgrade_step(step) {
            Ok(output) => {
                console.success_item(&format!("{} 完成", step.name));
                display_output(&console, &output);
                success_count += 1;
            }
            Err(err) => {
                let err_str = err.to_string();
                if err_str.contains("目前目錄沒有 Cargo.toml") {
                    console.warning(&format!("{} 跳過（無專案）", step.name));
                    skipped_count += 1;
                } else {
                    console.error_item(&format!("{} 失敗", step.name), &err_str);
                    failed_count += 1;
                }
            }
        }
        console.blank_line();
    }

    // 步驟 6: 顯示摘要
    console.show_summary("升級完成", success_count, failed_count);
    if skipped_count > 0 {
        console.info(&format!("跳過: {} 個步驟（無專案）", skipped_count));
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
        console.list_item("  ", &format!("... 還有 {} 行輸出", lines.len() - 5));
    }
}

#[cfg(test)]
mod tests {
    use super::tools::{REQUIRED_CARGO_TOOLS, UPGRADE_STEPS};

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
                !step.description.is_empty(),
                "步驟 {} 應該有描述",
                step.name
            );
        }
    }
}
