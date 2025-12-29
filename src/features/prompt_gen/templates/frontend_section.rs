//! 前端設計強化區塊

/// 前端設計強化區塊 - 條件式插入模板 03
///
/// 佔位符：
/// - `{IS_FRONTEND}`: 是否為前端功能
pub const FRONTEND_SECTION: &str = r#"## （條件式）前端設計強化（只有當 {IS_FRONTEND} = true 才執行）
若此功能是前端，請額外遵守：
- 使用 mcp chrome-devtools 輔助開發，必須測試不跑版、無錯誤、UI/UX 優秀
- Use the frontend-design skill
- 先詳細分析配色與設計方案（暗色系、有質感、低調專業、舒適配色）
- 重新設計元件顏色：高辨識度但不花俏；布局合理且簡潔
- 大量互動與動畫：
  - hover/focus/active feedback
  - 送出按鈕必須有明確回饋（loading/success/error）
- 移除不需要的 code/邏輯，同時 build 必須通過
"#;
