//! 強類型資料模型
//!
//! 提供完整的類型安全與驗證

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// 專案類型
// ============================================================================

/// 專案類型 - 決定測試策略和模板內容
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    /// 前端專案（Web UI）- 使用瀏覽器 E2E
    #[default]
    Frontend,
    /// 後端 API 服務 - 使用 HTTP 測試
    Backend,
    /// 全端應用 - 前後端整合
    Fullstack,
    /// CLI 工具 - 使用命令行測試
    Cli,
    /// Library / SDK - 使用單元/整合測試
    Library,
    /// 底層/系統級開發 - 驅動、kernel module
    #[serde(alias = "system")]
    SystemLevel,
    /// 演算法/ML - 效能敏感、需 benchmark
    Algorithm,
    /// Infrastructure 配置 - 使用配置驗證
    Infra,
}

#[allow(dead_code)]
impl ProjectType {
    /// 所有可用的專案類型
    pub const ALL: [ProjectType; 8] = [
        ProjectType::Frontend,
        ProjectType::Backend,
        ProjectType::Fullstack,
        ProjectType::Cli,
        ProjectType::Library,
        ProjectType::SystemLevel,
        ProjectType::Algorithm,
        ProjectType::Infra,
    ];

    /// 是否需要瀏覽器測試
    pub fn needs_browser(&self) -> bool {
        matches!(self, ProjectType::Frontend | ProjectType::Fullstack)
    }

    /// 是否需要部署（預設）
    pub fn typically_needs_deployment(&self) -> bool {
        matches!(
            self,
            ProjectType::Frontend
                | ProjectType::Backend
                | ProjectType::Fullstack
                | ProjectType::Infra
        )
    }

    /// 是否需要遠端驗證環境（預設）
    pub fn typically_needs_verification_env(&self) -> bool {
        matches!(
            self,
            ProjectType::Frontend
                | ProjectType::Backend
                | ProjectType::Fullstack
                | ProjectType::Infra
        )
    }

