/// AI 程式碼助手工具定義
#[derive(Debug, Clone)]
pub struct AiTool {
    pub package: &'static str,
    pub name: &'static str,
}

impl AiTool {
    pub const fn new(package: &'static str, name: &'static str) -> Self {
        Self { package, name }
    }
}

/// 預設的 AI 工具清單
pub const AI_TOOLS: &[AiTool] = &[
    AiTool::new("@anthropic-ai/claude-code", "Claude Code"),
    AiTool::new("@openai/codex", "OpenAI Codex"),
    AiTool::new("@google/gemini-cli", "Google Gemini CLI"),
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
    fn test_all_packages_have_scope() {
        for tool in AI_TOOLS {
            assert!(
                tool.package.starts_with('@'),
                "套件 {} 應該有 scope",
                tool.package
            );
        }
    }
}
