use crate::core::{FileCleaner, FileScanner, OperationResult, OperationStats};
use std::path::Path;

/// 掃描結果
pub struct ScanResult {
    pub items: Vec<std::path::PathBuf>,
    #[allow(dead_code)]
    pub filtered_count: usize,
}

impl ScanResult {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }
}

/// 清理結果
pub struct CleanResult {
    pub results: Vec<OperationResult>,
    pub stats: OperationStats,
}

impl CleanResult {
    pub fn from_results(results: Vec<OperationResult>) -> Self {
        let stats = OperationStats::from_results(&results);
        Self { results, stats }
    }
}

/// Terraform 清理服務
pub struct TerraformCleanerService<S: FileScanner, C: FileCleaner> {
    scanner: S,
    cleaner: C,
}

impl<S: FileScanner, C: FileCleaner> TerraformCleanerService<S, C> {
    pub fn new(scanner: S, cleaner: C) -> Self {
        Self { scanner, cleaner }
    }

    /// 掃描快取檔案
    pub fn scan(&self, root: &Path) -> ScanResult {
        let items = self.scanner.scan(root);
        ScanResult {
            items,
            filtered_count: 0,
        }
    }

    /// 清理指定的檔案
    pub fn clean(&self, items: Vec<std::path::PathBuf>) -> CleanResult {
        let results = self.cleaner.clean(items);
        CleanResult::from_results(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{FileCleaner, FileScanner, OperationResult, OperationType};
    use std::path::{Path, PathBuf};

    struct MockScanner {
        items: Vec<PathBuf>,
    }

    impl FileScanner for MockScanner {
        fn scan(&self, _root: &Path) -> Vec<PathBuf> {
            self.items.clone()
        }
    }

    struct MockCleaner;

    impl FileCleaner for MockCleaner {
        fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult> {
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
        let _service = TerraformCleanerService::new(scanner, cleaner);
    }

    #[test]
    fn test_scan_empty() {
        let scanner = MockScanner { items: vec![] };
        let cleaner = MockCleaner;
        let service = TerraformCleanerService::new(scanner, cleaner);

        let result = service.scan(Path::new("/test"));
        assert!(result.is_empty());
    }

    #[test]
    fn test_clean_success() {
        let items = vec![PathBuf::from("/test/file")];
        let scanner = MockScanner {
            items: items.clone(),
        };
        let cleaner = MockCleaner;
        let service = TerraformCleanerService::new(scanner, cleaner);

        let result = service.clean(items);
        assert_eq!(result.stats.success, 1);
        assert_eq!(result.stats.failed, 0);
    }
}
