//! 模板 03 - 重構與優化

/// 第三階段模板：重構、流程優化、品質提升
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{IS_FRONTEND}`: 是否為前端功能 ("true" 或 "false")
/// - `{FRONTEND_SECTION_BLOCK}`: 前端設計強化區塊（條件式）
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
pub const TEMPLATE_03: &str = r#"# [Feature] {FEATURE_KEY} - Refactor, Flow Optimization, Quality Improvements (Include Frontend Redesign If Needed)

You are continuing the same feature context. Based on the previous two prompts and the artifacts in `features/{FEATURE_KEY}/`, refactor and optimize.

## Tooling Requirements (Must Follow)
- Use structured planning for the refactor strategy (plan first, break into reversible steps).
- If this is a frontend feature ({IS_FRONTEND} = true), use browser developer tools or available automation for development and validation.

{STATE_REQUIREMENT}

## Engineering Rules (Must Follow, self-audit at the end)
(Follow your existing standards: SOLID / Clean Code / layering / observability / testing / consistency / security)
- SOLID (SRP/OCP/LSP/ISP/DIP) is required
- Clean Code is required
- Domain must not depend directly on Infrastructure
- Handlers must not contain business logic
- Error layering and traceability are required
- Testing: success/failure/edge cases; mock external dependencies; write a failing test before fixing a bug
- Use formatter/linter/typecheck when available
- Credentials must not be committed to the codebase

{FRONTEND_SECTION_BLOCK}

## Execution Flow (Must Follow)
1) Structured planning: identify pain points, choose cuts, split into reversible steps
2) Implement refactors + add tests + ensure build/test/lint/typecheck pass
3) If validation is needed: deploy to the verification environment
4) Update:
   - `REFACTOR_NOTES.md` (required)
   - (Frontend) `DESIGN_GUIDE.md` (required)
   - `STATE.md` and set STATUS to `P3_REFACTORED_DEPLOYED`

## Final Response Format (Required)
- Refactor summary, risks, and rollback plan
- Quality status (build/test/lint/typecheck)
- STATE.md status (including STATUS)
- Guidance for Prompt #4 regression E2E
"#;
