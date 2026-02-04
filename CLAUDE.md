# 開發守則

請遵守以下的規則，將每個部件拆成 component，每個職責都簡單，每一個功能都必須完整。
以下要求，請一個一個確認，自己要理解、驗證、分析、設計、規劃、執行，修復所有錯誤。
使用 sequential-thinking 來規劃。

## 設計規範（必須遵守）

### SOLID（必須）

- **S（SRP）單一職責**：一個模組/類別/函式只負責一件事；改動理由應該只有一個。
- **O（OCP）開放封閉**：新增行為用擴充（介面/策略/注入），避免修改既有核心邏輯造成回歸。
- **L（LSP）里氏替換**：子型別可替換父型別，不能改變原契約語意（輸入/輸出/例外）。
- **I（ISP）介面隔離**：小介面、按需依賴；避免「胖介面」逼迫使用者依賴不需要的方法。
- **D（DIP）依賴反轉**：高階策略依賴抽象；IO/外部系統以介面注入，方便測試與替換。

### Clean Code（必須）

- 命名具體、可讀、可搜尋；避免縮寫與模糊詞（如 `data`, `info`, `tmp`）。
- 函式短小、單一責任；避免深層巢狀（> 2 層建議重構）。
- 以「意圖」為中心設計 API；呼叫端讀起來像自然語句。
- 避免重複（DRY），但也避免過度抽象；抽象必須能降低未來變更成本。
- 註解只補「為什麼」，不重述「做什麼」；若註解在解釋程式在做什麼，代表程式需要更清楚。

### 程式結構

- 分層清楚：**Domain（商業邏輯）不得直接依賴 Infrastructure（DB/HTTP/Queue）**，透過介面隔離。
- **禁止業務邏輯散落在 Controller/Handler**：Handler 只做輸入驗證/授權/轉換/呼叫 use-case。
- 模組邊界清楚：跨模組只能透過公開介面，不得偷用內部細節。

### 錯誤處理與可觀測性

- 所有錯誤都要「可追踪」：具體錯誤碼/訊息、必要上下文（request id / user id / correlation id）。
- 例外/錯誤要分層：Domain error vs Infra error，不得混用。

### 測試（必須）

- 新增/修改行為必須附測試，至少涵蓋：
    - 主要成功路徑
    - 重要失敗路徑（權限不足、輸入非法、外部依賴失敗）
    - 邊界條件（空值、最大長度、時間邊界、並發）
- 單元測試不得依賴真實外部系統（DB/HTTP），用 stub/mock 或測試替身。
- 修 bug 必須提供「會失敗的測試」再修正（防回歸）。

## 可維護性與一致性（必須）

### 格式化與靜態檢查

- 必須啟用：formatter、linter、type check（能用就用）。
- 禁止忽略規則（`// nolint` / `eslint-disable` / `# noqa`）除非有明確理由且附註。

### Formatter（必須）

- `cargo fmt --all -- --check`

### Linter（必須：Clippy）

- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Type check（必須）

- `cargo check --workspace --all-targets --all-features`

### Test（必須）

- `cargo test --workspace --all-features`

## 安全規範

### 機敏資料與憑證

- 憑證/金鑰/Token **不得寫進程式碼或 repo**；使用 Secret Manager/環境變數/CI secrets。
- 機敏資料在傳輸與儲存時需加密（TLS、DB encryption/at-rest）。
- 日誌、追蹤、錯誤回報不得輸出機敏資料；必要時只保留 hash 或部分遮罩。

## Skill Installer Development（必須參考文件）

**⚠️ 重要：** 開發 Skill Installer 擴充功能前，**必須先閱讀並遵循** [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md)。

該文件包含：
- 完整的 Extension 定義格式與欄位說明
- Marketplace-based 插件安裝架構（git clone、symlink、JSON registries）
- `${CLAUDE_PLUGIN_ROOT}` 變數轉換機制
- Hooks 轉換流程（Claude → Gemini）
- 依賴安裝（npm/bun）流程
- CLI 相容性矩陣與限制說明

### 必要步驟

1. **讀取文件**：先閱讀 `docs/SKILL_INSTALLER.md` 了解完整架構
2. **Extension 定義**：在 `src/features/skill_installer/tools.rs` 新增 Extension
3. **i18n 支援**：在 `src/i18n/mod.rs` 及所有 locale 檔案新增翻譯
4. **CLI 相容性**：正確設定 `cli_support`、`skill_subpath`、`command_file`、`has_hooks`
5. **Marketplace 插件**：如需完整 repo 結構，設定 `marketplace_name`、`marketplace_plugin_path`、`version`

### 重要限制

| 功能 | Claude | Codex | Gemini |
|-----|--------|-------|--------|
| Hooks | ✅ 完整支援 | ❌ 不支援 | ✅ 轉換支援 |
| Marketplace plugins | ✅ 完整支援 | ❌ 不支援 | ✅ 變數轉換 |
| `${CLAUDE_PLUGIN_ROOT}` | ✅ 原生支援 | ❌ 不支援 | ⚠️ 轉為絕對路徑 |
| `allowed-tools` | ✅ 支援 | ❌ 移除 | ❌ 移除 |

### 測試驗證

```bash
cargo test skill_installer
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

詳細說明請參閱 [docs/SKILL_INSTALLER.md](docs/SKILL_INSTALLER.md)。
