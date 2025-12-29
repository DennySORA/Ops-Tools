//! 模板 03 - 重構與優化

/// 第三階段模板：重構、流程優化、品質提升
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{IS_FRONTEND}`: 是否為前端功能 ("true" 或 "false")
/// - `{FRONTEND_SECTION_BLOCK}`: 前端設計強化區塊（條件式）
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
pub const TEMPLATE_03: &str = r#"# [Feature] {FEATURE_KEY} - 重構、流程優化、品質提升（必要時含前端設計重做）

你正在同一輪（Feature）的 session 中，請基於前兩個 prompts 的結果與 `features/{FEATURE_KEY}/` 產物做重構與優化。

## 強制工具要求（必須）
- 你必須使用 `sequential-thinking` 來規劃重構策略（先計畫、分段、可回退）。
- 若為前端功能（{IS_FRONTEND} = true），你必須使用 `mcp chrome-devtools` 輔助開發與驗證。

{STATE_REQUIREMENT}

## 開發守則（必須逐條遵守，並在最後自我稽核）
（以下略，保持你原本規範，請照做：SOLID / Clean Code / 分層 / 可觀測性 / 測試 / 一致性 / 安全）
- SOLID（SRP/OCP/LSP/ISP/DIP）必須
- Clean Code 必須
- Domain 不得直接依賴 Infrastructure
- Handler 不得承載業務邏輯
- 錯誤分層 + 可追踪
- 測試：成功/失敗/邊界；外部依賴 mock；修 bug 先寫 failing test
- formatter/linter/typecheck 能用就用
- 憑證不得進 repo

{FRONTEND_SECTION_BLOCK}

## 執行流程（必須）
1) sequential-thinking：盤點痛點、選切點、拆成可回退步驟
2) 實作重構 + 補測試 + build/test/lint/typecheck 通過
3) 如需驗證：部署 INT
4) 更新：
   - `REFACTOR_NOTES.md`（必須）
   - （前端）`DESIGN_GUIDE.md`（必須）
   - `STATE.md` 並把 STATUS 設為 `P3_REFACTORED_INT_DEPLOYED`

## 最終回覆格式（必須）
- 重構摘要、風險與回退方案
- 品質狀態（build/test/lint/typecheck）
- STATE.md 狀態（包含 STATUS）
- 下一步（Prompt #4）回歸 E2E 指引
"#;
