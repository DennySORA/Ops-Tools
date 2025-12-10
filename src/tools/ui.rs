use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use std::path::PathBuf;

pub struct UserInterface {
    theme: ColorfulTheme,
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

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
        eprintln!("{} {}", "錯誤：".red(), message);
    }

    pub fn header(&self, title: &str) {
        println!("\n{}", "=".repeat(50).cyan());
        println!("{}", title.bold().cyan());
        println!("{}", "=".repeat(50).cyan());
    }

    pub fn separator(&self) {
        println!("{}", "-".repeat(50).bright_black());
    }

    pub fn show_items(&self, items: &[PathBuf], item_type_fn: impl Fn(&PathBuf) -> &str) {
        for item in items {
            let item_type = item_type_fn(item);
            println!("  {} {}", item_type.blue(), item.display());
        }
    }

    pub fn show_items_with_title(
        &self,
        title: &str,
        items: &[PathBuf],
        item_type_fn: impl Fn(&PathBuf) -> &str,
    ) {
        println!("\n{}", title);
        self.show_items(items, item_type_fn);
    }

    pub fn confirm(&self, prompt: &str) -> bool {
        Confirm::with_theme(&self.theme)
            .with_prompt(prompt)
            .default(false)
            .interact()
            .unwrap_or(false)
    }

    pub fn confirm_with_options(&self, prompt: &str, default_yes: bool) -> bool {
        let options = vec!["是", "否"];
        let default = if default_yes { 0 } else { 1 };

        let selection = Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(&options)
            .default(default)
            .interact()
            .unwrap_or(1);

        selection == 0
    }

    #[allow(dead_code)]
    pub fn select<'a>(&self, prompt: &str, items: &'a [&'a str]) -> Option<usize> {
        Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(items)
            .interact()
            .ok()
    }

    #[allow(dead_code)]
    pub fn select_with_default<'a>(
        &self,
        prompt: &str,
        items: &'a [&'a str],
        default: usize,
    ) -> Option<usize> {
        Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .ok()
    }

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

    pub fn list_item(&self, icon: &str, message: &str) {
        println!("  {} {}", icon, message);
    }

    pub fn success_item(&self, message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    pub fn error_item(&self, message: &str, error: &str) {
        eprintln!("{} {} - {}", "✗".red(), message, error.red());
    }
}

impl Default for UserInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_interface_creation() {
        let ui = UserInterface::new();
        ui.info("測試訊息");
        ui.success("成功訊息");
        ui.warning("警告訊息");
    }
}
