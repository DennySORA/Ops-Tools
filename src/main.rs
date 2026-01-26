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
#[derive(Clone, Copy)]
struct MenuItem {
    name_key: &'static str,
    desc_key: &'static str,
    handler: fn(),
}

#[derive(Clone)]
struct Category {
    name_key: &'static str,
    desc_key: &'static str,
    items: Vec<MenuItem>,
}

enum TopLevelChoice {
    Action(MenuItem),
    Category(Category),
    Settings,
    Header,
    Exit,
}

struct TopLevelOption {
    label: String,
    choice: TopLevelChoice,
    selectable: bool,
}

/// Get all executable menu items (excludes language and exit)
fn all_actions() -> Vec<MenuItem> {
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
        MenuItem {
            name_key: keys::MENU_RUST_BUILDER,
            desc_key: keys::MENU_RUST_BUILDER_DESC,
            handler: features::rust_builder::run,
        },
        MenuItem {
            name_key: keys::MENU_CONTAINER_BUILDER,
            desc_key: keys::MENU_CONTAINER_BUILDER_DESC,
            handler: features::container_builder::run,
        },
        MenuItem {
            name_key: keys::MENU_SKILL_INSTALLER,
            desc_key: keys::MENU_SKILL_INSTALLER_DESC,
            handler: features::skill_installer::run,
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

fn find_action(items: &[MenuItem], key: &str) -> MenuItem {
    items
        .iter()
        .find(|item| item.name_key == key)
        .copied()
        .expect("Menu item missing from catalog")
}

fn build_categories(items: &[MenuItem]) -> Vec<Category> {
    vec![
        Category {
            name_key: keys::MENU_CATEGORY_BUILD,
            desc_key: keys::MENU_CATEGORY_BUILD_DESC,
            items: vec![
                find_action(items, keys::MENU_RUST_BUILDER),
                find_action(items, keys::MENU_CONTAINER_BUILDER),
            ],
        },
        Category {
            name_key: keys::MENU_CATEGORY_AI,
            desc_key: keys::MENU_CATEGORY_AI_DESC,
            items: vec![
                find_action(items, keys::MENU_MCP_MANAGER),
                find_action(items, keys::MENU_SKILL_INSTALLER),
                find_action(items, keys::MENU_PROMPT_GEN),
            ],
        },
        Category {
            name_key: keys::MENU_CATEGORY_UPGRADE,
            desc_key: keys::MENU_CATEGORY_UPGRADE_DESC,
            items: vec![
                find_action(items, keys::MENU_TOOL_UPGRADER),
                find_action(items, keys::MENU_RUST_UPGRADER),
                find_action(items, keys::MENU_PACKAGE_MANAGER),
            ],
        },
        Category {
            name_key: keys::MENU_CATEGORY_INFRA,
            desc_key: keys::MENU_CATEGORY_INFRA_DESC,
            items: vec![
                find_action(items, keys::MENU_TERRAFORM_CLEANER),
                find_action(items, keys::MENU_KUBECONFIG_MANAGER),
            ],
        },
        Category {
            name_key: keys::MENU_CATEGORY_SECURITY,
            desc_key: keys::MENU_CATEGORY_SECURITY_DESC,
            items: vec![find_action(items, keys::MENU_SECURITY_SCANNER)],
        },
    ]
}

fn build_common_actions(mut items: Vec<MenuItem>, config: &AppConfig) -> Vec<MenuItem> {
    sort_by_usage(&mut items, config);
    let limit = config.common_actions_limit().min(items.len().max(1));
    items.truncate(limit);
    items
}

fn format_action_options(items: &[MenuItem]) -> Vec<String> {
    let max_name_width = items
        .iter()
        .map(|item| i18n::t(item.name_key).width())
        .max()
        .unwrap_or(0);

    items
        .iter()
        .map(|item| {
            let name = format!("  {}", i18n::t(item.name_key));
            let desc = i18n::t(item.desc_key);
            let padding = max_name_width - name.trim_start().width();
            format!("{}{} — {}", name, " ".repeat(padding), desc)
        })
        .collect()
}

fn build_pinned_actions(all_items: &[MenuItem], config: &AppConfig) -> Vec<MenuItem> {
    config
        .pinned_items()
        .iter()
        .filter_map(|key| all_items.iter().find(|item| item.name_key == key).copied())
        .collect()
}

fn format_top_level_options(
    pinned_actions: &[MenuItem],
    common_actions: &[MenuItem],
    categories: &[Category],
) -> Vec<TopLevelOption> {
    let settings_name = i18n::t(keys::MENU_SETTINGS);
    let settings_desc = i18n::t(keys::MENU_SETTINGS_DESC);
    let pin_icon = i18n::t(keys::MENU_PIN_ICON);

    let max_name_width = pinned_actions
        .iter()
        .chain(common_actions.iter())
        .map(|item| i18n::t(item.name_key).width())
        .chain(categories.iter().map(|cat| i18n::t(cat.name_key).width()))
        .max()
        .unwrap_or(0);

    let mut options = Vec::new();

    // Pinned header (only show if there are pinned items)
    if !pinned_actions.is_empty() {
        options.push(TopLevelOption {
            label: format!("{} {}", pin_icon, i18n::t(keys::MENU_PINNED)),
            choice: TopLevelChoice::Header,
            selectable: false,
        });

        for item in pinned_actions {
            let name = format!("  {}", i18n::t(item.name_key));
            let desc = i18n::t(item.desc_key);
            let padding = max_name_width.saturating_sub(name.trim_start().width());
            options.push(TopLevelOption {
                label: format!("{}{} — {}", name, " ".repeat(padding), desc),
                choice: TopLevelChoice::Action(*item),
                selectable: true,
            });
        }
    }

    // Common header
    options.push(TopLevelOption {
        label: i18n::t(keys::MENU_COMMON).to_string(),
        choice: TopLevelChoice::Header,
        selectable: false,
    });

    for item in common_actions {
        let name = format!("  {}", i18n::t(item.name_key));
        let desc = i18n::t(item.desc_key);
        let padding = max_name_width.saturating_sub(name.trim_start().width());
        options.push(TopLevelOption {
            label: format!("{}{} — {}", name, " ".repeat(padding), desc),
            choice: TopLevelChoice::Action(*item),
            selectable: true,
        });
    }

    // Categories header
    options.push(TopLevelOption {
        label: i18n::t(keys::MENU_CATEGORIES).to_string(),
        choice: TopLevelChoice::Header,
        selectable: false,
    });

    for category in categories {
        let name = i18n::t(category.name_key);
        let desc = i18n::t(category.desc_key);
        let padding = max_name_width.saturating_sub(name.width());
        options.push(TopLevelOption {
            label: format!("  {}{} — {}", name, " ".repeat(padding), desc),
            choice: TopLevelChoice::Category(category.clone()),
            selectable: true,
        });
    }

    let padding = max_name_width.saturating_sub(settings_name.width());
    options.push(TopLevelOption {
        label: format!(
            "  {}{} — {}",
            settings_name,
            " ".repeat(padding),
            settings_desc
        ),
        choice: TopLevelChoice::Settings,
        selectable: true,
    });

    options.push(TopLevelOption {
        label: i18n::t(keys::MENU_EXIT).to_string(),
        choice: TopLevelChoice::Exit,
        selectable: true,
    });

    options
}

fn select_category_item(category: &Category, config: &AppConfig) -> Option<MenuItem> {
    let mut items = category.items.clone();
    sort_by_usage(&mut items, config);
    let mut options = format_action_options(&items);
    options.push(i18n::t(keys::MENU_BACK).to_string());

    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
    let prompt = crate::tr!(
        keys::MENU_CATEGORY_PROMPT,
        category = i18n::t(category.name_key)
    );

    let selection_opt = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&option_refs)
        .default(0)
        .interact_opt()
        .unwrap();

    match selection_opt {
        Some(selection) if selection < items.len() => Some(items[selection]),
        _ => None,
    }
}

fn open_settings(prompts: &Prompts, console: &Console) {
    let mut config = load_config().ok().flatten().unwrap_or_default();

    loop {
        let settings_items = [
            (keys::MENU_LANGUAGE, keys::MENU_LANGUAGE_DESC),
            (
                keys::SETTINGS_COMMON_COUNT_NAME,
                keys::SETTINGS_COMMON_COUNT_DESC,
            ),
            (keys::MENU_PIN_MANAGE, keys::MENU_PIN_MANAGE_DESC),
            (keys::MENU_PIN_REORDER, keys::MENU_PIN_REORDER_DESC),
        ];

        let max_name_width = settings_items
            .iter()
            .map(|(name_key, _)| i18n::t(name_key).width())
            .max()
            .unwrap_or(0);

        let mut options: Vec<String> = settings_items
            .iter()
            .map(|(name_key, desc_key)| {
                let name = i18n::t(name_key);
                let desc = i18n::t(desc_key);
                let padding = max_name_width - name.width();
                format!("{}{} — {}", name, " ".repeat(padding), desc)
            })
            .collect();

        options.push(i18n::t(keys::MENU_BACK).to_string());
        let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        let selection_opt = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::SETTINGS_MENU_PROMPT))
            .items(&option_refs)
            .default(0)
            .interact_opt()
            .unwrap();

        match selection_opt {
            Some(0) => select_language(prompts, console),
            Some(1) => configure_common_actions(prompts, console, &mut config),
            Some(2) => manage_pins(console, &mut config),
            Some(3) => reorder_pins(console, &mut config),
            _ => break,
        }
    }
}

