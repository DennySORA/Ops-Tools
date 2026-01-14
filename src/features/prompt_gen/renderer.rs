//! 模板渲染器
//!
//! 負責將 FeatureSpec 渲染為提示檔案

use super::models::{FeaturePrompts, FeatureSpec, ProjectType, TextContent};
use super::templates::{
    FRONTEND_SECTION, STATE_REQUIREMENT_BLOCK, TEMPLATE_01, TEMPLATE_02_FIXED, TEMPLATE_03,
    TEMPLATE_04_FIXED,
};

// ============================================================================
// 提示檔案名稱常數
// ============================================================================

/// 提示檔案名稱
pub mod filenames {
    pub const PROMPT_01: &str = "01_requirements_and_delivery.md";
    pub const PROMPT_02: &str = "02_int_e2e_validate.md";
    pub const PROMPT_03: &str = "03_refactor_and_polish.md";
    pub const PROMPT_04: &str = "04_int_e2e_regression.md";
}

// ============================================================================
// 渲染器
// ============================================================================

/// 提示渲染器
pub struct PromptRenderer;

impl PromptRenderer {
    /// 渲染單一功能的所有提示
    pub fn render(spec: &FeatureSpec) -> FeaturePrompts {
        let mut prompts = FeaturePrompts::new(spec.feature_key.clone());

        // 取得有效的專案類型
        let project_type = spec.effective_project_type();

        // 準備共用區塊
        let context_block = spec.context.to_block(false);
        let requirements_block = spec.requirements.to_block(false);
        let acceptance_block = Self::format_acceptance(&spec.acceptance_criteria);
        let creds_block = spec
            .int_credentials
            .to_block(false, "-（不需要或由環境變數/SSO/既有機制提供）");
        let state_requirement = Self::render_state_requirement(spec.feature_key.as_str());
        let frontend_section_block = if project_type.needs_browser() {
            FRONTEND_SECTION.replace("{IS_FRONTEND}", "true")
        } else {
            String::new()
        };

        // 渲染 Prompt #1
        let prompt_01 = Self::render_template_01(
            spec.feature_key.as_str(),
            spec.feature_name.as_str(),
            &context_block,
            &requirements_block,
            &acceptance_block,
            spec.verification_url_str(),
            &creds_block,
            &state_requirement,
        );
        prompts.add_prompt(filenames::PROMPT_01, prompt_01);

        // 渲染 Prompt #2
        let prompt_02 =
            Self::render_template_02(spec.feature_key.as_str(), &state_requirement, project_type);
        prompts.add_prompt(filenames::PROMPT_02, prompt_02);

        // 渲染 Prompt #3
        let prompt_03 = Self::render_template_03(
            spec.feature_key.as_str(),
            project_type.needs_browser(),
            &frontend_section_block,
            &state_requirement,
        );
        prompts.add_prompt(filenames::PROMPT_03, prompt_03);

        // 渲染 Prompt #4
        let prompt_04 =
            Self::render_template_04(spec.feature_key.as_str(), &state_requirement, project_type);
        prompts.add_prompt(filenames::PROMPT_04, prompt_04);

        prompts
    }

    /// 渲染狀態需求區塊
    fn render_state_requirement(feature_key: &str) -> String {
        STATE_REQUIREMENT_BLOCK.replace("{FEATURE_KEY}", feature_key)
    }

    /// 格式化驗收條件
    fn format_acceptance(criteria: &TextContent) -> String {
        match criteria {
            TextContent::Multiple(_) => criteria.to_block(true),
            TextContent::Single(_) => criteria.to_block(false),
        }
    }

    /// 渲染模板 01
    #[allow(clippy::too_many_arguments)]
    fn render_template_01(
        feature_key: &str,
        feature_name: &str,
        context_block: &str,
        requirements_block: &str,
        acceptance_block: &str,
        verification_url: &str,
        int_credentials_block: &str,
        state_requirement: &str,
    ) -> String {
        TEMPLATE_01
            .replace("{FEATURE_KEY}", feature_key)
            .replace("{FEATURE_NAME}", feature_name)
            .replace("{CONTEXT_BLOCK}", context_block)
            .replace("{REQUIREMENTS_BLOCK}", requirements_block)
            .replace("{ACCEPTANCE_BLOCK}", acceptance_block)
            .replace("{VERIFICATION_URL}", verification_url)
            .replace("{VERIFICATION_CREDENTIALS_BLOCK}", int_credentials_block)
            .replace("{STATE_REQUIREMENT}", state_requirement)
    }

    /// 渲染模板 02
    fn render_template_02(
        feature_key: &str,
        state_requirement: &str,
        project_type: ProjectType,
    ) -> String {
        TEMPLATE_02_FIXED
            .replace("{FEATURE_KEY}", feature_key)
            .replace("{STATE_REQUIREMENT}", state_requirement)
            .replace("{PROJECT_TYPE}", &project_type.to_string())
            .replace(
                "{E2E_TESTING_INSTRUCTIONS}",
                project_type.e2e_instructions(),
            )
    }

