use std::path::{Path, PathBuf};

pub trait Scanner {
    fn scan(&self, root: &Path) -> Vec<PathBuf>;
}

pub trait Cleaner {
    fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult>;
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub operation_type: OperationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum OperationType {
    Delete,
    Create,
    Update,
    Read,
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
}

#[derive(Debug, Default, Clone)]
pub struct Statistics {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

impl Statistics {
    pub fn from_results(results: &[OperationResult]) -> Self {
        let success = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        Self {
            total: results.len(),
            success,
            failed,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f64 / self.total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_result_success() {
        let result = OperationResult::success(PathBuf::from("/test"), OperationType::Delete);
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.operation_type, OperationType::Delete);
    }

    #[test]
    fn test_operation_result_failure() {
        let result = OperationResult::failure(
            PathBuf::from("/test"),
            OperationType::Delete,
            "Error".to_string(),
        );
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_statistics_from_results() {
        let results = vec![
            OperationResult::success(PathBuf::from("/p1"), OperationType::Delete),
            OperationResult::success(PathBuf::from("/p2"), OperationType::Delete),
            OperationResult::failure(
                PathBuf::from("/p3"),
                OperationType::Delete,
                "Error".to_string(),
            ),
        ];

        let stats = Statistics::from_results(&results);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.success, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.success_rate(), 66.66666666666666);
    }
}
