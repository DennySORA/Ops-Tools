mod scanner;

use crate::ui::{Console, Prompts};
use scanner::ContentScanner;
use std::env;
use std::path::PathBuf;
use std::time::Instant;

/// 執行高風險套件掃描功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header("高風險套件安全掃描器");
    console.blank_line();

    // 取得要搜尋的套件名稱
    let package_name = prompts.input("請輸入要搜尋的套件名稱");

    if package_name.is_empty() {
        console.error("套件名稱不能為空");
        return;
    }

    // 取得掃描目錄
    let scan_dir = prompts.input_with_default("請輸入要掃描的目錄", ".");
    let scan_path = if scan_dir == "." {
        env::current_dir().unwrap()
    } else {
        PathBuf::from(&scan_dir)
    };

    if !scan_path.exists() || !scan_path.is_dir() {
        console.error(&format!("無效的目錄: {}", scan_path.display()));
        return;
    }

    // 開始掃描
    console.blank_line();
    console.phase(1, 2, "收集檔案");
    console.stat("掃描目錄", &scan_path.display().to_string());
    console.stat("搜尋目標", &package_name);

    let start_time = Instant::now();

    let scanner = ContentScanner::new(package_name.clone());
    let (files, file_count) = scanner.collect_files(&scan_path);

    console.stat("找到檔案數", &file_count.to_string());

    console.blank_line();
    console.phase(2, 2, "並行搜尋內容");

    let result = scanner.scan(&scan_path);
    let elapsed = start_time.elapsed();

    // 顯示結果
    console.blank_line();
    console.header("掃描結果報告");
    console.blank_line();

    console.stat("耗時", &format!("{:.2}秒", elapsed.as_secs_f64()));
    console.stat("掃描檔案數", &result.stats.files_scanned.to_string());

    if elapsed.as_secs_f64() > 0.0 {
        let files_per_sec = result.stats.files_scanned as f64 / elapsed.as_secs_f64();
        console.stat("效能", &format!("{:.0} 檔案/秒", files_per_sec));
    }

    console.blank_line();

    if result.matches.is_empty() {
        console.success("太好了！未發現該套件！");
    } else {
        console.warning(&format!("發現 {} 處匹配！", result.matches.len()));
        console.blank_line();

        for m in &result.matches {
            console.error_item(
                &format!("{}:{}", m.file_path.display(), m.line_number),
                &truncate_line(&m.line_content, 60),
            );
        }

        console.blank_line();
        console.warning("建議：請檢查這些檔案並移除可疑的套件");
    }

    // 忽略未使用的變數警告
    let _ = files;
}

fn truncate_line(line: &str, max_len: usize) -> String {
    if line.len() > max_len {
        format!("{}...", &line[..max_len])
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_line() {
        assert_eq!(truncate_line("short", 10), "short");
        assert_eq!(truncate_line("this is a long line", 10), "this is a ...");
    }
}