    /// 渲染模板 03
    fn render_template_03(
        feature_key: &str,
        is_frontend: bool,
        frontend_section_block: &str,
        state_requirement: &str,
    ) -> String {
        TEMPLATE_03
            .replace("{FEATURE_KEY}", feature_key)
            .replace("{IS_FRONTEND}", if is_frontend { "true" } else { "false" })
            .replace("{FRONTEND_SECTION_BLOCK}", frontend_section_block)
            .replace("{STATE_REQUIREMENT}", state_requirement)
    }

    /// 渲染模板 04
    fn render_template_04(
        feature_key: &str,
        state_requirement: &str,
        project_type: ProjectType,
    ) -> String {
        TEMPLATE_04_FIXED
            .replace("{FEATURE_KEY}", feature_key)
            .replace("{STATE_REQUIREMENT}", state_requirement)
            .replace("{PROJECT_TYPE}", &project_type.to_string())
            .replace(
                "{E2E_TESTING_INSTRUCTIONS}",
                project_type.e2e_instructions(),
            )
    }
}

// ============================================================================
// 批次渲染
// ============================================================================

/// 批次渲染多個功能
pub fn render_all(specs: &[FeatureSpec]) -> Vec<FeaturePrompts> {
    specs.iter().map(PromptRenderer::render).collect()
}

// ============================================================================
// 測試
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::prompt_gen::models::{
        FeatureKey, FeatureName, FeatureOptions, OptionalTextContent, ProjectType, VerificationUrl,
    };

    fn create_test_spec() -> FeatureSpec {
        FeatureSpec {
            feature_key: FeatureKey::new("test-feature").unwrap(),
            feature_name: FeatureName::new("Test Feature").unwrap(),
            project_type: ProjectType::Frontend,
            options: FeatureOptions::from_project_type(ProjectType::Frontend),
            context: TextContent::Single("Test context".to_string()),
            requirements: TextContent::Multiple(vec![
                "Requirement 1".to_string(),
                "Requirement 2".to_string(),
            ]),
            acceptance_criteria: TextContent::Multiple(vec![
                "Criteria 1".to_string(),
                "Criteria 2".to_string(),
            ]),
            verification_url: Some(VerificationUrl::new("https://example.com").unwrap()),
            int_credentials: OptionalTextContent::None,
            is_frontend: false,
        }
    }

    #[test]
    fn test_render_generates_four_prompts() {
        let spec = create_test_spec();
        let prompts = PromptRenderer::render(&spec);

        assert_eq!(prompts.prompts.len(), 4);
        assert_eq!(prompts.prompts[0].filename, filenames::PROMPT_01);
        assert_eq!(prompts.prompts[1].filename, filenames::PROMPT_02);
        assert_eq!(prompts.prompts[2].filename, filenames::PROMPT_03);
        assert_eq!(prompts.prompts[3].filename, filenames::PROMPT_04);
    }

    #[test]
    fn test_render_contains_feature_key() {
        let spec = create_test_spec();
        let prompts = PromptRenderer::render(&spec);

        for prompt in &prompts.prompts {
            assert!(
                prompt.content.contains("test-feature"),
                "Prompt {} should contain feature key",
                prompt.filename
            );
        }
    }

    #[test]
    fn test_render_frontend_section() {
        let mut spec = create_test_spec();
        spec.project_type = ProjectType::Frontend;

        let prompts = PromptRenderer::render(&spec);
        let prompt_03 = &prompts.prompts[2].content;

        assert!(prompt_03.contains("visual direction"));
    }

    #[test]
    fn test_render_non_frontend_no_section() {
        let mut spec = create_test_spec();
        spec.project_type = ProjectType::Backend;

        let prompts = PromptRenderer::render(&spec);
        let prompt_03 = &prompts.prompts[2].content;

        assert!(!prompt_03.contains("visual direction"));
    }

    #[test]
    fn test_render_backend_project_type() {
        let mut spec = create_test_spec();
        spec.project_type = ProjectType::Backend;

        let prompts = PromptRenderer::render(&spec);
        let prompt_02 = &prompts.prompts[1].content;
        let prompt_04 = &prompts.prompts[3].content;

        assert!(prompt_02.contains("Type: backend"));
        assert!(prompt_02.contains("HTTP client tools"));
        assert!(prompt_04.contains("Type: backend"));
    }

    #[test]
    fn test_render_cli_project_type() {
        let mut spec = create_test_spec();
        spec.project_type = ProjectType::Cli;

        let prompts = PromptRenderer::render(&spec);
        let prompt_02 = &prompts.prompts[1].content;

        assert!(prompt_02.contains("Type: cli"));
        assert!(prompt_02.contains("CLI commands"));
    }

    #[test]
    fn test_render_library_project_type() {
        let mut spec = create_test_spec();
        spec.project_type = ProjectType::Library;

        let prompts = PromptRenderer::render(&spec);
        let prompt_02 = &prompts.prompts[1].content;

        assert!(prompt_02.contains("Type: library"));
        assert!(prompt_02.contains("test suite"));
    }

    #[test]
    fn test_is_frontend_backward_compatibility() {
        let mut spec = create_test_spec();
        spec.project_type = ProjectType::Backend;
        spec.is_frontend = true; // 向後相容：這應該覆蓋 project_type

        let prompts = PromptRenderer::render(&spec);
        let prompt_02 = &prompts.prompts[1].content;

        // 應該使用 Frontend 的設定
        assert!(prompt_02.contains("Type: frontend"));
        assert!(prompt_02.contains("real browser"));
    }
}