    /// 取得角色描述
    pub fn role_description(&self) -> &'static str {
        match self {
            ProjectType::Frontend => "senior frontend engineer",
            ProjectType::Backend => "senior backend engineer",
            ProjectType::Fullstack => "senior full-stack engineer",
            ProjectType::Cli => "senior software engineer specializing in CLI tools",
            ProjectType::Library => "senior software engineer specializing in library/SDK design",
            ProjectType::SystemLevel => {
                "senior systems engineer specializing in low-level/OS development"
            }
            ProjectType::Algorithm => {
                "senior software engineer specializing in algorithms and performance optimization"
            }
            ProjectType::Infra => "senior infrastructure/DevOps engineer",
        }
    }

    /// 取得 E2E 測試說明
    pub fn e2e_instructions(&self) -> &'static str {
        match self {
            ProjectType::Frontend => {
                "Use a real browser and developer tools for comprehensive end-to-end testing (including console/network/error checks)."
            }
            ProjectType::Backend => {
                "Use HTTP client tools (curl, httpie, Postman, or similar) to test API endpoints. Verify request/response payloads, status codes, headers, and error handling."
            }
            ProjectType::Fullstack => {
                "Use a real browser for UI testing and HTTP tools for API testing. Verify full request flow from UI to backend to database."
            }
            ProjectType::Cli => {
                "Execute CLI commands in a shell environment. Verify exit codes, stdout/stderr output, file operations, and edge cases (invalid arguments, missing files, etc.)."
            }
            ProjectType::Library => {
                "Run the test suite (unit + integration tests). Verify public API behavior, edge cases, error handling, and compatibility with expected use cases."
            }
            ProjectType::SystemLevel => {
                "Run system-level tests in appropriate environment (VM, container, or bare metal). Verify memory safety, resource handling, and hardware interaction."
            }
            ProjectType::Algorithm => {
                "Run benchmark suite and correctness tests. Verify algorithmic correctness, performance characteristics, and edge cases with various input sizes."
            }
            ProjectType::Infra => {
                "Validate infrastructure configuration using appropriate tools (terraform validate, kubectl dry-run, ansible --check, etc.). Verify resource states and connectivity."
            }
        }
    }

    /// 取得專案類型專屬的需求區塊
    pub fn specific_requirements(&self) -> &'static str {
        match self {
            ProjectType::Frontend => {
                r#"## Frontend-Specific Requirements
- UI must reflect backend/DB truth (no client-side inference as source of truth)
- Chrome DevTools: console 0 error AND network 0 failure
- Define clear visual direction (typography, color, hierarchy, spacing)
- Interactions must provide explicit feedback (hover/focus/active states)
- Primary actions must show loading/success/error states"#
            }
            ProjectType::Backend => {
                r#"## Backend-Specific Requirements
- API contracts must be documented (OpenAPI/Swagger recommended)
- Error responses must follow consistent format with error codes
- Endpoints must be idempotent where applicable
- Rate limiting and authentication must be implemented
- Database migrations must be reversible"#
            }
            ProjectType::Fullstack => {
                r#"## Fullstack-Specific Requirements
- API contracts between frontend and backend must be documented
- UI must reflect backend/DB truth (no client-side inference)
- End-to-end data flow must be validated
- Chrome DevTools: console 0 error AND network 0 failure
- Database migrations must be reversible"#
            }
            ProjectType::Cli => {
                r#"## CLI-Specific Requirements
- Argument parsing must handle all edge cases (missing args, invalid values)
- Help text (--help) must be comprehensive and accurate
- Exit codes must follow conventions (0=success, non-zero=error)
- Stderr for errors, stdout for normal output
- Support for stdin/stdout piping where applicable
- Shell completion scripts if applicable"#
            }
            ProjectType::Library => {
                r#"## Library-Specific Requirements
- Public API must be stable and well-documented
- Breaking changes must follow semantic versioning
- Documentation must include usage examples
- Dependencies must be minimal and justified
- Thread safety and error handling must be documented
- Package metadata (name, version, description) must be complete"#
            }
            ProjectType::SystemLevel => {
                r#"## System-Level Requirements
- Memory safety must be verified (no leaks, no undefined behavior)
- Resource cleanup must be guaranteed (RAII or equivalent)
- Performance constraints must be documented and tested
- Hardware compatibility requirements must be specified
- Privilege escalation must be minimal and documented
- Error handling must not crash the system"#
            }
            ProjectType::Algorithm => {
                r#"## Algorithm-Specific Requirements
- Time complexity must be documented and verified
- Space complexity must be documented and verified
- Numerical stability must be considered (if applicable)
- Edge cases must be identified and tested (empty input, max size, etc.)
- Benchmark suite must cover representative workloads
- Comparison with baseline/alternative implementations if applicable"#
            }
            ProjectType::Infra => {
                r#"## Infrastructure-Specific Requirements
- Configuration must be idempotent (re-running produces same result)
- Rollback procedure must be documented and tested
- State drift detection and remediation must be considered
- Secrets must not be committed (use secret managers)
- Resource dependencies must be explicitly declared
- Dry-run validation before actual apply"#
            }
        }
    }

    /// 取得專案類型專屬的產出物清單
    pub fn artifacts_description(&self) -> &'static str {
        match self {
            ProjectType::Frontend | ProjectType::Fullstack => {
                r#"1) `STATE.md`: Current state (decisions, completed items, TODOs, risks, how to validate; include STATUS field)
2) `E2E_PLAN.md`: Browser-executable end-to-end checklist (steps must be precise)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `RUNBOOK_VERIFICATION.md`: How to deploy, rollback, and required configuration
5) `CHANGELOG.md`: Feature change summary (reviewer-facing)"#
            }
            ProjectType::Backend => {
                r#"1) `STATE.md`: Current state (decisions, completed items, TODOs, risks; include STATUS field)
2) `E2E_PLAN.md`: API testing checklist (endpoints, payloads, expected responses)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `RUNBOOK_VERIFICATION.md`: How to deploy, rollback, and required configuration
5) `API_SPEC.md`: API documentation (or OpenAPI spec file)
6) `CHANGELOG.md`: Feature change summary (reviewer-facing)"#
            }
            ProjectType::Cli => {
                r#"1) `STATE.md`: Current state (decisions, completed items, TODOs, risks; include STATUS field)
2) `TEST_PLAN.md`: CLI testing checklist (commands, arguments, expected output/exit codes)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `USAGE.md`: Command usage documentation with examples
5) `CHANGELOG.md`: Feature change summary (reviewer-facing)"#
            }
            ProjectType::Library => {
                r#"1) `STATE.md`: Current state (decisions, completed items, TODOs, risks; include STATUS field)
2) `TEST_PLAN.md`: Test coverage plan (unit tests, integration tests, edge cases)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `API_DOCS.md`: Public API documentation with examples
5) `PUBLISH_CHECKLIST.md`: Steps to publish (version bump, changelog, registry publish)
6) `CHANGELOG.md`: Feature change summary (reviewer-facing)"#
            }
            ProjectType::SystemLevel => {
                r#"1) `STATE.md`: Current state (decisions, completed items, TODOs, risks; include STATUS field)
2) `TEST_PLAN.md`: System testing checklist (environments, hardware requirements)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `SAFETY_CHECKLIST.md`: Memory safety, resource cleanup, error handling verification
5) `PERFORMANCE_REPORT.md`: Performance measurements and constraints
6) `CHANGELOG.md`: Feature change summary (reviewer-facing)"#
            }
            ProjectType::Algorithm => {
                r#"1) `STATE.md`: Current state (decisions, completed items, TODOs, risks; include STATUS field)
2) `TEST_PLAN.md`: Correctness testing plan (edge cases, invariants)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `BENCHMARK_PLAN.md`: Benchmark methodology, datasets, expected performance
5) `COMPLEXITY_ANALYSIS.md`: Time/space complexity documentation
6) `CHANGELOG.md`: Feature change summary (reviewer-facing)"#
            }
            ProjectType::Infra => {
                r#"1) `STATE.md`: Current state (decisions, completed items, TODOs, risks; include STATUS field)
2) `VALIDATION_PLAN.md`: Infrastructure validation checklist (dry-run, connectivity, state)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `RUNBOOK.md`: How to apply, rollback, and required configuration
5) `DRIFT_DETECTION.md`: How to detect and remediate configuration drift
6) `CHANGELOG.md`: Feature change summary (reviewer-facing)"#
            }
        }
    }
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectType::Frontend => write!(f, "frontend"),
            ProjectType::Backend => write!(f, "backend"),
            ProjectType::Fullstack => write!(f, "fullstack"),
            ProjectType::Cli => write!(f, "cli"),
            ProjectType::Library => write!(f, "library"),
            ProjectType::SystemLevel => write!(f, "systemlevel"),
            ProjectType::Algorithm => write!(f, "algorithm"),
            ProjectType::Infra => write!(f, "infra"),
        }
    }
}

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

