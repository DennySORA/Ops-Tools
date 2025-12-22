# Tools - DevOps 工具集

基於 SOLID 原則和 Clean Code 實踐的 Rust CLI 工具集。

## 功能特色

### 1. Terraform/Terragrunt 快取清理

智能清理 Terraform 和 Terragrunt 產生的快取檔案：

- `.terragrunt-cache` 目錄
- `.terraform` 目錄
- `.terraform.lock.hcl` 檔案
- 自動過濾重複的子路徑，避免重複刪除

### 2. AI 程式碼助手工具升級

批次升級 AI 程式碼助手工具：

| 套件 | 名稱 |
|------|------|
| `@anthropic-ai/claude-code` | Claude Code |
| `@openai/codex` | OpenAI Codex |
| `@google/gemini-cli` | Google Gemini CLI |

### 3. 高風險套件安全掃描

高效能並行掃描專案中的套件依賴：

- 支援任意套件名稱搜尋
- 忽略 `node_modules`、`.git`、`target` 等目錄
- 自動跳過二進位檔案
- 顯示匹配的檔案路徑和行號

### 4. MCP 工具管理

管理 Claude 和 Codex CLI 的 MCP 伺服器：

| MCP 工具 | 說明 |
|----------|------|
| `sequential-thinking` | 循序思考 |
| `context7` | 文檔查詢 |
| `chrome-devtools` | 瀏覽器開發工具 |
| `kubernetes` | K8s 管理 |
| `github` | GitHub 整合 |

**需要的環境變數**（編譯時設定於 `.env`）：
- `GITHUB_PERSONAL_ACCESS_TOKEN`
- `GITHUB_HOST`
- `CONTEXT7_API_KEY`

## 安裝

```bash
# 編譯
cargo build --release

# 設定環境變數（可選，用於 MCP 管理功能）
cp .env.example .env
# 編輯 .env 填入你的憑證
```

## 使用

```bash
cargo run
# 或
./target/release/tools
```

選擇功能選單：
1. 清理 Terraform/Terragrunt 快取檔案
2. 升級 AI 程式碼助手工具
3. 掃描高風險套件（安全檢測）
4. 管理 MCP 工具（Claude/Codex）

## 架構設計

### 分層架構

```
src/
├── main.rs                              # 入口點
│
├── core/                                # 核心抽象層
│   ├── error.rs                         # 統一錯誤類型 (OperationError)
│   ├── result.rs                        # 操作結果 (OperationResult, OperationStats)
│   ├── traits.rs                        # 核心 trait (FileScanner, FileCleaner)
│   └── path_utils.rs                    # 路徑工具函數
│
├── features/                            # 功能模組
│   ├── terraform_cleaner/               # Terraform 清理
│   │   ├── scanner.rs                   # 掃描器
│   │   ├── cleaner.rs                   # 清理器
│   │   └── service.rs                   # 服務層
│   ├── tool_upgrader/                   # AI 工具升級
│   │   ├── tools.rs                     # 工具定義
│   │   └── upgrader.rs                  # 升級執行器
│   ├── package_scanner/                 # 套件掃描
│   │   └── scanner.rs                   # 內容掃描器
│   └── mcp_manager/                     # MCP 管理
│       ├── config.rs                    # 環境變數配置
│       ├── tools.rs                     # MCP 工具定義
│       └── executor.rs                  # CLI 執行器
│
└── ui/                                  # UI 層
    ├── console.rs                       # 控制台輸出
    ├── progress.rs                      # 進度條
    └── prompts.rs                       # 使用者輸入
```

### SOLID 原則應用

#### 1. 單一職責原則 (SRP)
每個模組只負責一件事：
- `scanner.rs` - 掃描檔案
- `cleaner.rs` - 刪除檔案
- `service.rs` - 協調業務邏輯
- `console.rs` - 輸出顯示
- `prompts.rs` - 使用者輸入

#### 2. 開放封閉原則 (OCP)
透過 trait 擴展功能：
```rust
pub trait FileScanner {
    fn scan(&self, root: &Path) -> Vec<PathBuf>;
}

pub trait FileCleaner {
    fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult>;
}
```

#### 3. 依賴反轉原則 (DIP)
Service 依賴抽象而非具體實作：
```rust
pub struct TerraformCleanerService<S: FileScanner, C: FileCleaner> {
    scanner: S,
    cleaner: C,
}
```

### 錯誤處理

統一的錯誤類型：
```rust
pub enum OperationError {
    Io { path: String, source: io::Error },
    Command { command: String, message: String },
    Config { key: String, message: String },
    Validation(String),
    Cancelled,
}
```

## 測試

```bash
# 執行所有測試
cargo test

# 執行特定模組測試
cargo test terraform_cleaner
cargo test package_scanner

# 執行 clippy 檢查
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

目前有 **44 個測試**，全部通過。

## 擴展新功能

要新增一個清理功能，只需要：

1. 在 `features/` 建立新模組目錄
2. 實作 `FileScanner` trait（如需掃描）
3. 實作 `FileCleaner` trait（如需清理）或使用現有的
4. 建立 `service.rs` 協調邏輯
5. 建立 `mod.rs` 的 `run()` 公開函數
6. 在 `main.rs` 註冊功能

## 依賴項

| 依賴 | 用途 |
|------|------|
| `dialoguer` | 互動式 CLI |
| `walkdir` | 目錄遍歷 |
| `colored` | 終端顏色 |
| `indicatif` | 進度條 |
| `rayon` | 並行處理 |
| `ignore` | 快速檔案遍歷（支援 .gitignore） |

## 授權

MIT License
