//! 模板 01 - 需求、實作、部署（驗證環境）

/// 第一階段模板：需求與交付
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{FEATURE_NAME}`: 功能名稱
/// - `{CONTEXT_BLOCK}`: 上下文區塊
/// - `{REQUIREMENTS_BLOCK}`: 需求區塊
/// - `{ACCEPTANCE_BLOCK}`: 驗收條件區塊
/// - `{VERIFICATION_URL}`: 驗證 URL（可為空）
/// - `{VERIFICATION_CREDENTIALS_BLOCK}`: 驗證環境憑證
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
pub const TEMPLATE_01: &str = r#"# [Feature] {FEATURE_KEY} - Requirements, Implementation, Deployment (Verification Environment)

You are a senior full-stack engineer/Tech Lead. Implement this feature in the codebase and deploy it to the verification environment.
This round of work (Prompt #1-#4) must remain coherent; also write key state to files so it can be continued later.

## Planning Requirements (Must Follow)
- Complete structured planning before implementation (use any planning tool if available).

## Inputs
- Feature Key: {FEATURE_KEY}
- Feature Name: {FEATURE_NAME}

- Context:
{CONTEXT_BLOCK}

- Requirements:
{REQUIREMENTS_BLOCK}

- Acceptance Criteria:
{ACCEPTANCE_BLOCK}

- Verification URL (optional): {VERIFICATION_URL}
- Verification Credentials / Login Method (if needed):
{VERIFICATION_CREDENTIALS_BLOCK}

{STATE_REQUIREMENT}

## Required Artifacts (Must Produce)
Create and maintain the following files under `features/{FEATURE_KEY}/` (create if missing):
1) `STATE.md`: Current state (decisions, completed items, TODOs, risks, how to validate in the verification environment; include STATUS field)
2) `E2E_PLAN.md`: Browser-executable end-to-end checklist (steps must be precise)
3) `ACCEPTANCE.md`: Convert acceptance criteria into a checklist
4) `RUNBOOK_VERIFICATION.md`: How to deploy, rollback, and required configuration
5) `CHANGELOG.md`: Feature change summary (reviewer-facing)

## Execution Flow (Strict Order)
1) Structured planning:
   - Review codebase structure, related modules, and current behavior
   - Clarify requirements and boundaries (if missing info, make reasonable assumptions and record in `STATE.md`)
   - Design the solution: data flow, module boundaries, error handling, observability, test strategy
   - Break into minimal deliverable steps (each step should build, test, and be reversible)

2) Implementation:
   - Implement required backend/frontend changes following codebase conventions
   - Add necessary tests (unit/integration; cover key success and failure paths)
   - Ensure lint/format/typecheck/build pass

3) Deploy to the verification environment:
   - Follow the codebase deployment approach
   - Record deployment method, version/commit, and config differences in `RUNBOOK_VERIFICATION.md` and `STATE.md`

4) Wrap-up:
   - Update `E2E_PLAN.md` (so Prompt #2 can follow it directly)
   - Update `STATE.md` and set STATUS to `P1_DONE_DEPLOYED`

## Important Constraints
- Credentials/keys/tokens must not be committed to the codebase. Use environment variables or existing secret mechanisms.

## Final Response Format (Required)
- Summary of work completed (include evidence of verification deployment: version/commit/tag and target location)
- STATE.md status (including STATUS)
- Guidance for Prompt #2 (aligned with `E2E_PLAN.md`)
"#;
