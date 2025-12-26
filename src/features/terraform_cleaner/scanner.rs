use crate::core::path_utils;
use crate::core::FileScanner;
use crate::i18n::{self, keys};
use crate::ui::Progress;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Terraform/Terragrunt 快取掃描器
pub struct TerraformScanner {
    targets: Vec<String>,
}

impl TerraformScanner {
    pub fn new() -> Self {
        Self {
            targets: vec![
                ".terragrunt-cache".to_string(),
                ".terraform.lock.hcl".to_string(),
                ".terraform".to_string(),
            ],
        }
    }

    #[allow(dead_code)]
    pub fn with_targets(targets: Vec<String>) -> Self {
        Self { targets }
    }

    fn should_include(&self, file_name: &str) -> bool {
        self.targets.iter().any(|target| file_name == target)
    }
}

impl Default for TerraformScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl FileScanner for TerraformScanner {
    fn scan(&self, root: &Path) -> Vec<PathBuf> {
        let mut found_items = Vec::new();

        let total_entries: u64 = WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .count() as u64;

        let progress = Progress::new(total_entries, i18n::t(keys::TERRAFORM_PROGRESS_SCANNING));

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy();

            if self.should_include(&file_name) {
                found_items.push(entry.path().to_path_buf());
            }

            progress.inc();
        }

        progress.finish_with_message(i18n::t(keys::TERRAFORM_PROGRESS_SCANNED));

        path_utils::filter_subpaths(found_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_should_include() {
        let scanner = TerraformScanner::new();
        assert!(scanner.should_include(".terraform"));
        assert!(scanner.should_include(".terragrunt-cache"));
        assert!(scanner.should_include(".terraform.lock.hcl"));
        assert!(!scanner.should_include("other_file.txt"));
    }

    #[test]
    fn test_custom_targets() {
        let scanner = TerraformScanner::with_targets(vec!["custom_target".to_string()]);
        assert!(scanner.should_include("custom_target"));
        assert!(!scanner.should_include(".terraform"));
    }

    #[test]
    fn test_scan_filters_children() {
        let temp_dir = tempfile::tempdir().unwrap();
        let terragrunt_cache = temp_dir.path().join(".terragrunt-cache");

        let nested_terraform = terragrunt_cache.join("module/.terraform");
        fs::create_dir_all(&nested_terraform).unwrap();
        fs::write(nested_terraform.join("dummy.txt"), "test").unwrap();

        let scanner = TerraformScanner::new();
        let mut results = scanner.scan(temp_dir.path());
        results.sort();

        assert_eq!(results, vec![terragrunt_cache]);
    }
}