fn configure_common_actions(prompts: &Prompts, console: &Console, config: &mut AppConfig) {
    let options: Vec<String> = (1..=6).map(|n| n.to_string()).collect();
    let default = config
        .common_actions_limit()
        .saturating_sub(1)
        .min(options.len() - 1);
    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

    if let Some(index) = prompts.select_with_default(
        i18n::t(keys::SETTINGS_COMMON_COUNT_PROMPT),
        &option_refs,
        default,
    ) {
        let value = index + 1;
        config.common_actions_limit = value as u32;
        match save_config(config) {
            Ok(_) => console.success(&crate::tr!(
                keys::SETTINGS_COMMON_COUNT_SAVED,
                count = value
            )),
            Err(err) => console.warning(&crate::tr!(keys::CONFIG_SAVE_FAILED, error = err)),
        }
    }
}

fn manage_pins(console: &Console, config: &mut AppConfig) {
    use dialoguer::MultiSelect;

    let actions = all_actions();
    let pin_icon = i18n::t(keys::MENU_PIN_ICON);

    // Build options with pin status
    let options: Vec<String> = actions
        .iter()
        .map(|item| {
            let name = i18n::t(item.name_key);
            let desc = i18n::t(item.desc_key);
            if config.is_pinned(item.name_key) {
                format!("{} {} — {}", pin_icon, name, desc)
            } else {
                format!("  {} — {}", name, desc)
            }
        })
        .collect();

    // Get currently pinned indices
    let defaults: Vec<bool> = actions
        .iter()
        .map(|item| config.is_pinned(item.name_key))
        .collect();

    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::MENU_PIN_PROMPT))
        .items(&option_refs)
        .defaults(&defaults)
        .interact_opt();

    if let Ok(Some(selected)) = selection {
        // Clear all pins and re-add selected ones
        config.pinned_items.clear();
        for idx in selected {
            config.pin_item(actions[idx].name_key);
        }

        match save_config(config) {
            Ok(_) => {
                let count = config.pinned_items().len();
                if count > 0 {
                    console.success(&format!(
                        "{} {}",
                        pin_icon,
                        crate::tr!(keys::MENU_PIN_COUNT, count = count)
                    ));
                } else {
                    console.info(i18n::t(keys::MENU_PIN_CLEARED));
                }
            }
            Err(err) => console.warning(&crate::tr!(keys::CONFIG_SAVE_FAILED, error = err)),
        }
    }
}

