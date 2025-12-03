use std::path::PathBuf;
use crate::tools::remove;
use crate::tools::progress::ProgressTracker;
use crate::tools::traits::{Cleaner, OperationResult, OperationType};

pub struct FileCleaner;

impl FileCleaner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl Cleaner for FileCleaner {
    fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult> {
        let mut results = Vec::new();
        let total = items.len() as u64;
        let progress = ProgressTracker::new(total, "刪除中");

        for item in items {
            let result = match remove::remove_item(&item) {
                Ok(_) => OperationResult::success(item, OperationType::Delete),
                Err(e) => OperationResult::failure(item, OperationType::Delete, e.to_string()),
            };

            results.push(result);
            progress.inc();
        }

        progress.finish_with_message("刪除完成");
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;

    #[test]
    fn test_file_cleaner_creation() {
        let _cleaner = FileCleaner::new();
    }

    #[test]
    fn test_file_cleaner_removes_terragrunt_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        let terragrunt_cache = temp_dir.path().join(".terragrunt-cache/nested");
        fs::create_dir_all(&terragrunt_cache).unwrap();
        fs::write(terragrunt_cache.join("dummy.txt"), "test data").unwrap();

        let cleaner = FileCleaner::new();
        let results = cleaner.clean(vec![temp_dir.path().join(".terragrunt-cache")]);

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(!temp_dir.path().join(".terragrunt-cache").exists());
    }
}
