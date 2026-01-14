//! 模板 02 - 驗證環境 E2E 驗證

/// 第二階段模板：驗證環境 E2E 驗證（直到符合預期）
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
/// - `{PROJECT_TYPE}`: 專案類型（frontend/backend/cli/library/infra）
/// - `{E2E_TESTING_INSTRUCTIONS}`: 根據專案類型的測試說明
pub const TEMPLATE_02_FIXED: &str = r#"# [Fixed] Verification Environment E2E Validation (Until Passing)

You are continuing the same feature context. Use the previous prompt outputs and codebase artifacts to run E2E.
Goal: validate the feature end-to-end in the verification environment. If it fails, fix, redeploy, and retest until it passes.

## Project Type
- Type: {PROJECT_TYPE}

## Tooling Requirements (Must Follow)
- Use structured planning to schedule E2E execution and remediation.
- {E2E_TESTING_INSTRUCTIONS}

{STATE_REQUIREMENT}

## Operating Rules (Must Follow)
1) Read and follow:
   - `features/{FEATURE_KEY}/E2E_PLAN.md`
   - `features/{FEATURE_KEY}/ACCEPTANCE.md`
   - `features/{FEATURE_KEY}/RUNBOOK_VERIFICATION.md`
   - `features/{FEATURE_KEY}/STATE.md`

2) Execute tests in the verification environment using appropriate tools for {PROJECT_TYPE} projects
3) If failing: fix -> test -> deploy to the verification environment -> re-validate (until passing)
4) When passing:
   - Update `E2E_RUN_REPORT.md`
   - Update `STATE.md` and set STATUS to `P2_E2E_PASSED`

## Stop Condition
- You may only complete this prompt when all items in `ACCEPTANCE.md` are checked off and the core E2E flows plus key failure scenarios pass.

## Final Response Format (Required)
- E2E pass summary (aligned with `ACCEPTANCE.md`)
- STATE.md status (including STATUS)
- Guidance for Prompt #3 refactoring/optimization
"#;
