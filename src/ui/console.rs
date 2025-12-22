use colored::Colorize;
use std::path::PathBuf;

/// 控制台輸出工具
pub struct Console;

impl Console {
    pub fn new() -> Self {
        Self
    }

    // === 基本訊息輸出 ===

    pub fn info(&self, message: &str) {
        println!("{}", message.cyan());
    }

    pub fn success(&self, message: &str) {
        println!("{}", message.green());
    }

    pub fn warning(&self, message: &str) {
        println!("{}", message.yellow());
    }

    pub fn error(&self, message: &str) {
        eprintln!("{} {}", "錯誤:".red().bold(), message);
    }

    // === 結構化輸出 ===

    pub fn header(&self, title: &str) {
        println!("\n{}", "=".repeat(50).cyan());
        println!("{}", title.bold().cyan());
        println!("{}", "=".repeat(50).cyan());
    }

    pub fn separator(&self) {
        println!("{}", "-".repeat(50).bright_black());
    }

    pub fn blank_line(&self) {
        println!();
    }

    // === 列表輸出 ===

    pub fn list_item(&self, icon: &str, message: &str) {
        println!("  {} {}", icon, message);
    }

    pub fn success_item(&self, message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    pub fn error_item(&self, message: &str, error: &str) {
        eprintln!("{} {} - {}", "✗".red(), message, error.red());
    }

    // === 路徑列表 ===

    pub fn show_paths(&self, paths: &[PathBuf], type_fn: impl Fn(&PathBuf) -> &str) {
        for path in paths {
            let item_type = type_fn(path);
            println!("  {} {}", item_type.blue(), path.display());
        }
    }

    pub fn show_paths_with_title(
        &self,
        title: &str,
        paths: &[PathBuf],
        type_fn: impl Fn(&PathBuf) -> &str,
    ) {
        println!("\n{}", title);
        self.show_paths(paths, type_fn);
    }

    // === 統計與摘要 ===

    pub fn show_summary(&self, title: &str, success: usize, failed: usize) {
        println!("\n{}", "=".repeat(50).cyan());
        println!(
            "{} 成功: {}, 失敗: {}",
            title.green(),
            success.to_string().green(),
            failed.to_string().red()
        );
        println!("{}", "=".repeat(50).cyan());
    }

    pub fn show_progress(&self, current: usize, total: usize, message: &str) {
        println!("[{}/{}] {}", current, total, message);
    }

    // === 階段性輸出 ===

    pub fn phase(&self, current: usize, total: usize, message: &str) {
        println!(
            "{} 階段 {}/{}: {}",
            "=>".cyan().bold(),
            current,
            total,
            message
        );
    }

    pub fn stat(&self, label: &str, value: &str) {
        println!("   {}: {}", label, value.yellow());
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_creation() {
        let console = Console::new();
        console.info("test info");
        console.success("test success");
        console.warning("test warning");
    }

    #[test]
    fn test_show_paths() {
        let console = Console::new();
        let paths = vec![PathBuf::from("/test/path")];
        console.show_paths(&paths, |_| "DIR");
    }
}
