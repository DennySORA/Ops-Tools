mod config;
mod generator;
mod template;

use std::env;
use std::path::{Path, PathBuf};

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};

use generator::TerragruntGenerator;

/// 主入口：生成 Terragrunt 項目結構
pub fn generate_structure() {
    println!("{}", "Terragrunt 項目結構生成器".cyan().bold());
    println!("{}", "=".repeat(50));

    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    println!("目前目錄：{}", current_dir.display());

    let generator = TerragruntGenerator::new(current_dir.clone());

    // 顯示現有服務
    match generator.list_existing_services() {
        Ok(services) if !services.is_empty() => {
            println!("\n{}：", "現有服務".yellow());
            for service in &services {
                println!("  - {service}");
            }
        }
        Ok(_) => {
            println!("\n{}", "目前沒有現有服務".dimmed());
        }
        Err(err) => {
            println!("{}: {err}", "無法讀取現有服務".red());
        }
    }

    loop {
        println!();

        let options = vec!["創建新服務結構", "保存模版文件", "返回主選單"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("請選擇操作")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => create_service_structure(&generator),
            1 => save_templates(&generator, &current_dir),
            2 => {
                println!("{}", "返回主選單".green());
                break;
            }
            _ => unreachable!(),
        }
    }
}

fn create_service_structure(generator: &TerragruntGenerator) {
    let service_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("請輸入服務名稱")
        .validate_with(|input: &String| TerragruntGenerator::validate_service_name(input))
        .interact_text()
        .unwrap();

    // 預覽將要創建的結構
    println!("\n{}", "=".repeat(50));
    println!("{}", "預覽模式".cyan().bold());
    println!("{}", "=".repeat(50));

    let preview = generator.preview_structure(&service_name);

    // 顯示將要創建的文件
    if !preview.will_create.is_empty() {
        println!(
            "\n{} ({} 個)：",
            "將要創建的文件".green(),
            preview.will_create.len()
        );
        for file in &preview.will_create {
            println!("  {}", format!("+ {}", file.display()).green());
        }
    }

    // 顯示已存在的文件
    if !preview.already_exists.is_empty() {
        println!(
            "\n{} ({} 個)：",
            "已存在的文件（將跳過）".yellow(),
            preview.already_exists.len()
        );
        for file in &preview.already_exists {
            println!("  {}", format!("○ {}", file.display()).dimmed());
        }
    }

    // 如果沒有需要創建的文件
    if !preview.has_files_to_create() {
        println!("\n{}", "=".repeat(50));
        println!(
            "{}",
            format!("服務 '{service_name}' 的所有文件都已存在，無需創建新文件。").dimmed()
        );
        return;
    }

    // 確認是否創建
    println!("\n{}", "=".repeat(50));
    let confirm = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "確定要創建以上 {} 個文件嗎？",
            preview.files_to_create()
        ))
        .items(&["是，創建文件", "否，取消操作"])
        .default(0)
        .interact()
        .unwrap();

    if confirm != 0 {
        println!("{}", "已取消操作".yellow());
        return;
    }

    // 執行創建
    println!("\n正在為服務 '{}' 生成結構...", service_name.cyan());

    let result = generator.generate_structure(&service_name);

    // 顯示結果
    println!("\n{}", "=".repeat(50));
    println!("{}", "執行結果".cyan().bold());
    println!("{}", "=".repeat(50));

    if !result.created_files.is_empty() {
        println!("\n{}：", "已創建文件".green());
        for file in &result.created_files {
            println!("  {}", format!("✓ {}", file.display()).green());
        }
    }

    if !result.existing_files.is_empty() {
        println!("\n{}：", "已存在文件".yellow());
        for file in &result.existing_files {
            println!("  {}", format!("○ {}", file.display()).dimmed());
        }
    }

    if result.has_errors() {
        println!("\n{}：", "錯誤".red());
        for error in &result.errors {
            println!("  {}", format!("✗ {error}").red());
        }
    }

    println!("\n{}", "=".repeat(50));
    if result.has_errors() {
        println!(
            "{}",
            format!(
                "服務 '{service_name}' 的結構生成完成，但有 {} 個錯誤。",
                result.errors.len()
            )
            .yellow()
        );
    } else {
        println!(
            "{}",
            format!(
                "服務 '{service_name}' 的結構生成完成！共創建了 {} 個文件。",
                result.files_created()
            )
            .green()
        );
    }
}

fn save_templates(generator: &TerragruntGenerator, base_path: &Path) {
    let templates_dir = base_path.join("templates");

    match generator.save_templates(&templates_dir) {
        Ok(()) => {
            println!(
                "{}",
                format!("模版已保存到：{}", templates_dir.display()).green()
            );
        }
        Err(err) => {
            println!("{}: {err}", "保存模版失敗".red());
        }
    }
}
