/// 升級指令的型別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeCommand {
    /// 透過 Node 套件管理器安裝（會自動加上 @latest）
    PackageManager {
        manager: &'static str,
        package: &'static str,
    },
    /// 直接呼叫自訂命令
    Custom {
        program: &'static str,
        args: &'static [&'static str],
    },
}

/// AI 程式碼助手工具定義
#[derive(Debug, Clone, Copy)]
pub struct AiTool {
    /// 工具名稱
    pub name: &'static str,
    /// 清單顯示用的目標描述（套件名稱或指令）
    pub display: &'static str,
    /// 升級方式
    pub command: UpgradeCommand,
}

impl AiTool {
    #[allow(dead_code)]
    pub const fn from_package(name: &'static str, package: &'static str) -> Self {
        Self::from_package_with_manager(name, package, "npm")
    }

    pub const fn from_package_with_manager(
        name: &'static str,
        package: &'static str,
        manager: &'static str,
    ) -> Self {
        Self {
            name,
            display: package,
            command: UpgradeCommand::PackageManager { manager, package },
        }
    }

    pub const fn with_custom_command(
        name: &'static str,
        display: &'static str,
        program: &'static str,
        args: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            display,
            command: UpgradeCommand::Custom { program, args },
        }
    }
}

/// 預設的 AI 工具清單
pub const AI_TOOLS: &[AiTool] = &[
    // Claude 官方建議使用內建安裝指令進行更新
    AiTool::with_custom_command("Claude Code", "claude install", "claude", &["install"]),
    AiTool::with_custom_command(
        "OpenAI Codex",
        "bun install -g @openai/codex",
        "bun",
        &["install", "-g", "@openai/codex"],
    ),
    AiTool::from_package_with_manager("Google Gemini CLI", "@google/gemini-cli", "npm"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_ai_tools_not_empty() {
        assert!(!AI_TOOLS.is_empty());
    }

    #[test]
    fn test_package_tools_have_scope() {
        for tool in AI_TOOLS {
            if let UpgradeCommand::PackageManager { package, .. } = tool.command {
                assert!(package.starts_with('@'), "套件 {} 應該有 scope", package);
            }
        }
    }

    #[test]
    fn test_claude_uses_custom_command() {
        let claude = AI_TOOLS
            .iter()
            .find(|t| t.name.contains("Claude"))
            .expect("Claude tool should exist");

        assert!(matches!(claude.command, UpgradeCommand::Custom { .. }));
    }
}
