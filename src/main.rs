mod core;
mod features;
mod ui;

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};

fn main() {
    loop {
        let options = vec![
            "清理 Terraform/Terragrunt 快取檔案",
            "升級 AI 程式碼助手工具",
            "管理 MCP 工具（Claude/Codex）",
            "退出",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("請選擇功能")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => features::terraform_cleaner::run(),
            1 => features::tool_upgrader::run(),
            2 => features::mcp_manager::run(),
            3 => {
                println!("{}", "再見！".green());
                break;
            }
            _ => unreachable!(),
        }

        println!("\n");
    }
}