fn reorder_pins(console: &Console, config: &mut AppConfig) {
    use dialoguer::Select;

    let actions = all_actions();
    let pinned_keys: Vec<String> = config.pinned_items().to_vec();

    if pinned_keys.is_empty() {
        console.info(i18n::t(keys::MENU_PIN_REORDER_EMPTY));
        return;
    }

    // Get display names for pinned items
    let pinned_items: Vec<(&str, String)> = pinned_keys
        .iter()
        .filter_map(|key| {
            actions
                .iter()
                .find(|a| a.name_key == key)
                .map(|a| (a.name_key, i18n::t(a.name_key).to_string()))
        })
        .collect();

    let mut new_order: Vec<&str> = Vec::new();
    let mut remaining: Vec<(&str, String)> = pinned_items;

    while !remaining.is_empty() {
        let options: Vec<String> = remaining
            .iter()
            .enumerate()
            .map(|(i, (_, name))| format!("{}. {}", i + 1, name))
            .collect();

        let prompt = format!(
            "{} ({}/{})",
            i18n::t(keys::MENU_PIN_REORDER_PROMPT),
            new_order.len() + 1,
            new_order.len() + remaining.len()
        );

        let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(&prompt)
            .items(&option_refs)
            .default(0)
            .interact_opt();

        match selection {
            Ok(Some(idx)) => {
                let (key, _) = remaining.remove(idx);
                new_order.push(key);
            }
            _ => {
                // User cancelled - keep original order
                return;
            }
        }
    }

    // Update config with new order
    config.pinned_items.clear();
    for key in new_order {
        config.pinned_items.push(key.to_string());
    }

    match save_config(config) {
        Ok(_) => console.success(i18n::t(keys::MENU_PIN_REORDER_DONE)),
        Err(err) => console.warning(&crate::tr!(keys::CONFIG_SAVE_FAILED, error = err)),
    }
}

