//! 模板 04 - INT E2E 回歸驗證

/// 第四階段模板：INT E2E 回歸驗證（重構後）
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
pub const TEMPLATE_04_FIXED: &str = r#"# [Fixed] INT E2E 回歸驗證（重構後，直到符合預期）

你正在延續同一輪（Feature）的 session。
目標：針對重構後版本，在 INT 用真實瀏覽器做完整回歸 E2E，直到 `ACCEPTANCE.md` 全部符合。

## 強制工具要求（必須）
- 你必須使用 `sequential-thinking` 規劃回歸測試與風險點。
- 你必須使用 `mcp chrome-devtools` 打開 INT 網站做全面 E2E（含 console/network/error）。

{STATE_REQUIREMENT}

## 必須遵循的檔案
- `features/{FEATURE_KEY}/E2E_PLAN.md`
- `features/{FEATURE_KEY}/ACCEPTANCE.md`
- `features/{FEATURE_KEY}/STATE.md`
- `features/{FEATURE_KEY}/REFACTOR_NOTES.md`
- （若存在）`features/{FEATURE_KEY}/DESIGN_GUIDE.md`

## 執行方式（必須）
1) sequential-thinking：列回歸面、決定順序
2) chrome-devtools：照 E2E_PLAN 跑全流程，檢查 console/network/效能/跑版/互動
3) 失敗即修復迴圈：最小修復 → 測試 → 部署 INT → 再驗證
4) 全部通過後：
   - 更新 `E2E_RUN_REPORT.md`
   - 更新 `STATE.md`，並把 STATUS 設為 `READY`

## 最終回覆格式（必須）
- 回歸 E2E 通過摘要（對應 `ACCEPTANCE.md`）
- STATE.md 狀態（包含 STATUS=READY）
- 本輪 Feature 結案摘要
"#;
