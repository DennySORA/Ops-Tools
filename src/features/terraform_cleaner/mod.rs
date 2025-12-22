mod cleaner;
mod scanner;
mod service;

use crate::ui::{Console, Prompts};
use cleaner::Cleaner;
use scanner::TerraformScanner;
use service::TerraformCleanerService;
use std::path::Path;

/// 執行 Terraform 快取清理功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            console.error(&format!("無法取得當前目錄: {}", e));
            return;
        }
    };

    execute(&current_dir, &console, &prompts);
}

fn execute(root: &Path, console: &Console, prompts: &Prompts) {
    console.info("開始掃描當前目錄...");
    console.info(&format!("掃描目錄: {}", root.display()));

    let scanner = TerraformScanner::new();
    let cleaner = Cleaner::new();
    let service = TerraformCleanerService::new(scanner, cleaner);

    // 1. 掃描
    let scan_result = service.scan(root);

    if scan_result.is_empty() {
        console.warning("沒有找到任何 Terraform/Terragrunt 快取檔案");
        return;
    }

    // 2. 顯示找到的項目
    console.show_paths_with_title(
        &format!("找到 {} 個項目:", scan_result.count()),
        &scan_result.items,
        |item| if item.is_dir() { "目錄" } else { "檔案" },
    );

    // 3. 確認刪除
    if !prompts.confirm_with_options("確定要刪除這些項目嗎？", false) {
        console.warning("已取消刪除操作");
        return;
    }

    // 4. 執行刪除
    let clean_result = service.clean(scan_result.items);

    // 5. 顯示結果
    for result in &clean_result.results {
        if result.success {
            console.success_item(&format!("已刪除: {}", result.path.display()));
        } else if let Some(err) = &result.error {
            console.error_item(&format!("刪除失敗: {}", result.path.display()), err);
        }
    }

    // 6. 顯示統計
    console.show_summary(
        "清理完成",
        clean_result.stats.success,
        clean_result.stats.failed,
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // 確保模組可以編譯
    }
}
