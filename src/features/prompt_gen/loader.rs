//! YAML/JSON 檔案載入器
//!
//! 負責讀取和解析規格檔案

use super::models::{SpecFile, ValidationError};
use std::path::Path;
use thiserror::Error;

// ============================================================================
// 錯誤類型
// ============================================================================

/// 載入錯誤
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("檔案不存在：{0}")]
    FileNotFound(String),

    #[error("無法讀取檔案：{0}")]
    ReadError(#[from] std::io::Error),

    #[error("不支援的檔案格式：{extension}（請用 .yaml/.yml 或 .json）")]
    UnsupportedFormat { extension: String },

    #[error("YAML 解析錯誤：{0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON 解析錯誤：{0}")]
    JsonError(#[from] serde_json::Error),

    #[error("驗證錯誤：{0}")]
    ValidationError(#[from] ValidationError),
}

// ============================================================================
// 檔案格式
// ============================================================================

/// 支援的檔案格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Yaml,
    Json,
}

impl FileFormat {
    /// 從副檔名判斷格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

// ============================================================================
// 載入器
// ============================================================================

/// 規格檔案載入器
pub struct SpecLoader;

impl SpecLoader {
    /// 從檔案路徑載入規格
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<SpecFile, LoadError> {
        let path = path.as_ref();

        // 檢查檔案是否存在
        if !path.exists() {
            return Err(LoadError::FileNotFound(path.display().to_string()));
        }

        // 取得副檔名
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        // 判斷格式
        let format =
            FileFormat::from_extension(extension).ok_or_else(|| LoadError::UnsupportedFormat {
                extension: extension.to_string(),
            })?;

        // 讀取檔案內容
        let content = std::fs::read_to_string(path)?;

        // 解析檔案
        Self::parse(&content, format)
    }

    /// 從字串解析規格
    pub fn parse(content: &str, format: FileFormat) -> Result<SpecFile, LoadError> {
        let spec: SpecFile = match format {
            FileFormat::Yaml => serde_yaml::from_str(content)?,
            FileFormat::Json => serde_json::from_str(content)?,
        };

        // 驗證規格
        spec.validate()?;

        Ok(spec)
    }
}

// ============================================================================
// 測試
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format_detection() {
        assert_eq!(FileFormat::from_extension("yaml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_extension("yml"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_extension("YAML"), Some(FileFormat::Yaml));
        assert_eq!(FileFormat::from_extension("json"), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_extension("JSON"), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
features:
  - feature_key: test-feature
    feature_name: "Test Feature"
    context: "Test context"
    requirements: "Test requirements"
    acceptance_criteria:
      - "Criteria 1"
      - "Criteria 2"
    verification_url: "https://example.com"
    is_frontend: false
"#;

        let result = SpecLoader::parse(yaml, FileFormat::Yaml);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.features.len(), 1);
        assert_eq!(spec.features[0].feature_key.as_str(), "test-feature");
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{
    "features": [
        {
            "feature_key": "test-feature",
            "feature_name": "Test Feature",
            "context": "Test context",
            "requirements": "Test requirements",
            "acceptance_criteria": ["Criteria 1", "Criteria 2"],
            "verification_url": "https://example.com",
            "is_frontend": false
        }
    ]
}"#;

        let result = SpecLoader::parse(json, FileFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_features() {
        let yaml = r#"
features: []
"#;

        let result = SpecLoader::parse(yaml, FileFormat::Yaml);
        assert!(result.is_err());
    }

    /// 測試所有專案類型的 YAML 範例能否正確解析
    #[test]
    fn test_parse_all_project_types() {
        use crate::features::prompt_gen::models::ProjectType;

        for project_type in ProjectType::ALL {
            let yaml = format!(
                r#"
features:
  - feature_key: test-{pt}
    feature_name: "Add: Test feature for {pt}"
    project_type: {pt}
    options:
      needs_local_validation: true
      has_verification_env: false
      needs_deployment: false
    context:
      - "Test context for {pt} project"
      - "Second line of context"
    requirements:
      - "Requirement 1"
      - "Requirement 2"
    acceptance_criteria:
      - "Criteria 1"
      - "Criteria 2"
"#,
                pt = project_type
            );

            let result = SpecLoader::parse(&yaml, FileFormat::Yaml);
            assert!(
                result.is_ok(),
                "Failed to parse YAML for project type {}: {:?}",
                project_type,
                result.err()
            );

            let spec = result.unwrap();
            assert_eq!(spec.features.len(), 1);
            assert_eq!(
                spec.features[0].effective_project_type(),
                project_type,
                "Project type mismatch for {}",
                project_type
            );
        }
    }

    /// 測試帶有 options 的完整 YAML 能否解析
    #[test]
    fn test_parse_yaml_with_options() {
        let yaml = r#"
features:
  - feature_key: cli-feature
    feature_name: "Add: CLI validation command"
    project_type: cli
    options:
      needs_local_validation: true
      has_verification_env: false
      needs_deployment: false
      test_command: "cargo test"
      build_command: "cargo build --release"
    context:
      - "Add a validation command"
    requirements:
      - "Command must accept --help flag"
    acceptance_criteria:
      - "Exit code 0 on success"
"#;

        let result = SpecLoader::parse(yaml, FileFormat::Yaml);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let spec = result.unwrap();
        assert_eq!(
            spec.features[0].options.test_command,
            Some("cargo test".to_string())
        );
        assert_eq!(
            spec.features[0].options.build_command,
            Some("cargo build --release".to_string())
        );
        assert!(!spec.features[0].options.has_verification_env);
        assert!(!spec.features[0].options.needs_deployment);
    }

    /// 測試完整端到端流程：YAML -> Parse -> Render
    #[test]
    fn test_end_to_end_yaml_to_prompts() {
        use crate::features::prompt_gen::renderer::PromptRenderer;

        let yaml = r#"
features:
  - feature_key: add-user-auth
    feature_name: "Add: User authentication API"
    project_type: backend
    options:
      needs_local_validation: true
      has_verification_env: true
      needs_deployment: true
    context:
      - "Implement JWT-based authentication for the API"
      - "Required for securing all protected endpoints"
    requirements:
      - "POST /auth/login accepts email and password"
      - "Returns JWT token on success"
      - "Token expires in 24 hours"
    acceptance_criteria:
      - "Login with valid credentials returns 200 and token"
      - "Login with invalid credentials returns 401"
      - "Protected endpoints reject requests without token"
    verification_url: "https://int.example.com"
"#;

        // Step 1: Parse YAML
        let spec = SpecLoader::parse(yaml, FileFormat::Yaml).expect("Failed to parse YAML");
        assert_eq!(spec.features.len(), 1);

        // Step 2: Render prompts
        let feature_prompts = PromptRenderer::render(&spec.features[0]);
        assert_eq!(feature_prompts.prompts.len(), 4);

        // Step 3: Verify prompt content
        let prompt_01 = &feature_prompts.prompts[0].content;
        assert!(
            prompt_01.contains("add-user-auth"),
            "Should contain feature key"
        );
        assert!(
            prompt_01.contains("User authentication API"),
            "Should contain feature name"
        );
        assert!(
            prompt_01.contains("JWT-based authentication"),
            "Should contain context"
        );
        assert!(
            prompt_01.contains("POST /auth/login"),
            "Should contain requirements"
        );
        assert!(
            prompt_01.contains("https://int.example.com"),
            "Should contain verification URL"
        );

        let prompt_02 = &feature_prompts.prompts[1].content;
        assert!(
            prompt_02.contains("Type: backend"),
            "Should contain project type"
        );
        assert!(
            prompt_02.contains("HTTP client tools"),
            "Should contain backend-specific instructions"
        );

        // Verify filenames
        assert_eq!(
            feature_prompts.prompts[0].filename,
            "01_requirements_and_delivery.md"
        );
        assert_eq!(
            feature_prompts.prompts[1].filename,
            "02_int_e2e_validate.md"
        );
        assert_eq!(
            feature_prompts.prompts[2].filename,
            "03_refactor_and_polish.md"
        );
        assert_eq!(
            feature_prompts.prompts[3].filename,
            "04_int_e2e_regression.md"
        );
    }
}
