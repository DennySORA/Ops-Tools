use super::result::OperationResult;
use std::path::{Path, PathBuf};

/// 檔案系統掃描器 trait
pub trait FileScanner {
    /// 掃描指定目錄，回傳符合條件的路徑列表
    fn scan(&self, root: &Path) -> Vec<PathBuf>;
}

/// 檔案清理器 trait
pub trait FileCleaner {
    /// 清理指定的檔案/目錄列表
    fn clean(&self, items: Vec<PathBuf>) -> Vec<OperationResult>;
}
