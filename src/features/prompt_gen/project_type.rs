//! 專案類型定義
//!
//! 定義不同專案類型的測試策略和模板內容

use serde::{Deserialize, Serialize};
use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Frontend.to_string(), "frontend");
        assert_eq!(ProjectType::Backend.to_string(), "backend");
        assert_eq!(ProjectType::Cli.to_string(), "cli");
    }

    #[test]
    fn test_project_type_all() {
        assert_eq!(ProjectType::ALL.len(), 8);
    }

    #[test]
    fn test_needs_browser() {
        assert!(ProjectType::Frontend.needs_browser());
        assert!(ProjectType::Fullstack.needs_browser());
        assert!(!ProjectType::Backend.needs_browser());
        assert!(!ProjectType::Cli.needs_browser());
    }

    #[test]
    fn test_typically_needs_deployment() {
        assert!(ProjectType::Frontend.typically_needs_deployment());
        assert!(ProjectType::Backend.typically_needs_deployment());
        assert!(!ProjectType::Cli.typically_needs_deployment());
        assert!(!ProjectType::Library.typically_needs_deployment());
    }
}
