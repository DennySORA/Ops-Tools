//! 模板 04 - 驗證環境 E2E 回歸驗證

/// 第四階段模板：驗證環境 E2E 回歸驗證（重構後）
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
/// - `{PROJECT_TYPE}`: 專案類型（frontend/backend/cli/library/infra）
/// - `{E2E_TESTING_INSTRUCTIONS}`: 根據專案類型的測試說明
pub const TEMPLATE_04_FIXED: &str = r#"# [Fixed] Verification Environment E2E Regression (Post-Refactor, Until Passing)

You are continuing the same feature context.
Goal: for the refactored version, run full regression E2E in the verification environment until `ACCEPTANCE.md` is fully satisfied.

## Project Type
- Type: {PROJECT_TYPE}

## Tooling Requirements (Must Follow)
- Use structured planning to identify regression scope and risks.
- {E2E_TESTING_INSTRUCTIONS}

{STATE_REQUIREMENT}

## Required Files
- `features/{FEATURE_KEY}/E2E_PLAN.md`
- `features/{FEATURE_KEY}/ACCEPTANCE.md`
- `features/{FEATURE_KEY}/STATE.md`
- `features/{FEATURE_KEY}/REFACTOR_NOTES.md`
- (If present) `features/{FEATURE_KEY}/DESIGN_GUIDE.md`

## Execution Steps (Must Follow)
1) Structured planning: list regression areas and decide order
2) Run the full E2E plan using appropriate tools for {PROJECT_TYPE} projects
3) Fix loop on failure: minimal fix -> test -> deploy to the verification environment -> re-validate
4) When all pass:
   - Update `E2E_RUN_REPORT.md`
   - Update `STATE.md` and set STATUS to `READY`

## Final Response Format (Required)
- Regression E2E pass summary (aligned with `ACCEPTANCE.md`)
- STATE.md status (include STATUS=READY)
- Feature close-out summary
"#;
