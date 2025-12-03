mod directory_scanner;
mod executor;

use std::path::Path;
use directory_scanner::DirectoryScanner;
use executor::TerragruntExecutor;
use crate::tools::ui::UserInterface;
use crate::tools::report::ReportGenerator;

/// Terragrunt Apply 服務配置
pub struct TerragruntApplyConfig {
    /// 要跳過的目錄名稱
    pub skip_directories: Vec<String>,
    /// 是否在失敗時停止
    pub stop_on_failure: bool,
}

impl Default for TerragruntApplyConfig {
    fn default() -> Self {
        Self {
            skip_directories: vec!["monitoring".to_string(), "kafka-provision".to_string()],
            stop_on_failure: true,
        }
    }
}

/// Terragrunt Apply 服務 - 批次執行 terragrunt apply
pub struct TerragruntApplyService {
    config: TerragruntApplyConfig,
    ui: UserInterface,
    reporter: ReportGenerator,
}

impl TerragruntApplyService {
    pub fn new(config: TerragruntApplyConfig) -> Self {
        Self {
            config,
            ui: UserInterface::new(),
            reporter: ReportGenerator::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(TerragruntApplyConfig::default())
    }

    /// 執行批次 apply
    pub fn execute(&self, base_dir: &Path) {
        self.ui.header("Terragrunt 批次 Apply");
        self.ui.info(&format!("基礎目錄: {}", base_dir.display()));
        self.ui.info(&format!(
            "預設跳過目錄: {}",
            self.config.skip_directories.join(", ")
        ));
        self.ui.separator();

        // 1. 掃描子目錄
        let scanner = DirectoryScanner::new(&self.config.skip_directories);
        let directories = match scanner.scan(base_dir) {
            Ok(dirs) => dirs,
            Err(e) => {
                self.ui.error(&format!("掃描目錄失敗: {}", e));
                return;
            }
        };

        if directories.is_empty() {
            self.ui.warning("沒有找到需要處理的子目錄");
            return;
        }

        // 2. 顯示找到的目錄並讓使用者選擇要跳過的
        self.ui.info(&format!("\n找到 {} 個目錄:", directories.len()));
        for (idx, dir) in directories.iter().enumerate() {
            self.ui.list_item(
                &format!("{}.", idx + 1),
                &dir.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            );
        }

        // 3. 讓使用者選擇要跳過的目錄
        let selected_directories = self.select_directories_to_process(&directories);

        if selected_directories.is_empty() {
            self.ui.warning("沒有選擇任何目錄");
            return;
        }

        // 4. 顯示將要處理的目錄
        self.ui.separator();
        self.ui.info(&format!("將要處理 {} 個目錄:", selected_directories.len()));
        for (idx, dir) in selected_directories.iter().enumerate() {
            self.ui.list_item(
                &format!("{}.", idx + 1),
                &dir.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            );
        }

        // 5. 確認執行
        if !self.ui.confirm_with_options("\n確定要執行 terragrunt apply 嗎？", false) {
            self.ui.warning("操作已取消");
            return;
        }

        // 6. 執行 terragrunt apply
        let executor = TerragruntExecutor::new();
        let total = selected_directories.len();
        let mut results = Vec::new();

        self.ui.separator();
        self.ui.info("開始執行 terragrunt apply...\n");

        for (idx, dir) in selected_directories.iter().enumerate() {
            let dir_name = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            self.ui.show_progress(idx + 1, total, &format!("處理: {}", dir_name));

            let result = executor.apply(dir);
            results.push((dir_name.clone(), result.clone()));

            match result {
                Ok(_) => {
                    self.ui.success_item(&format!("{} - Apply 成功", dir_name));
                }
                Err(ref e) => {
                    self.ui.error_item(&format!("{} - Apply 失敗", dir_name), e);

                    if self.config.stop_on_failure {
                        self.ui.error("\n遇到錯誤，停止執行");
                        break;
                    }
                }
            }
        }

        // 7. 顯示統計報告
        self.show_summary(&results);
    }

    /// 讓使用者選擇要處理的目錄（可以跳過某些目錄）
    fn select_directories_to_process(&self, directories: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
        use dialoguer::MultiSelect;

        self.ui.separator();
        self.ui.info("請選擇要處理的目錄 (空白鍵選擇/取消選擇，Enter 確認):");

        // 準備選項：目錄名稱列表
        let dir_names: Vec<String> = directories
            .iter()
            .map(|dir| {
                dir.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        // 使用 MultiSelect 讓使用者勾選
        let selections = MultiSelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("選擇要處理的目錄（預設全選）")
            .items(&dir_names)
            .defaults(&vec![true; dir_names.len()]) // 預設全選
            .interact()
            .unwrap_or_default();

        // 根據選擇結果返回對應的目錄
        selections
            .into_iter()
            .map(|idx| directories[idx].clone())
            .collect()
    }

    fn show_summary(&self, results: &[(String, Result<(), String>)]) {
        let total = results.len();
        let success = results.iter().filter(|(_, r)| r.is_ok()).count();
        let failed = results.iter().filter(|(_, r)| r.is_err()).count();

        self.ui.separator();
        self.ui.show_summary("Terragrunt Apply", success, failed);

        if failed > 0 {
            self.ui.warning(&format!(
                "\n成功率: {:.1}%",
                (success as f64 / total as f64) * 100.0
            ));

            // 顯示失敗的目錄
            self.ui.error("\n失敗的目錄:");
            for (dir_name, result) in results {
                if let Err(e) = result {
                    self.ui.error_item(dir_name, e);
                }
            }
        } else {
            self.ui.success(&format!("\n✓ 所有 {} 個目錄都執行成功！", total));
        }
    }
}

/// 公開函數 - 執行 terragrunt apply
pub fn batch_apply() {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            let ui = UserInterface::new();
            ui.error(&format!("無法取得當前目錄: {}", e));
            return;
        }
    };

    let service = TerragruntApplyService::with_default_config();
    service.execute(&current_dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TerragruntApplyConfig::default();
        assert_eq!(config.skip_directories.len(), 2);
        assert!(config.stop_on_failure);
    }

    #[test]
    fn test_service_creation() {
        let service = TerragruntApplyService::with_default_config();
        assert_eq!(service.config.skip_directories.len(), 2);
    }
}
