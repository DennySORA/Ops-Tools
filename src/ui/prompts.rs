use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};

/// 使用者輸入提示工具
pub struct Prompts {
    theme: ColorfulTheme,
}

impl Prompts {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    /// 簡單確認（預設否）
    pub fn confirm(&self, prompt: &str) -> bool {
        Confirm::with_theme(&self.theme)
            .with_prompt(prompt)
            .default(false)
            .interact()
            .unwrap_or(false)
    }

    /// 確認對話框（使用選項式）
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

    /// 單選選單
    pub fn select(&self, prompt: &str, items: &[&str]) -> Option<usize> {
        Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(items)
            .interact()
            .ok()
    }

    /// 單選選單（帶預設值）
    #[allow(dead_code)]
    pub fn select_with_default(
        &self,
        prompt: &str,
        items: &[&str],
        default: usize,
    ) -> Option<usize> {
        Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .ok()
    }

    /// 多選選單
    pub fn multi_select(&self, prompt: &str, items: &[String], defaults: &[bool]) -> Vec<usize> {
        MultiSelect::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(items)
            .defaults(defaults)
            .interact()
            .unwrap_or_default()
    }
}

impl Default for Prompts {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_creation() {
        let _prompts = Prompts::new();
    }
}