fn main() {
    let prompts = Prompts::new();
    let console = Console::new();

    if !apply_saved_language(&console) {
        select_language_on_start(&prompts, &console);
    }

    loop {
        let config = load_config().ok().flatten().unwrap_or_default();
        let actions = all_actions();
        let categories = build_categories(&actions);
        let pinned_actions = build_pinned_actions(&actions, &config);
        let common_actions = build_common_actions(actions.clone(), &config);
        let options = format_top_level_options(&pinned_actions, &common_actions, &categories);
        let option_refs: Vec<&str> = options.iter().map(|opt| opt.label.as_str()).collect();

        let default_index = options.iter().position(|opt| opt.selectable).unwrap_or(0);

        let selection_opt = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(i18n::t(keys::MENU_PROMPT))
            .items(&option_refs)
            .default(default_index)
            .interact_opt()
            .unwrap();

        let Some(selection) = selection_opt else {
            println!("{}", i18n::t(keys::MENU_GOODBYE).green());
            break;
        };

        if !options[selection].selectable {
            continue;
        }

        match &options[selection].choice {
            TopLevelChoice::Action(item) => {
                record_usage(item.name_key, &console);
                (item.handler)();
            }
            TopLevelChoice::Category(category) => {
                if let Some(item) = select_category_item(category, &config) {
                    record_usage(item.name_key, &console);
                    (item.handler)();
                }
            }
            TopLevelChoice::Settings => {
                open_settings(&prompts, &console);
            }
            TopLevelChoice::Header => {}
            TopLevelChoice::Exit => {
                println!("{}", i18n::t(keys::MENU_GOODBYE).green());
                break;
            }
        }

        println!();
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
