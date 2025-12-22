use crate::ui::Progress;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// 掃描結果中發現的匹配
#[derive(Debug, Clone)]
pub struct ScanMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
}

/// 掃描統計資訊
#[derive(Debug, Default)]
pub struct ScanStats {
    pub files_scanned: u64,
    #[allow(dead_code)]
    pub matches_found: u64,
}

/// 掃描結果
pub struct ScanResult {
    pub matches: Vec<ScanMatch>,
    pub stats: ScanStats,
}

/// 高效能並行檔案內容掃描器
pub struct ContentScanner {
    package_name: String,
}

impl ContentScanner {
    pub fn new(package_name: String) -> Self {
        Self { package_name }
    }

    /// 掃描指定目錄（回傳結果，不直接輸出）
    pub fn scan(&self, root: &Path) -> ScanResult {
        let all_files = self.collect_all_files(root);
        let total_files = all_files.len() as u64;

        if total_files == 0 {
            return ScanResult {
                matches: Vec::new(),
                stats: ScanStats::default(),
            };
        }

        let progress = Progress::new(total_files, "搜尋中");
        let scanned_count = AtomicU64::new(0);
        let matches: Arc<Mutex<Vec<ScanMatch>>> = Arc::new(Mutex::new(Vec::new()));

        let search_term = self.package_name.to_lowercase();

        all_files.par_iter().for_each(|file_path| {
            if let Ok(content) = fs::read_to_string(file_path) {
                let content_lower = content.to_lowercase();

                if content_lower.contains(&search_term) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(&search_term) {
                            let scan_match = ScanMatch {
                                file_path: file_path.clone(),
                                line_number: line_num + 1,
                                line_content: line.trim().to_string(),
                            };
                            matches.lock().unwrap().push(scan_match);
                        }
                    }
                }
            }

            scanned_count.fetch_add(1, Ordering::Relaxed);
            progress.set_position(scanned_count.load(Ordering::Relaxed));
        });

        progress.finish_with_message("搜尋完成");

        let final_matches = Arc::try_unwrap(matches).unwrap().into_inner().unwrap();
        let stats = ScanStats {
            files_scanned: scanned_count.load(Ordering::Relaxed),
            matches_found: final_matches.len() as u64,
        };

        ScanResult {
            matches: final_matches,
            stats,
        }
    }

    /// 收集可掃描的檔案
    pub fn collect_files(&self, root: &Path) -> (Vec<PathBuf>, u64) {
        let files = self.collect_all_files(root);
        let count = files.len() as u64;
        (files, count)
    }

    fn collect_all_files(&self, root: &Path) -> Vec<PathBuf> {
        let spinner = Progress::spinner("掃描目錄中...");
        let files: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));

        WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|entry| {
                let file_name = entry.file_name().to_string_lossy();
                !matches!(
                    file_name.as_ref(),
                    "node_modules"
                        | ".git"
                        | "vendor"
                        | "target"
                        | "dist"
                        | "build"
                        | ".next"
                        | "__pycache__"
                        | ".venv"
                        | "venv"
                        | ".idea"
                        | ".vscode"
                )
            })
            .build_parallel()
            .run(|| {
                let files = Arc::clone(&files);

                Box::new(move |entry| {
                    if let Ok(entry) = entry {
                        if entry.file_type().is_some_and(|ft| ft.is_file()) {
                            let path = entry.path();

                            if !Self::is_binary_file(path) {
                                files.lock().unwrap().push(path.to_path_buf());
                            }
                        }
                    }
                    ignore::WalkState::Continue
                })
            });

        spinner.finish_and_clear();

        Arc::try_unwrap(files).unwrap().into_inner().unwrap()
    }

    fn is_binary_file(path: &Path) -> bool {
        let binary_extensions = [
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "svg", "pdf", "zip", "tar", "gz",
            "rar", "7z", "exe", "dll", "so", "dylib", "bin", "dat", "db", "sqlite", "wasm", "ttf",
            "otf", "woff", "woff2", "eot", "mp3", "mp4", "avi", "mov", "mkv", "flv", "wmv", "wav",
            "ogg", "webm", "class", "jar", "pyc", "pyo", "o", "a", "lib", "node", "lock",
        ];

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| binary_extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scanner_finds_package() {
        let temp_dir = tempdir().unwrap();

        let test_file = temp_dir.path().join("package.json");
        fs::write(
            &test_file,
            r#"{"dependencies": {"malicious-package": "1.0.0"}}"#,
        )
        .unwrap();

        let scanner = ContentScanner::new("malicious-package".to_string());
        let result = scanner.scan(temp_dir.path());

        assert_eq!(result.matches.len(), 1);
        assert!(result.matches[0].line_content.contains("malicious-package"));
    }

    #[test]
    fn test_case_insensitive() {
        let temp_dir = tempdir().unwrap();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "This contains MALICIOUS-PACKAGE here").unwrap();

        let scanner = ContentScanner::new("malicious-package".to_string());
        let result = scanner.scan(temp_dir.path());

        assert_eq!(result.matches.len(), 1);
    }
}
