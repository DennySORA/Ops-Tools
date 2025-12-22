use super::error::Result;
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

/// 外部命令執行器 trait（用於依賴反轉）
#[allow(dead_code)]
pub trait CommandExecutor {
    /// 執行外部命令並回傳輸出
    fn execute(&self, command: &str, args: &[String]) -> Result<String>;
}

/// 進度回調 trait
#[allow(dead_code)]
pub trait ProgressReporter {
    fn report(&self, current: u64, total: u64, message: &str);
    fn finish(&self, message: &str);
}
