mod cleaner;
mod scanner;
mod service;

use crate::i18n::{self, keys};
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
            console.error(&crate::tr!(keys::TERRAFORM_CURRENT_DIR_FAILED,
                error = e
            ));
            return;
        }
    };

    execute(&current_dir, &console, &prompts);
}

fn execute(root: &Path, console: &Console, prompts: &Prompts) {
    console.info(i18n::t(keys::TERRAFORM_SCAN_START));
    console.info(&crate::tr!(keys::TERRAFORM_SCAN_DIR,
        path = root.display()
    ));

    let scanner = TerraformScanner::new();
    let cleaner = Cleaner::new();
    let service = TerraformCleanerService::new(scanner, cleaner);

    // 1. 掃描
    let scan_result = service.scan(root);

    if scan_result.is_empty() {
        console.warning(i18n::t(keys::TERRAFORM_NO_CACHE));
        return;
    }

    // 2. 顯示找到的項目
    console.show_paths_with_title(
        &crate::tr!(keys::TERRAFORM_FOUND_ITEMS,
            count = scan_result.count()
        ),
        &scan_result.items,
        |item| {
            if item.is_dir() {
                i18n::t(keys::TERRAFORM_ITEM_DIR)
            } else {
                i18n::t(keys::TERRAFORM_ITEM_FILE)
            }
        },
    );

    // 3. 確認刪除
    if !prompts.confirm_with_options(i18n::t(keys::TERRAFORM_CONFIRM_DELETE), false) {
        console.warning(i18n::t(keys::TERRAFORM_DELETE_CANCELLED));
        return;
    }

    // 4. 執行刪除
    let clean_result = service.clean(scan_result.items);

    // 5. 顯示結果
    for result in &clean_result.results {
        if result.success {
            console.success_item(&crate::tr!(keys::TERRAFORM_DELETED,
                path = result.path.display()
            ));
        } else if let Some(err) = &result.error {
            console.error_item(
                &crate::tr!(keys::TERRAFORM_DELETE_FAILED,
                    path = result.path.display()
                ),
                err,
            );
        }
    }

    // 6. 顯示統計
    console.show_summary(
        i18n::t(keys::TERRAFORM_SUMMARY_TITLE),
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
