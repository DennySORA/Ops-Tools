use crate::tools::traits::{OperationResult, OperationType, Statistics};
use crate::tools::ui::UserInterface;

pub struct ReportGenerator {
    ui: UserInterface,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {
            ui: UserInterface::new(),
        }
    }

    pub fn generate_statistics(&self, results: &[OperationResult]) -> Statistics {
        Statistics::from_results(results)
    }

    pub fn show_summary(&self, stats: &Statistics, title: &str) {
        self.ui.show_summary(title, stats.success, stats.failed);
    }

    pub fn show_detailed_report(&self, results: &[OperationResult], title: &str) {
        let stats = self.generate_statistics(results);

        // 顯示成功的項目
        let successful = results.iter().filter(|r| r.success).collect::<Vec<_>>();
        if !successful.is_empty() {
            self.ui
                .info(&format!("\n✓ 成功 ({} 項):", successful.len()));
            for result in successful {
                self.ui.success_item(&result.path.display().to_string());
            }
        }

        // 顯示失敗的項目
        let failed = results.iter().filter(|r| !r.success).collect::<Vec<_>>();
        if !failed.is_empty() {
            self.ui.warning(&format!("\n✗ 失敗 ({} 項):", failed.len()));
            for result in failed {
                if let Some(error) = &result.error {
                    self.ui
                        .error_item(&result.path.display().to_string(), error);
                }
            }
        }

        // 顯示統計摘要
        self.show_summary(&stats, title);

        // 顯示成功率
        if stats.total > 0 {
            let rate = stats.success_rate();
            let rate_str = format!("成功率: {:.1}%", rate);
            if rate >= 90.0 {
                self.ui.success(&rate_str);
            } else if rate >= 70.0 {
                self.ui.warning(&rate_str);
            } else {
                self.ui.error(&rate_str);
            }
        }
    }

    pub fn show_operation_report(&self, results: &[OperationResult]) {
        let operation_name = if !results.is_empty() {
            match results[0].operation_type {
                OperationType::Delete => "刪除",
                OperationType::Create => "建立",
                OperationType::Update => "更新",
                OperationType::Read => "讀取",
            }
        } else {
            "操作"
        };

        self.show_detailed_report(results, operation_name);
    }

    pub fn show_result_inline(&self, result: &OperationResult) {
        if result.success {
            self.ui.success_item(&result.path.display().to_string());
        } else if let Some(error) = &result.error {
            self.ui
                .error_item(&result.path.display().to_string(), error);
        }
    }

    #[allow(dead_code)]
    pub fn show_grouped_report(&self, results: &[OperationResult]) {
        use std::collections::HashMap;

        let mut grouped: HashMap<OperationType, Vec<&OperationResult>> = HashMap::new();
        for result in results {
            grouped
                .entry(result.operation_type)
                .or_default()
                .push(result);
        }

        for (op_type, group_results) in grouped {
            let op_name = match op_type {
                OperationType::Delete => "刪除操作",
                OperationType::Create => "建立操作",
                OperationType::Update => "更新操作",
                OperationType::Read => "讀取操作",
            };

            self.ui.separator();
            self.ui.header(op_name);

            let group_vec: Vec<OperationResult> =
                group_results.iter().map(|&r| r.clone()).collect();
            self.show_detailed_report(&group_vec, op_name);
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_report_generator_creation() {
        let generator = ReportGenerator::new();
        let results = vec![
            OperationResult::success(PathBuf::from("/test1"), OperationType::Delete),
            OperationResult::failure(
                PathBuf::from("/test2"),
                OperationType::Delete,
                "Error".to_string(),
            ),
        ];

        let stats = generator.generate_statistics(&results);
        assert_eq!(stats.total, 2);
        assert_eq!(stats.success, 1);
        assert_eq!(stats.failed, 1);
    }
}