/// 驗證 URL（可為空；若提供則必須是有效 URL 格式）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct VerificationUrl(String);

impl VerificationUrl {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into().trim().to_string();
        if !value.is_empty() && !value.starts_with("http://") && !value.starts_with("https://") {
            return Err(ValidationError::InvalidFormat {
                field: "verification_url".to_string(),
                expected: "必須以 http:// 或 https:// 開頭，或留空".to_string(),
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for VerificationUrl {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<VerificationUrl> for String {
    fn from(url: VerificationUrl) -> Self {
        url.0
    }
}

impl fmt::Display for VerificationUrl {
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
// 功能選項配置
// ============================================================================

/// 功能選項 - 控制模板生成的可選配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureOptions {
    /// 是否有遠端驗證環境（如果為 false，跳過部署和遠端驗證相關內容）
    #[serde(default)]
    pub has_verification_env: bool,

    /// 是否需要本地完整驗證
    #[serde(default = "default_true")]
    pub needs_local_validation: bool,

    /// 是否需要部署
    #[serde(default)]
    pub needs_deployment: bool,

    /// 自定義驗證方式說明（選填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_validation: Option<String>,

    /// 自定義測試命令（選填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_command: Option<String>,

    /// 自定義建置命令（選填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
}

impl Default for FeatureOptions {
    fn default() -> Self {
        Self {
            has_verification_env: false,
            needs_local_validation: true, // 預設需要本地驗證
            needs_deployment: false,
            custom_validation: None,
            test_command: None,
            build_command: None,
        }
    }
}

fn default_true() -> bool {
    true
}

#[allow(dead_code)]
impl FeatureOptions {
    /// 根據專案類型建立預設選項
    pub fn from_project_type(project_type: ProjectType) -> Self {
        Self {
            has_verification_env: project_type.typically_needs_verification_env(),
            needs_local_validation: true,
            needs_deployment: project_type.typically_needs_deployment(),
            custom_validation: None,
            test_command: None,
            build_command: None,
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

    /// 專案類型（決定測試策略）
    #[serde(default)]
    pub project_type: ProjectType,

    /// 功能選項配置
    #[serde(default)]
    pub options: FeatureOptions,

    /// 上下文說明（必填）
    #[serde(default)]
    pub context: TextContent,

    /// 需求列表（必填）
    #[serde(default)]
    pub requirements: TextContent,

    /// 驗收條件（必填）
    #[serde(default)]
    pub acceptance_criteria: TextContent,

    /// 驗證 URL（可為空，僅當 options.has_verification_env = true 時使用）
    #[serde(default, skip_serializing_if = "is_empty_url")]
    pub verification_url: Option<VerificationUrl>,

    /// 驗證環境憑證（選填，僅當 options.has_verification_env = true 時使用）
    #[serde(default, skip_serializing_if = "is_none_content")]
    pub int_credentials: OptionalTextContent,

    /// 是否為前端功能（向後相容，已棄用，請使用 project_type）
    #[serde(default, skip_serializing)]
    pub is_frontend: bool,
}

fn is_empty_url(url: &Option<VerificationUrl>) -> bool {
    url.is_none() || url.as_ref().map(|u| u.as_str().is_empty()).unwrap_or(true)
}

fn is_none_content(content: &OptionalTextContent) -> bool {
    matches!(content, OptionalTextContent::None)
}

#[allow(dead_code)]
impl FeatureSpec {
    /// 驗證功能規格
    pub fn validate(&self) -> Result<(), ValidationError> {
        // 基本驗證已在類型轉換時完成
        // 這裡可以加入額外的業務邏輯驗證
        Ok(())
    }

    /// 取得有效的專案類型（向後相容：若 is_frontend=true 則視為 Frontend）
    pub fn effective_project_type(&self) -> ProjectType {
        if self.is_frontend {
            ProjectType::Frontend
        } else {
            self.project_type
        }
    }

    /// 取得驗證 URL 字串（如果有的話）
    pub fn verification_url_str(&self) -> &str {
        self.verification_url
            .as_ref()
            .map(|u| u.as_str())
            .unwrap_or("")
    }

    /// 是否有遠端驗證環境
    pub fn has_verification_env(&self) -> bool {
        self.options.has_verification_env
    }

    /// 是否需要部署
    pub fn needs_deployment(&self) -> bool {
        self.options.needs_deployment
    }

    /// 是否需要本地驗證
    pub fn needs_local_validation(&self) -> bool {
        self.options.needs_local_validation
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
    fn test_verification_url_valid() {
        assert!(VerificationUrl::new("").is_ok());
        assert!(VerificationUrl::new("https://example.com").is_ok());
        assert!(VerificationUrl::new("http://localhost:3000").is_ok());
    }

    #[test]
    fn test_verification_url_invalid() {
        assert!(VerificationUrl::new("ftp://example.com").is_err());
        assert!(VerificationUrl::new("example.com").is_err());
    }

    #[test]
    fn test_text_content_to_block() {
        let single = TextContent::Single("Hello".to_string());
        assert_eq!(single.to_block(false), "Hello");

        let multiple = TextContent::Multiple(vec!["Line 1".to_string(), "Line 2".to_string()]);
        assert_eq!(multiple.to_block(true), "- Line 1\n- Line 2");
        assert_eq!(multiple.to_block(false), "Line 1\nLine 2");
    }

    // =========================================================================
    // ProjectType 測試
    // =========================================================================

    #[test]
    fn test_project_type_all_variants() {
        assert_eq!(ProjectType::ALL.len(), 8);
        assert!(ProjectType::ALL.contains(&ProjectType::Frontend));
        assert!(ProjectType::ALL.contains(&ProjectType::Backend));
        assert!(ProjectType::ALL.contains(&ProjectType::Fullstack));
        assert!(ProjectType::ALL.contains(&ProjectType::Cli));
        assert!(ProjectType::ALL.contains(&ProjectType::Library));
        assert!(ProjectType::ALL.contains(&ProjectType::SystemLevel));
        assert!(ProjectType::ALL.contains(&ProjectType::Algorithm));
        assert!(ProjectType::ALL.contains(&ProjectType::Infra));
    }

    #[test]
    fn test_project_type_needs_browser() {
        assert!(ProjectType::Frontend.needs_browser());
        assert!(ProjectType::Fullstack.needs_browser());
        assert!(!ProjectType::Backend.needs_browser());
        assert!(!ProjectType::Cli.needs_browser());
        assert!(!ProjectType::Library.needs_browser());
        assert!(!ProjectType::SystemLevel.needs_browser());
        assert!(!ProjectType::Algorithm.needs_browser());
        assert!(!ProjectType::Infra.needs_browser());
    }

    #[test]
    fn test_project_type_typically_needs_deployment() {
        assert!(ProjectType::Frontend.typically_needs_deployment());
        assert!(ProjectType::Backend.typically_needs_deployment());
        assert!(ProjectType::Fullstack.typically_needs_deployment());
        assert!(ProjectType::Infra.typically_needs_deployment());
        assert!(!ProjectType::Cli.typically_needs_deployment());
        assert!(!ProjectType::Library.typically_needs_deployment());
        assert!(!ProjectType::SystemLevel.typically_needs_deployment());
        assert!(!ProjectType::Algorithm.typically_needs_deployment());
    }

    #[test]
    fn test_project_type_typically_needs_verification_env() {
        assert!(ProjectType::Frontend.typically_needs_verification_env());
        assert!(ProjectType::Backend.typically_needs_verification_env());
        assert!(ProjectType::Fullstack.typically_needs_verification_env());
        assert!(ProjectType::Infra.typically_needs_verification_env());
        assert!(!ProjectType::Cli.typically_needs_verification_env());
        assert!(!ProjectType::Library.typically_needs_verification_env());
        assert!(!ProjectType::SystemLevel.typically_needs_verification_env());
        assert!(!ProjectType::Algorithm.typically_needs_verification_env());
    }

    #[test]
    fn test_project_type_role_description() {
        assert!(ProjectType::Frontend
            .role_description()
            .contains("frontend"));
        assert!(ProjectType::Backend.role_description().contains("backend"));
        assert!(ProjectType::Cli.role_description().contains("CLI"));
        assert!(ProjectType::Library.role_description().contains("library"));
        assert!(ProjectType::SystemLevel
            .role_description()
            .contains("systems"));
        assert!(ProjectType::Algorithm
            .role_description()
            .contains("algorithms"));
        assert!(ProjectType::Infra
            .role_description()
            .contains("infrastructure"));
    }

    #[test]
    fn test_project_type_e2e_instructions() {
        assert!(ProjectType::Frontend.e2e_instructions().contains("browser"));
        assert!(ProjectType::Backend.e2e_instructions().contains("HTTP"));
        assert!(ProjectType::Cli.e2e_instructions().contains("exit codes"));
        assert!(ProjectType::Library
            .e2e_instructions()
            .contains("test suite"));
        assert!(ProjectType::SystemLevel
            .e2e_instructions()
            .contains("memory safety"));
        assert!(ProjectType::Algorithm
            .e2e_instructions()
            .contains("benchmark"));
        assert!(ProjectType::Infra.e2e_instructions().contains("terraform"));
    }

    #[test]
    fn test_project_type_specific_requirements() {
        assert!(ProjectType::Frontend
            .specific_requirements()
            .contains("Chrome DevTools"));
        assert!(ProjectType::Backend
            .specific_requirements()
            .contains("API contracts"));
        assert!(ProjectType::Cli
            .specific_requirements()
            .contains("Exit codes"));
        assert!(ProjectType::Library
            .specific_requirements()
            .contains("semantic versioning"));
        assert!(ProjectType::SystemLevel
            .specific_requirements()
            .contains("Memory safety"));
        assert!(ProjectType::Algorithm
            .specific_requirements()
            .contains("Time complexity"));
        assert!(ProjectType::Infra
            .specific_requirements()
            .contains("idempotent"));
    }

    #[test]
    fn test_project_type_artifacts_description() {
        assert!(ProjectType::Frontend
            .artifacts_description()
            .contains("E2E_PLAN"));
        assert!(ProjectType::Backend
            .artifacts_description()
            .contains("API_SPEC"));
        assert!(ProjectType::Cli.artifacts_description().contains("USAGE"));
        assert!(ProjectType::Library
            .artifacts_description()
            .contains("PUBLISH_CHECKLIST"));
        assert!(ProjectType::SystemLevel
            .artifacts_description()
            .contains("SAFETY_CHECKLIST"));
        assert!(ProjectType::Algorithm
            .artifacts_description()
            .contains("BENCHMARK_PLAN"));
        assert!(ProjectType::Infra
            .artifacts_description()
            .contains("DRIFT_DETECTION"));
    }

    #[test]
    fn test_project_type_display() {
        assert_eq!(format!("{}", ProjectType::Frontend), "frontend");
        assert_eq!(format!("{}", ProjectType::Backend), "backend");
        assert_eq!(format!("{}", ProjectType::Fullstack), "fullstack");
        assert_eq!(format!("{}", ProjectType::Cli), "cli");
        assert_eq!(format!("{}", ProjectType::Library), "library");
        assert_eq!(format!("{}", ProjectType::SystemLevel), "systemlevel");
        assert_eq!(format!("{}", ProjectType::Algorithm), "algorithm");
        assert_eq!(format!("{}", ProjectType::Infra), "infra");
    }

    // =========================================================================
    // FeatureOptions 測試
    // =========================================================================

    #[test]
    fn test_feature_options_default() {
        let opts = FeatureOptions::default();
        assert!(!opts.has_verification_env);
        assert!(opts.needs_local_validation);
        assert!(!opts.needs_deployment);
        assert!(opts.custom_validation.is_none());
        assert!(opts.test_command.is_none());
        assert!(opts.build_command.is_none());
    }

    #[test]
    fn test_feature_options_from_frontend() {
        let opts = FeatureOptions::from_project_type(ProjectType::Frontend);
        assert!(opts.has_verification_env);
        assert!(opts.needs_local_validation);
        assert!(opts.needs_deployment);
    }

    #[test]
    fn test_feature_options_from_cli() {
        let opts = FeatureOptions::from_project_type(ProjectType::Cli);
        assert!(!opts.has_verification_env);
        assert!(opts.needs_local_validation);
        assert!(!opts.needs_deployment);
    }

    #[test]
    fn test_feature_options_from_library() {
        let opts = FeatureOptions::from_project_type(ProjectType::Library);
        assert!(!opts.has_verification_env);
        assert!(opts.needs_local_validation);
        assert!(!opts.needs_deployment);
    }

    #[test]
    fn test_feature_options_from_backend() {
        let opts = FeatureOptions::from_project_type(ProjectType::Backend);
        assert!(opts.has_verification_env);
        assert!(opts.needs_local_validation);
        assert!(opts.needs_deployment);
    }

    #[test]
    fn test_feature_options_from_algorithm() {
        let opts = FeatureOptions::from_project_type(ProjectType::Algorithm);
        assert!(!opts.has_verification_env);
        assert!(opts.needs_local_validation);
        assert!(!opts.needs_deployment);
    }

    // =========================================================================
    // FeatureSpec 測試
    // =========================================================================

    fn create_test_feature_spec(project_type: ProjectType) -> FeatureSpec {
        FeatureSpec {
            feature_key: FeatureKey::new("test-feature").unwrap(),
            feature_name: FeatureName::new("Test Feature").unwrap(),
            project_type,
            options: FeatureOptions::from_project_type(project_type),
            context: TextContent::Single("Test context".to_string()),
            requirements: TextContent::Single("Test requirements".to_string()),
            acceptance_criteria: TextContent::Single("Test criteria".to_string()),
            verification_url: Some(VerificationUrl::new("https://example.com").unwrap()),
            int_credentials: OptionalTextContent::None,
            is_frontend: false,
        }
    }

    #[test]
    fn test_feature_spec_effective_project_type() {
        let spec = create_test_feature_spec(ProjectType::Backend);
        assert_eq!(spec.effective_project_type(), ProjectType::Backend);
    }

    #[test]
    fn test_feature_spec_is_frontend_backward_compat() {
        let mut spec = create_test_feature_spec(ProjectType::Backend);
        spec.is_frontend = true;
        // 當 is_frontend = true 時，effective_project_type 應返回 Frontend
        assert_eq!(spec.effective_project_type(), ProjectType::Frontend);
    }

    #[test]
    fn test_feature_spec_verification_url_str() {
        let spec = create_test_feature_spec(ProjectType::Frontend);
        assert_eq!(spec.verification_url_str(), "https://example.com");
    }

    #[test]
    fn test_feature_spec_verification_url_str_empty() {
        let mut spec = create_test_feature_spec(ProjectType::Cli);
        spec.verification_url = None;
        assert_eq!(spec.verification_url_str(), "");
    }

    #[test]
    fn test_feature_spec_has_verification_env() {
        let spec = create_test_feature_spec(ProjectType::Frontend);
        assert!(spec.has_verification_env());

        let spec_cli = create_test_feature_spec(ProjectType::Cli);
        assert!(!spec_cli.has_verification_env());
    }

    #[test]
    fn test_feature_spec_needs_deployment() {
        let spec = create_test_feature_spec(ProjectType::Backend);
        assert!(spec.needs_deployment());

        let spec_lib = create_test_feature_spec(ProjectType::Library);
        assert!(!spec_lib.needs_deployment());
    }

    #[test]
    fn test_feature_spec_needs_local_validation() {
        let spec = create_test_feature_spec(ProjectType::Algorithm);
        assert!(spec.needs_local_validation());
    }

    #[test]
    fn test_feature_spec_validate() {
        let spec = create_test_feature_spec(ProjectType::Frontend);
        assert!(spec.validate().is_ok());
    }
}
