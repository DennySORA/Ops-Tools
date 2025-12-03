use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 目錄掃描器 - 掃描子目錄（只掃描一層）
pub struct DirectoryScanner {
    skip_directories: Vec<String>,
}

impl DirectoryScanner {
    pub fn new(skip_directories: &[String]) -> Self {
        Self {
            skip_directories: skip_directories.to_vec(),
        }
    }

    /// 掃描指定目錄下的所有子目錄（不遞迴）
    pub fn scan(&self, base_dir: &Path) -> io::Result<Vec<PathBuf>> {
        let mut directories = Vec::new();

        // 讀取目錄內容
        let entries = fs::read_dir(base_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // 只處理目錄
            if !path.is_dir() {
                continue;
            }

            // 取得目錄名稱
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // 檢查是否需要跳過
            if self.should_skip(&dir_name) {
                println!("跳過目錄: {}", dir_name);
                continue;
            }

            directories.push(path);
        }

        // 排序以確保執行順序一致
        directories.sort();

        Ok(directories)
    }

    fn should_skip(&self, dir_name: &str) -> bool {
        self.skip_directories.iter().any(|skip| skip == dir_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let skip = vec!["monitoring".to_string(), "kafka-provision".to_string()];
        let scanner = DirectoryScanner::new(&skip);
        assert_eq!(scanner.skip_directories.len(), 2);
    }

    #[test]
    fn test_should_skip() {
        let skip = vec!["monitoring".to_string(), "kafka-provision".to_string()];
        let scanner = DirectoryScanner::new(&skip);

        assert!(scanner.should_skip("monitoring"));
        assert!(scanner.should_skip("kafka-provision"));
        assert!(!scanner.should_skip("other-dir"));
    }
}
