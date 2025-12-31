//! 前端設計強化區塊

/// 前端設計強化區塊 - 條件式插入模板 03
///
/// 佔位符：
/// - `{IS_FRONTEND}`: 是否為前端功能
pub const FRONTEND_SECTION: &str = r#"## (Conditional) Frontend Design and UX (Only when {IS_FRONTEND} = true)
If this feature includes frontend work, also follow:
- Use browser developer tools or available automation to validate; ensure no layout breakage, no errors, and reasonable UX
- Define a clear visual direction (typography, color, hierarchy, spacing) and keep it consistent and readable
- Use colors that are easy to distinguish without being harsh; keep layout simple and information hierarchy clear
- Interactions and motion must provide explicit feedback:
  - hover/focus/active
  - primary actions must show loading/success/error states
- Remove unnecessary code/logic while keeping build green
"#;
