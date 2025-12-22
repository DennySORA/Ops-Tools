use std::path::PathBuf;

/// 操作類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum OperationType {
    Delete,
    Create,
    Scan,
    Execute,
}

/// 單一操作的結果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OperationResult {
    pub path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub operation_type: OperationType,
}

impl OperationResult {
    pub fn success(path: PathBuf, operation_type: OperationType) -> Self {
        Self {
            path,
            success: true,
            error: None,
            operation_type,
        }
    }

    pub fn failure(path: PathBuf, operation_type: OperationType, error: String) -> Self {
        Self {
            path,
            success: false,
            error: Some(error),
            operation_type,
        }
    }

    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        self.success
    }

    #[allow(dead_code)]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// 批次操作的統計資訊
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct OperationStats {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

impl OperationStats {
    pub fn from_results(results: &[OperationResult]) -> Self {
        let success = results.iter().filter(|r| r.success).count();
        let failed = results.len() - success;

        Self {
            total: results.len(),
            success,
            failed,
        }
    }

    #[allow(dead_code)]
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f64 / self.total as f64) * 100.0
        }
    }

    #[allow(dead_code)]
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_result_success() {
        let result = OperationResult::success(PathBuf::from("/test"), OperationType::Delete);
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_operation_result_failure() {
        let result = OperationResult::failure(
            PathBuf::from("/test"),
            OperationType::Delete,
            "error message".to_string(),
        );
        assert!(result.is_failure());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_stats_from_results() {
        let results = vec![
            OperationResult::success(PathBuf::from("/p1"), OperationType::Delete),
            OperationResult::success(PathBuf::from("/p2"), OperationType::Delete),
            OperationResult::failure(PathBuf::from("/p3"), OperationType::Delete, "err".into()),
        ];

        let stats = OperationStats::from_results(&results);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.success, 2);
        assert_eq!(stats.failed, 1);
        assert!(stats.has_failures());
    }

    #[test]
    fn test_success_rate() {
        let stats = OperationStats {
            total: 4,
            success: 3,
            failed: 1,
        };
        assert_eq!(stats.success_rate(), 75.0);
    }

    #[test]
    fn test_empty_stats() {
        let stats = OperationStats::default();
        assert_eq!(stats.success_rate(), 0.0);
        assert!(!stats.has_failures());
    }
}
