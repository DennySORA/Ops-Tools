mod component;
mod tools;

use colored::*;
use component::{clear_terrform, mcp_manager, npm_upgrade, package_scanner, terragrunt_apply};
use dialoguer::{theme::ColorfulTheme, Select};

fn main() {
    loop {
        let options = vec![
            "清理 Terraform/Terragrunt 快取檔案",
            "批次執行 Terragrunt Apply",
            "升級 AI 程式碼助手工具",
            "掃描高風險套件（安全檢測）",
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
            0 => clear_terrform::clean_terraform_cache(),
            1 => terragrunt_apply::batch_apply(),
            2 => npm_upgrade::upgrade_ai_tools(),
            3 => package_scanner::scan_risky_packages(),
            4 => mcp_manager::manage_mcp(),
            5 => {
                println!("{}", "再見！".green());
                break;
            }
            _ => unreachable!(),
        }

        println!("\n");
    }
}
