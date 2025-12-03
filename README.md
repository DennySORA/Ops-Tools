# Tools - 智能清理工具集

基於 SOLID 原則和 Clean Code 實踐的 Rust 工具集。

## 功能特色

### 🧹 Terraform/Terragrunt 快取清理

智能清理 Terraform 和 Terragrunt 產生的快取檔案：

- ✅ `.terragrunt-cache` 目錄
- ✅ `.terraform` 目錄
- ✅ `.terraform.lock.hcl` 檔案

### 🚀 Terragrunt 批次 Apply

批次執行多個目錄的 `terragrunt apply`，取代不安全的 shell script：

- ✅ 自動掃描子目錄
- ✅ 可配置跳過特定目錄（預設：monitoring, kafka-provision）
- ✅ 進度追蹤與即時輸出
- ✅ 詳細的成功/失敗統計
- ✅ 失敗時自動停止（可配置）
- ✅ 使用者確認機制
- ✅ 完整的錯誤處理

### 🔐 Base64 轉換

- ✅ 貼上任意文字後立即轉成 Base64
- ✅ 支援多行輸入（Ctrl+D 結束輸入，Windows 按 Ctrl+Z 後 Enter）
- ✅ 直接在終端輸出結果

### 🎯 智能去重

**新功能**：自動過濾重複的子路徑，避免重複刪除。

#### 範例

當掃描到以下結構：
```
/project/.terragrunt-cache
/project/.terragrunt-cache/sub1/.terraform
/project/.terragrunt-cache/sub1/.terraform.lock.hcl
/project/module/.terraform
/project/module/.terraform.lock.hcl
```

系統會智能過濾，只保留：
```
/project/.terragrunt-cache         # 父目錄
/project/module/.terraform          # 獨立檔案
/project/module/.terraform.lock.hcl # 獨立檔案
```

**原理**：當刪除父目錄 `.terragrunt-cache` 時，其所有子項目會自動被刪除，因此不需要單獨列出。

### 📊 進度追蹤

- 實時顯示掃描進度
- 實時顯示刪除進度
- 進度條視覺化

### 📋 詳細報告

- 成功/失敗統計
- 成功率計算
- 詳細錯誤資訊
- 顏色標示（成功綠色、失敗紅色、警告黃色）

## 安裝

```bash
cargo build --release
```

## 使用

```bash
cargo run
# 或
./target/release/tools
```

選擇需要的功能：
- "清理 Terraform/Terragrunt 快取檔案"
- "批次執行 Terragrunt Apply"
- "貼上內容轉 Base64"

## 架構設計

### SOLID 原則應用

#### 1. 單一職責原則 (SRP)
每個模組只負責一件事：
- `scanner.rs` - 掃描檔案
- `cleaner.rs` - 刪除檔案
- `ui.rs` - 使用者互動
- `progress.rs` - 進度追蹤
- `report.rs` - 報告生成
- `path_utils.rs` - 路徑處理

#### 2. 開放封閉原則 (OCP)
透過 trait 擴展功能：
```rust
pub trait Scanner {
    fn scan(&self, root: &Path) -> Vec<PathBuf>;
}

pub trait Cleaner {
    fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult>;
}
```

#### 3. 里氏替換原則 (LSP)
所有實作 trait 的類型都可替換使用。

#### 4. 介面隔離原則 (ISP)
介面最小化，只包含必要方法。

#### 5. 依賴反轉原則 (DIP)
依賴抽象而非具體實作：
```rust
pub struct TerraformCleanService<S: Scanner, C: Cleaner> {
    scanner: S,
    cleaner: C,
    // ...
}
```

## 模組結構

```
src/
├── main.rs                      # 主程式
├── component/                   # 功能模組
│   └── clear_terrform/
│       ├── mod.rs              # 服務協調器
│       ├── scanner.rs          # 掃描器（含智能去重）
│       └── cleaner.rs          # 清理器
└── tools/                       # 共用工具（可重用）
    ├── traits.rs               # 通用介面定義
    ├── ui.rs                   # UI 工具
    ├── progress.rs             # 進度追蹤
    ├── report.rs               # 報告生成
    ├── path_utils.rs           # 路徑工具（智能去重）
    └── remove.rs               # 檔案刪除
```

## 核心功能

### 路徑智能去重 (`path_utils.rs`)

提供以下函數：

#### `is_subpath(child: &Path, parent: &Path) -> bool`
檢查一個路徑是否是另一個路徑的子路徑。

```rust
let parent = PathBuf::from("/a/b");
let child = PathBuf::from("/a/b/c");
assert!(is_subpath(&child, &parent)); // true
```

#### `filter_subpaths(paths: Vec<PathBuf>) -> Vec<PathBuf>`
過濾掉被其他路徑包含的子路徑。

```rust
let paths = vec![
    PathBuf::from("/a/b"),
    PathBuf::from("/a/b/c"),
    PathBuf::from("/a/b/c/d"),
];
let filtered = filter_subpaths(paths);
// 結果: ["/a/b"]
```

#### `count_filtered_subpaths(original: &[PathBuf], filtered: &[PathBuf]) -> usize`
統計被過濾掉的路徑數量。

### 使用者介面 (`ui.rs`)

提供豐富的 UI 方法：

```rust
let ui = UserInterface::new();

ui.info("資訊訊息");
ui.success("成功訊息");
ui.warning("警告訊息");
ui.error("錯誤訊息");

// 確認對話框
if ui.confirm_with_options("確定要刪除嗎？", false) {
    // 執行刪除
}

// 顯示項目列表
ui.show_items_with_title("找到的項目:", &items, |item| {
    if item.is_dir() { "目錄" } else { "檔案" }
});
```

### 進度追蹤 (`progress.rs`)

```rust
let progress = ProgressTracker::new(100, "處理中");
for i in 0..100 {
    // 處理工作
    progress.inc();
}
progress.finish_with_message("完成！");
```

### 報告生成 (`report.rs`)

```rust
let reporter = ReportGenerator::new();

// 顯示即時反饋
for result in &results {
    reporter.show_result_inline(result);
}

// 顯示詳細報告
reporter.show_operation_report(&results);
```

## 測試

```bash
# 執行所有測試
cargo test

# 執行特定測試
cargo test path_utils
cargo test terragrunt_apply
```

目前有 24 個測試，全部通過 ✅

## 擴展新功能

要新增一個清理功能，只需要：

1. 實作 `Scanner` trait
2. （可選）實作 `Cleaner` trait 或使用現有的 `FileCleaner`
3. 使用共用的 `UserInterface`、`ProgressTracker`、`ReportGenerator`
4. 在 `main.rs` 註冊功能

範例請參考 `USAGE_EXAMPLES.md`。

## 依賴項

- `dialoguer` - 互動式 CLI
- `walkdir` - 目錄遍歷
- `colored` - 終端顏色
- `indicatif` - 進度條

## 授權

MIT License

## 貢獻

歡迎提交 Issue 和 Pull Request！
