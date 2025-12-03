mod scanner;

use colored::*;
use scanner::ContentScanner;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

/// 讀取使用者輸入（支援貼上）
fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// 掃描高風險套件的主入口函數
pub fn scan_risky_packages() {
    println!("\n{}", "═══════════════════════════════════════════".cyan());
    println!(
        "{}",
        "   🛡️  高風險套件安全掃描器  🛡️".bright_white().bold()
    );
    println!("{}", "═══════════════════════════════════════════".cyan());
    println!();

    // 取得要搜尋的套件名稱
    let package_name = read_line("請輸入要搜尋的套件名稱: ");

    if package_name.is_empty() {
        println!("{} 套件名稱不能為空", "錯誤:".red().bold());
        return;
    }

    // 取得掃描目錄（預設為當前目錄）
    let scan_dir = read_line("請輸入要掃描的目錄（按 Enter 使用當前目錄）: ");
    let scan_dir = if scan_dir.is_empty() { ".".to_string() } else { scan_dir };

    let scan_path = if scan_dir.trim() == "." {
        env::current_dir().unwrap()
    } else {
        PathBuf::from(scan_dir.trim())
    };

    if !scan_path.exists() || !scan_path.is_dir() {
        println!(
            "\n{} 無效的目錄: {}",
            "錯誤:".red().bold(),
            scan_path.display()
        );
        return;
    }

    println!("\n{}", "🚀 開始高速掃描...".green().bold());
    println!(
        "   掃描目錄: {}",
        scan_path.display().to_string().bright_blue()
    );
    println!(
        "   搜尋目標: {}",
        package_name.yellow().bold()
    );
    println!();

    // 開始計時
    let start_time = Instant::now();

    // 執行掃描
    let scanner = ContentScanner::new(package_name.clone());
    let (matches, stats) = scanner.scan(&scan_path);

    let elapsed = start_time.elapsed();

    // 輸出結果
    println!();
    println!("{}", "═══════════════════════════════════════════".cyan());
    println!("{}", "   📊 掃描結果報告".bright_white().bold());
    println!("{}", "═══════════════════════════════════════════".cyan());
    println!();

    println!(
        "   ⏱️  耗時: {:.2}秒",
        elapsed.as_secs_f64().to_string().yellow()
    );
    println!(
        "   📁 掃描檔案數: {}",
        stats.files_scanned.to_string().yellow()
    );

    // 計算效能指標
    if elapsed.as_secs_f64() > 0.0 {
        let files_per_sec = stats.files_scanned as f64 / elapsed.as_secs_f64();
        println!(
            "   ⚡ 效能: {:.0} 檔案/秒",
            files_per_sec.to_string().green()
        );
    }

    println!();

    if matches.is_empty() {
        println!(
            "{}",
            "   ✅ 太好了！未發現該套件！".green().bold()
        );
    } else {
        println!(
            "   {} 發現 {} 處匹配！",
            "⚠️  警告:".red().bold(),
            matches.len().to_string().red().bold()
        );
        println!();
        println!("{}", "   詳細資訊:".yellow());
        println!("{}", "   ─────────────────────────────────────────".dimmed());

        for m in &matches {
            println!(
                "   {} {}:{}",
                "⛔".red(),
                m.file_path.display().to_string().bright_blue(),
                m.line_number.to_string().yellow()
            );

            // 截斷過長的行內容
            let display_content = if m.line_content.len() > 80 {
                format!("{}...", &m.line_content[..77])
            } else {
                m.line_content.clone()
            };
            println!("      {}", display_content.dimmed());
        }

        println!();
        println!(
            "{}",
            "   💡 建議: 請檢查這些檔案並移除可疑的套件".yellow()
        );
    }

    println!();
    println!("{}", "═══════════════════════════════════════════".cyan());
}
