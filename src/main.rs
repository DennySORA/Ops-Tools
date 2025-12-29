mod core;
mod features;
mod i18n;
mod ui;

use crate::core::{load_config, save_config, AppConfig};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use i18n::{keys, Language};
use ui::{Console, Prompts};

fn main() {
    let prompts = Prompts::new();
    let console = Console::new();

    if !apply_saved_language(&console) {
        select_language_on_start(&prompts, &console);
    }

    loop {
        let options = vec![
            i18n::t(keys::MENU_TERRAFORM_CLEANER),
            i18n::t(keys::MENU_TOOL_UPGRADER),
            i18n::t(keys::MENU_RUST_UPGRADER),
            i18n::t(keys::MENU_GIT_SCANNER),
            i18n::t(keys::MENU_MCP_MANAGER),
            i18n::t(keys::MENU_PROMPT_GEN),
            i18n::t(keys::MENU_LANGUAGE),
            i18n::t(keys::MENU_EXIT),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::MENU_PROMPT))
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => features::terraform_cleaner::run(),
            1 => features::tool_upgrader::run(),
            2 => features::rust_upgrader::run(),
            3 => features::git_scanner::run(),
            4 => features::mcp_manager::run(),
            5 => features::prompt_gen::run(),
            6 => select_language(&prompts, &console),
            7 => {
                println!("{}", i18n::t(keys::MENU_GOODBYE).green());
                break;
            }
            _ => unreachable!(),
        }

        println!("\n");
    }
}

fn select_language_on_start(prompts: &Prompts, console: &Console) {
    let options: Vec<&str> = Language::ALL
        .iter()
        .map(|lang| lang.display_name())
        .collect();
    let prompt = "Select language / 選擇語言 / 选择语言 / 言語を選択";
    if let Some(index) =
        prompts.select_with_default(prompt, &options, i18n::current_language().index())
    {
        if let Some(language) = Language::from_index(index) {
            i18n::set_language(language);
            persist_language(console);
        }
    }
}

fn select_language(prompts: &Prompts, console: &Console) {
    let options: Vec<&str> = Language::ALL
        .iter()
        .map(|lang| lang.display_name())
        .collect();
    let default = i18n::current_language().index();
    if let Some(index) =
        prompts.select_with_default(i18n::t(keys::LANGUAGE_SELECT_PROMPT), &options, default)
    {
        if let Some(language) = Language::from_index(index) {
            i18n::set_language(language);
            console.success(&crate::tr!(
                keys::LANGUAGE_CHANGED,
                language = language.display_name()
            ));
            persist_language(console);
        }
    }
}

fn apply_saved_language(console: &Console) -> bool {
    match load_config() {
        Ok(Some(config)) => {
            if let Some(code) = config.language.as_deref() {
                if let Some(language) = Language::from_code(code) {
                    i18n::set_language(language);
                    return true;
                }
                console.warning(&crate::tr!(keys::CONFIG_LANGUAGE_INVALID, code = code));
            }
            false
        }
        Ok(None) => false,
        Err(err) => {
            console.warning(&crate::tr!(keys::CONFIG_LOAD_FAILED, error = err));
            false
        }
    }
}

fn persist_language(console: &Console) {
    let mut config = match load_config() {
        Ok(Some(config)) => config,
        Ok(None) => AppConfig::default(),
        Err(err) => {
            console.warning(&crate::tr!(keys::CONFIG_LOAD_FAILED, error = err));
            AppConfig::default()
        }
    };

    config.language = Some(i18n::current_language().code().to_string());
    if let Err(err) = save_config(&config) {
        console.warning(&crate::tr!(keys::CONFIG_SAVE_FAILED, error = err));
    }
}
