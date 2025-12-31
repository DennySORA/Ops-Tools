//! 模板 01 - 需求、實作、部署（INT）

/// 第一階段模板：需求與交付
///
/// 佔位符：
/// - `{FEATURE_KEY}`: 功能鍵值
/// - `{FEATURE_NAME}`: 功能名稱
/// - `{CONTEXT_BLOCK}`: 上下文區塊
/// - `{REQUIREMENTS_BLOCK}`: 需求區塊
/// - `{ACCEPTANCE_BLOCK}`: 驗收條件區塊
/// - `{VERIFICATION_URL}`: 驗證 URL（可為空）
/// - `{INT_CREDENTIALS_BLOCK}`: INT 環境憑證
/// - `{STATE_REQUIREMENT}`: 狀態需求區塊
pub const TEMPLATE_01: &str = r#"# [Feature] {FEATURE_KEY} - 需求、實作、部署（INT）

你是一位資深全端工程師/Tech Lead，在此 repo 內完成此功能並部署到 INT 環境。
本輪工作（Prompt #1~#4）需要保持記憶與連貫性；但請同時把關鍵狀態寫進檔案，以便 runner 續跑。

## 強制工具要求（必須遵守）
- 你必須使用 `sequential-thinking` 來做規劃（先規劃再動手）。

## 輸入資訊
- Feature Key: {FEATURE_KEY}
- Feature Name: {FEATURE_NAME}

- Context:
{CONTEXT_BLOCK}

- Requirements:
{REQUIREMENTS_BLOCK}

- Acceptance Criteria:
{ACCEPTANCE_BLOCK}

- Verification URL (optional): {VERIFICATION_URL}
- INT Credentials / Login Method (if needed):
{INT_CREDENTIALS_BLOCK}

{STATE_REQUIREMENT}

## 產出與落檔（必須）
在 `features/{FEATURE_KEY}/` 產出並維護以下檔案（若不存在就建立）：
1) `STATE.md`：本輪狀態（本 prompt 的決策、已完成項、待辦、風險、如何在 int 驗證；含 STATUS 欄位）
2) `E2E_PLAN.md`：可在瀏覽器執行的端到端測試清單（步驟要非常具體）
3) `ACCEPTANCE.md`：把驗收條件轉成可檢查項（checklist）
4) `RUNBOOK_INT.md`：如何部署/如何回滾/需要的設定
5) `CHANGELOG.md`：本功能變更摘要（面向 reviewer）

## 執行流程（請嚴格照順序）
1) 使用 `sequential-thinking`：
   - 讀 repo 結構、關聯模組與既有行為
   - 澄清需求與邊界（缺資訊時：做合理假設並寫入 `STATE.md`，不要卡住）
   - 設計方案：資料流/模組邊界/錯誤處理/觀測性/測試策略
   - 拆出可交付的最小步驟（每一步可 build、可測、可回退）

2) 實作：
   - 實作功能與必要的後端/前端改動（依 repo 慣例）
   - 補齊必要測試（單元/整合，至少涵蓋主要成功路徑與重要失敗路徑）
   - 確保 lint/format/typecheck/build 通過

3) 部署到 INT：
   - 依 repo 的部署方式完成部署
   - 將部署方式、版本/commit、設定差異寫入 `RUNBOOK_INT.md` 與 `STATE.md`

4) 收尾：
   - 更新 `E2E_PLAN.md`（讓下一個 Prompt #2 可以直接照做）
   - 更新 `STATE.md`，並把 STATUS 設為 `P1_DONE_INT_DEPLOYED`

## 重要約束
- 憑證/金鑰/Token 不得寫進程式碼或 repo。需要時請使用環境變數或既有 secret 機制。

## 最終回覆格式（必須）
- 本 prompt 完成摘要（含已部署到 INT 的證據：版本/commit/tag、部署位置）
- STATE.md 狀態（包含 STATUS）
- 下一步（Prompt #2）執行指引（對應 `E2E_PLAN.md`）
"#;
