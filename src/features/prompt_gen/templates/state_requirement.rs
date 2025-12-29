//! 狀態機需求區塊模板

/// 狀態需求區塊 - 用於所有模板的狀態管理
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
pub const STATE_REQUIREMENT_BLOCK: &str = r#"
## 狀態機（必須遵守，供 Runner 強制順序性）
你必須在 `features/{FEATURE_KEY}/STATE.md` 維護以下欄位（若不存在就建立），並在每個 prompt 結束時更新：

- FEATURE_KEY: {FEATURE_KEY}
- STATUS: <one of>
  - P1_DONE_INT_DEPLOYED
  - P2_E2E_PASSED
  - P3_REFACTORED_INT_DEPLOYED
  - READY

本 prompt 的結束狀態要求：
- Prompt #1 結束：STATUS 必須設為 `P1_DONE_INT_DEPLOYED`
- Prompt #2 結束：STATUS 必須設為 `P2_E2E_PASSED`
- Prompt #3 結束：STATUS 必須設為 `P3_REFACTORED_INT_DEPLOYED`
- Prompt #4 結束：STATUS 必須設為 `READY`

如果你做不到（例如仍有失敗的驗收項），你必須維持原狀態並在 STATE.md 清楚寫出原因與下一步。
"#;
