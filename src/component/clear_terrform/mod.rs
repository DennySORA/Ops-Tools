mod cleaner;
mod scanner;

use crate::tools::report::ReportGenerator;
use crate::tools::traits::{Cleaner, Scanner};
use crate::tools::ui::UserInterface;
use cleaner::FileCleaner;
use scanner::TerraformScanner;
use std::path::Path;

/// Terraform 清理服務 - 協調所有組件完成清理任務
/// 應用了 Facade 模式來簡化複雜的子系統互動
pub struct TerraformCleanService<S: Scanner, C: Cleaner> {
    scanner: S,
    cleaner: C,
    ui: UserInterface,
    reporter: ReportGenerator,
}

impl<S: Scanner, C: Cleaner> TerraformCleanService<S, C> {
    /// 建立新的清理服務實例
    pub fn new(scanner: S, cleaner: C) -> Self {
        Self {
            scanner,
            cleaner,
            ui: UserInterface::new(),
            reporter: ReportGenerator::new(),
        }
    }

    /// 執行清理流程
    pub fn execute(&self, root: &Path) {
        // 1. 顯示開始訊息
        self.ui.info("開始掃描當前目錄...");
        self.ui.info(&format!("掃描目錄: {}", root.display()));

        // 2. 掃描目標項目
        let found_items = self.scanner.scan(root);

        // 3. 檢查是否找到項目
        if found_items.is_empty() {
            self.ui
                .warning("沒有找到任何 Terraform/Terragrunt 快取檔案");
            return;
        }

        // 4. 顯示找到的項目
        self.ui.show_items_with_title(
            &format!("找到 {} 個項目:", found_items.len()),
            &found_items,
            |item| if item.is_dir() { "目錄" } else { "檔案" },
        );

        // 5. 確認是否刪除
        if !self
            .ui
            .confirm_with_options("確定要刪除這些項目嗎？", false)
        {
            self.ui.warning("已取消刪除操作");
            return;
        }

        // 6. 執行刪除
        let results = self.cleaner.clean(found_items);

        // 7. 顯示即時反饋
        for result in &results {
            self.reporter.show_result_inline(result);
        }

        // 8. 顯示詳細報告
        self.reporter.show_operation_report(&results);
    }
}

/// 公開的清理函數 - 保持向後兼容
pub fn clean_terraform_cache() {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            let ui = UserInterface::new();
            ui.error(&format!("無法取得當前目錄: {}", e));
            return;
        }
    };

    // 使用預設的掃描器和清理器
    let scanner = TerraformScanner::new();
    let cleaner = FileCleaner::new();
    let service = TerraformCleanService::new(scanner, cleaner);

    service.execute(&current_dir);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::OperationResult;
    use std::path::PathBuf;

    // Mock Scanner for testing
    struct MockScanner {
        items: Vec<PathBuf>,
    }

    impl Scanner for MockScanner {
        fn scan(&self, _root: &Path) -> Vec<PathBuf> {
            self.items.clone()
        }
    }

    // Mock Cleaner for testing
    struct MockCleaner;

    impl Cleaner for MockCleaner {
        fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult> {
            use crate::tools::traits::OperationType;
            items
                .into_iter()
                .map(|path| OperationResult::success(path, OperationType::Delete))
                .collect()
        }
    }

    #[test]
    fn test_service_creation() {
        let scanner = MockScanner { items: vec![] };
        let cleaner = MockCleaner;
        let _service = TerraformCleanService::new(scanner, cleaner);
    }
}
