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
}
