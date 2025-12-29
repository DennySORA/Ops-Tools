//! 模板 02 - INT E2E 驗證

/// 第二階段模板：INT E2E 驗證（直到符合預期）
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
pub const TEMPLATE_02_FIXED: &str = r#"# [Fixed] INT E2E 驗證（直到符合預期）

你正在延續同一輪（Feature）的 session，請使用前一個 prompt 的記憶與 repo 產物來做 E2E。
目標：在 INT 環境用真實瀏覽器把功能完整驗證到符合預期；若不符合，請修復、重新部署、再測到通過。

## 強制工具要求（必須）
- 你必須使用 `sequential-thinking` 來規劃 E2E 執行與修復策略。
- 你必須使用 `mcp chrome-devtools` 打開 INT 網站，進行全面端到端測試（包含 console/network/error 等檢查）。

{STATE_REQUIREMENT}

## 操作規範（必須）
1) 讀取並遵循：
   - `features/{FEATURE_KEY}/E2E_PLAN.md`
   - `features/{FEATURE_KEY}/ACCEPTANCE.md`
   - `features/{FEATURE_KEY}/RUNBOOK_INT.md`
   - `features/{FEATURE_KEY}/STATE.md`

2) 用 `mcp chrome-devtools` 在 INT 執行測試（逐步檢查畫面、互動、console/network）
3) 若失敗：修復 → 測試 → 部署 INT → 再驗證（直到通過）
4) 通過後：
   - 更新 `E2E_RUN_REPORT.md`
   - 更新 `STATE.md`，並把 STATUS 設為 `P2_E2E_PASSED`

## 停止條件
- 只有在 `ACCEPTANCE.md` 全部打勾、且 E2E 核心流程與重要失敗情境皆通過，才能宣告完成此 prompt。

## 最終回覆格式（必須）
- E2E 通過摘要（對應 `ACCEPTANCE.md`）
- STATE.md 狀態（包含 STATUS）
- 下一步（Prompt #3）建議的重構/優化方向
"#;
