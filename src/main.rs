mod core;
mod features;
mod i18n;
mod ui;

use crate::core::{load_config, save_config, AppConfig};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use i18n::{keys, Language};
use ui::{Console, Prompts};
use unicode_width::UnicodeWidthStr;

/// Menu item definition
struct MenuItem {
    name_key: &'static str,
    desc_key: &'static str,
    handler: fn(),
}

/// Get sortable menu items (excludes language and exit)
fn sortable_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem {
            name_key: keys::MENU_TERRAFORM_CLEANER,
            desc_key: keys::MENU_TERRAFORM_CLEANER_DESC,
            handler: features::terraform_cleaner::run,
        },
        MenuItem {
            name_key: keys::MENU_TOOL_UPGRADER,
            desc_key: keys::MENU_TOOL_UPGRADER_DESC,
            handler: features::tool_upgrader::run,
        },
        MenuItem {
            name_key: keys::MENU_PACKAGE_MANAGER,
            desc_key: keys::MENU_PACKAGE_MANAGER_DESC,
            handler: features::package_manager::run,
        },
        MenuItem {
            name_key: keys::MENU_RUST_UPGRADER,
            desc_key: keys::MENU_RUST_UPGRADER_DESC,
            handler: features::rust_upgrader::run,
        },
        MenuItem {
            name_key: keys::MENU_SECURITY_SCANNER,
            desc_key: keys::MENU_SECURITY_SCANNER_DESC,
            handler: features::security_scanner::run,
        },
        MenuItem {
            name_key: keys::MENU_MCP_MANAGER,
            desc_key: keys::MENU_MCP_MANAGER_DESC,
            handler: features::mcp_manager::run,
        },
        MenuItem {
            name_key: keys::MENU_PROMPT_GEN,
            desc_key: keys::MENU_PROMPT_GEN_DESC,
            handler: features::prompt_gen::run,
        },
        MenuItem {
            name_key: keys::MENU_KUBECONFIG_MANAGER,
            desc_key: keys::MENU_KUBECONFIG_MANAGER_DESC,
            handler: features::kubeconfig_manager::run,
        },
    ]
}

/// Sort menu items by usage frequency (descending)
fn sort_by_usage(items: &mut [MenuItem], config: &AppConfig) {
    items.sort_by(|a, b| {
        let usage_a = config.get_usage(a.name_key);
        let usage_b = config.get_usage(b.name_key);
        usage_b.cmp(&usage_a)
    });
}

/// Format menu options with aligned names and descriptions
fn format_menu_options(items: &[MenuItem]) -> Vec<String> {
    let language_name = i18n::t(keys::MENU_LANGUAGE);
    let language_desc = i18n::t(keys::MENU_LANGUAGE_DESC);

    let max_name_width = items
        .iter()
        .map(|item| i18n::t(item.name_key).width())
        .chain(std::iter::once(language_name.width()))
        .max()
        .unwrap_or(0);

    let mut options: Vec<String> = items
        .iter()
        .map(|item| {
            let name = i18n::t(item.name_key);
            let desc = i18n::t(item.desc_key);
            let padding = max_name_width - name.width();
            format!("{}{} — {}", name, " ".repeat(padding), desc)
        })
        .collect();

    let padding = max_name_width - language_name.width();
    options.push(format!(
        "{}{} — {}",
        language_name,
        " ".repeat(padding),
        language_desc
    ));

    options.push(i18n::t(keys::MENU_EXIT).to_string());

    options
}

fn main() {
    let prompts = Prompts::new();
    let console = Console::new();

    if !apply_saved_language(&console) {
        select_language_on_start(&prompts, &console);
    }

    loop {
        let config = load_config().ok().flatten().unwrap_or_default();
        let mut menu_items = sortable_menu_items();
        sort_by_usage(&mut menu_items, &config);

        let options = format_menu_options(&menu_items);
        let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::MENU_PROMPT))
            .items(&option_refs)
            .default(0)
            .interact()
            .unwrap();

        let sortable_count = menu_items.len();

        if selection < sortable_count {
            let selected_item = &menu_items[selection];
            record_usage(selected_item.name_key, &console);
            (selected_item.handler)();
        } else if selection == sortable_count {
            select_language(&prompts, &console);
        } else {
            println!("{}", i18n::t(keys::MENU_GOODBYE).green());
            break;
        }

        println!("\n");
    }
}

/// Record menu usage to config
fn record_usage(key: &str, console: &Console) {
    let mut config = load_config().ok().flatten().unwrap_or_default();
    config.increment_usage(key);
    if let Err(err) = save_config(&config) {
        console.warning(&crate::tr!(keys::CONFIG_SAVE_FAILED, error = err));
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
