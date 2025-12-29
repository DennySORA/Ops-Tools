//! 強類型資料模型
//!
//! 提供完整的類型安全與驗證

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// 自訂類型 - 強化類型安全
// ============================================================================

/// 功能鍵值（非空字串）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FeatureKey(String);

impl FeatureKey {
    /// 建立新的 FeatureKey，驗證非空
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(ValidationError::EmptyField("feature_key".to_string()));
        }
        // 驗證只包含有效字元（字母、數字、連字號、底線）
        if !value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ValidationError::InvalidFormat {
                field: "feature_key".to_string(),
                expected: "只能包含字母、數字、連字號和底線".to_string(),
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FeatureKey {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FeatureKey> for String {
    fn from(key: FeatureKey) -> Self {
        key.0
    }
}

impl fmt::Display for FeatureKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 功能名稱（非空字串）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FeatureName(String);

impl FeatureName {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(ValidationError::EmptyField("feature_name".to_string()));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FeatureName {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FeatureName> for String {
    fn from(name: FeatureName) -> Self {
        name.0
    }
}

impl fmt::Display for FeatureName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// INT 環境 URL（非空且必須是有效 URL 格式）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct IntUrl(String);

impl IntUrl {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(ValidationError::EmptyField("int_url".to_string()));
        }
        // 簡單的 URL 格式驗證
        if !value.starts_with("http://") && !value.starts_with("https://") {
            return Err(ValidationError::InvalidFormat {
                field: "int_url".to_string(),
                expected: "必須以 http:// 或 https:// 開頭".to_string(),
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for IntUrl {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<IntUrl> for String {
    fn from(url: IntUrl) -> Self {
        url.0
    }
}

impl fmt::Display for IntUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// 文字內容類型 - 支援單一字串或字串陣列
// ============================================================================

/// 文字內容 - 可以是單一字串或字串陣列
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextContent {
    /// 單一字串
    Single(String),
    /// 多行字串陣列
    Multiple(Vec<String>),
}

impl TextContent {
    /// 轉換為格式化的區塊文字
    pub fn to_block(&self, bullet: bool) -> String {
        match self {
            TextContent::Single(s) => {
                let text = s.trim();
                if text.is_empty() {
                    Self::empty_fallback()
                } else {
                    text.to_string()
                }
            }
            TextContent::Multiple(items) => {
                if items.is_empty() {
                    return Self::empty_fallback();
                }
                if bullet {
                    items
                        .iter()
                        .map(|line| format!("- {}", line))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    items
                        .iter()
                        .map(|s| s.trim())
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
        }
    }

    /// 預設的空值提示
    fn empty_fallback() -> String {
        "-（未提供）".to_string()
    }

    /// 檢查是否為空
    pub fn is_empty(&self) -> bool {
        match self {
            TextContent::Single(s) => s.trim().is_empty(),
            TextContent::Multiple(items) => {
                items.is_empty() || items.iter().all(|s| s.trim().is_empty())
            }
        }
    }
}

impl Default for TextContent {
    fn default() -> Self {
        TextContent::Single("-（未提供）".to_string())
    }
}

// ============================================================================
// 可選文字內容
// ============================================================================

/// 可選的文字內容
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalTextContent {
    #[default]
    None,
    Some(TextContent),
}

impl OptionalTextContent {
    /// 轉換為格式化的區塊文字
    pub fn to_block(&self, bullet: bool, empty_fallback: &str) -> String {
        match self {
            OptionalTextContent::None => empty_fallback.to_string(),
            OptionalTextContent::Some(content) => {
                if content.is_empty() {
                    empty_fallback.to_string()
                } else {
                    content.to_block(bullet)
                }
            }
        }
    }
}

// ============================================================================
// 功能規格
// ============================================================================

/// 功能規格 - 完整的強類型定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSpec {
    /// 功能鍵值（必填）
    pub feature_key: FeatureKey,

    /// 功能名稱（必填）
    pub feature_name: FeatureName,

    /// 上下文說明（必填）
    #[serde(default)]
    pub context: TextContent,

    /// 需求列表（必填）
    #[serde(default)]
    pub requirements: TextContent,

    /// 驗收條件（必填）
    #[serde(default)]
    pub acceptance_criteria: TextContent,

    /// INT 環境 URL（必填）
    pub int_url: IntUrl,

    /// INT 環境憑證（選填）
    #[serde(default)]
    pub int_credentials: OptionalTextContent,

    /// 是否為前端功能
    #[serde(default)]
    pub is_frontend: bool,
}

impl FeatureSpec {
    /// 驗證功能規格
    pub fn validate(&self) -> Result<(), ValidationError> {
        // 基本驗證已在類型轉換時完成
        // 這裡可以加入額外的業務邏輯驗證
        Ok(())
    }
}

// ============================================================================
// 規格檔案根結構
// ============================================================================

/// 規格檔案的根結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFile {
    /// 功能列表（必填，非空）
    pub features: Vec<FeatureSpec>,
}

impl SpecFile {
    /// 驗證規格檔案
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.features.is_empty() {
            return Err(ValidationError::EmptyFeatures);
        }

        // 檢查 feature_key 唯一性
        let mut seen_keys = std::collections::HashSet::new();
        for (idx, spec) in self.features.iter().enumerate() {
            if !seen_keys.insert(spec.feature_key.as_str()) {
                return Err(ValidationError::DuplicateFeatureKey {
                    key: spec.feature_key.to_string(),
                    index: idx + 1,
                });
            }
            spec.validate()?;
        }

        Ok(())
    }
}

// ============================================================================
// 驗證錯誤類型
// ============================================================================

/// 驗證錯誤
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("欄位 '{0}' 不可為空")]
    EmptyField(String),

    #[error("欄位 '{field}' 格式無效：{expected}")]
    InvalidFormat { field: String, expected: String },

    #[error("features 陣列不可為空")]
    EmptyFeatures,

    #[error("feature_key '{key}' 重複出現於第 {index} 項")]
    DuplicateFeatureKey { key: String, index: usize },

    #[error("features[{index}] 必須是 object")]
    InvalidFeatureType { index: usize },

    #[error("features[{index}] 缺少必要欄位 '{field}'")]
    MissingField { index: usize, field: String },
}

// ============================================================================
// 輸出結構
// ============================================================================

/// 生成的提示檔案
#[derive(Debug, Clone)]
pub struct GeneratedPrompt {
    /// 檔案名稱
    pub filename: String,
    /// 檔案內容
    pub content: String,
}

/// 單一功能的所有提示
#[derive(Debug, Clone)]
pub struct FeaturePrompts {
    /// 功能鍵值
    pub feature_key: FeatureKey,
    /// 生成的提示列表
    pub prompts: Vec<GeneratedPrompt>,
}

impl FeaturePrompts {
    pub fn new(feature_key: FeatureKey) -> Self {
        Self {
            feature_key,
            prompts: Vec::new(),
        }
    }

    pub fn add_prompt(&mut self, filename: impl Into<String>, content: impl Into<String>) {
        self.prompts.push(GeneratedPrompt {
            filename: filename.into(),
            content: content.into(),
        });
    }
}

// ============================================================================
// 測試
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_key_valid() {
        assert!(FeatureKey::new("valid-key").is_ok());
        assert!(FeatureKey::new("valid_key_123").is_ok());
    }

    #[test]
    fn test_feature_key_invalid() {
        assert!(FeatureKey::new("").is_err());
        assert!(FeatureKey::new("   ").is_err());
        assert!(FeatureKey::new("invalid key").is_err()); // 包含空格
    }

    #[test]
    fn test_int_url_valid() {
        assert!(IntUrl::new("https://example.com").is_ok());
        assert!(IntUrl::new("http://localhost:3000").is_ok());
    }

    #[test]
    fn test_int_url_invalid() {
        assert!(IntUrl::new("").is_err());
        assert!(IntUrl::new("ftp://example.com").is_err());
        assert!(IntUrl::new("example.com").is_err());
    }

    #[test]
    fn test_text_content_to_block() {
        let single = TextContent::Single("Hello".to_string());
        assert_eq!(single.to_block(false), "Hello");

        let multiple = TextContent::Multiple(vec!["Line 1".to_string(), "Line 2".to_string()]);
        assert_eq!(multiple.to_block(true), "- Line 1\n- Line 2");
        assert_eq!(multiple.to_block(false), "Line 1\nLine 2");
    }
}
