use crate::core::{FileCleaner, OperationResult, OperationType};
use crate::i18n::{self, keys};
use crate::ui::Progress;
use std::fs;
use std::path::{Path, PathBuf};

/// 檔案/目錄清理器
pub struct Cleaner;

impl Cleaner {
    pub fn new() -> Self {
        Self
    }

    fn remove_item(path: &Path) -> std::io::Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
    }
}

impl Default for Cleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl FileCleaner for Cleaner {
    fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult> {
        let mut results = Vec::new();
        let total = items.len() as u64;
        let progress = Progress::new(total, i18n::t(keys::TERRAFORM_PROGRESS_DELETING));

        for item in items {
            let result = match Self::remove_item(&item) {
                Ok(()) => OperationResult::success(item, OperationType::Delete),
                Err(e) => OperationResult::failure(item, OperationType::Delete, e.to_string()),
            };

            results.push(result);
            progress.inc();
        }

        progress.finish_with_message(i18n::t(keys::TERRAFORM_PROGRESS_DELETED));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cleaner_creation() {
        let _cleaner = Cleaner::new();
    }

    #[test]
    fn test_removes_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_dir = temp_dir.path().join(".terragrunt-cache/nested");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("dummy.txt"), "test data").unwrap();

        let cleaner = Cleaner::new();
        let results = cleaner.clean(vec![temp_dir.path().join(".terragrunt-cache")]);

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(!temp_dir.path().join(".terragrunt-cache").exists());
    }

    #[test]
    fn test_removes_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_file = temp_dir.path().join("test.txt");
        fs::write(&target_file, "test").unwrap();

        let cleaner = Cleaner::new();
        let results = cleaner.clean(vec![target_file.clone()]);

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(!target_file.exists());
    }
}
