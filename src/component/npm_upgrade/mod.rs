use std::process::Command;

use crate::tools::ui::UserInterface;

/// AI 程式碼助手的 pnpm 套件清單
const AI_TOOLS: &[(&str, &str)] = &[
    ("@anthropic-ai/claude-code", "Claude Code"),
    ("@github/copilot", "GitHub Copilot"),
    ("@openai/codex", "OpenAI Codex"),
    ("@google/gemini-cli", "Google Gemini CLI"),
];

/// 升級所有 AI 程式碼助手工具
pub fn upgrade_ai_tools() {
    let ui = UserInterface::new();
    ui.header("升級 AI 程式碼助手工具");

    ui.info("將升級以下工具：");
    for (pkg, name) in AI_TOOLS {
        ui.list_item("📦", &format!("{} ({})", name, pkg));
    }
    ui.separator();

    if !ui.confirm("確定要升級這些工具嗎？") {
        ui.warning("已取消升級");
        return;
    }

    println!();

    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, (pkg, name)) in AI_TOOLS.iter().enumerate() {
        ui.show_progress(i + 1, AI_TOOLS.len(), &format!("正在升級 {}...", name));

        match upgrade_package(pkg) {
            Ok(output) => {
                ui.success_item(&format!("{} 升級成功", name));
                if !output.trim().is_empty() {
                    for line in output.lines() {
                        println!("    {}", line);
                    }
                }
                success_count += 1;
            }
            Err(err) => {
                ui.error_item(&format!("{} 升級失敗", name), &err);
                for line in err.lines() {
                    println!("    {}", line);
                }
                failed_count += 1;
            }
        }
        println!();
    }

    ui.show_summary("升級完成", success_count, failed_count);
}

/// 執行 pnpm add -g 來升級指定套件
fn upgrade_package(package: &str) -> Result<String, String> {
    let full_package = format!("{}@latest", package);

    let output = Command::new("pnpm")
        .args(["add", "-g", &full_package])
        .output()
        .map_err(|e| format!("無法執行 pnpm: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(stderr.lines().next().unwrap_or("未知錯誤").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_tools_list_should_not_be_empty() {
        assert!(!AI_TOOLS.is_empty());
    }

    #[test]
    fn all_packages_should_have_scope() {
        for (pkg, _) in AI_TOOLS {
            assert!(pkg.starts_with('@'), "套件 {} 應該有 scope", pkg);
        }
    }
}
