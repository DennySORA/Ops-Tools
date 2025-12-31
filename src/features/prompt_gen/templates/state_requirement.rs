//! 狀態機需求區塊模板

/// 狀態需求區塊 - 用於所有模板的狀態管理
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
pub const STATE_REQUIREMENT_BLOCK: &str = r#"
## State Machine (Required, for enforcing step order)
You must maintain the following fields in `features/{FEATURE_KEY}/STATE.md` (create if missing) and update them at the end of each prompt:

- FEATURE_KEY: {FEATURE_KEY}
- STATUS: <one of>
  - P1_DONE_DEPLOYED
  - P2_E2E_PASSED
  - P3_REFACTORED_DEPLOYED
  - READY

Required end status for each prompt:
- Prompt #1 end: STATUS must be `P1_DONE_DEPLOYED`
- Prompt #2 end: STATUS must be `P2_E2E_PASSED`
- Prompt #3 end: STATUS must be `P3_REFACTORED_DEPLOYED`
- Prompt #4 end: STATUS must be `READY`

If you cannot meet the requirement (for example, failed acceptance items remain), keep the previous STATUS and clearly record the reason and next steps in STATE.md.
"#;
