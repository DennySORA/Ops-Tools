mod executor;
mod tools;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use executor::ExtensionExecutor;
use tools::{get_available_extensions, CliType, Extension};

/// Run the skill installer feature
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::SKILL_INSTALLER_HEADER));

    // Select CLI type
    let cli_options = ["Anthropic Claude", "OpenAI Codex", "Google Gemini"];
    let cli_selection = prompts.select(i18n::t(keys::SKILL_INSTALLER_SELECT_CLI), &cli_options);

    let cli = match cli_selection {
        Some(0) => CliType::Claude,
        Some(1) => CliType::Codex,
        Some(2) => CliType::Gemini,
        _ => {
            console.warning(i18n::t(keys::SKILL_INSTALLER_CANCELLED));
            return;
        }
    };

    console.blank_line();
    console.info(&crate::tr!(
        keys::SKILL_INSTALLER_USING_CLI,
        cli = cli.display_name()
    ));

    let executor = ExtensionExecutor::new(cli);

    // Scan installed extensions
    console.info(i18n::t(keys::SKILL_INSTALLER_SCANNING));
    let installed = executor.list_installed().unwrap_or_default();

    if installed.is_empty() {
        console.warning(i18n::t(keys::SKILL_INSTALLER_NONE_INSTALLED));
    } else {
        console.success(&crate::tr!(
            keys::SKILL_INSTALLER_FOUND_INSTALLED,
            count = installed.len()
        ));
        for (name, ext_type) in &installed {
            console.list_item("✓", &format!("{} ({})", name, ext_type.display_name()));
        }
    }

    console.blank_line();
    console.separator();

    // Get available extensions for this CLI
    let available_extensions = get_available_extensions(cli);

    if available_extensions.is_empty() {
        console.warning(i18n::t(keys::SKILL_INSTALLER_NO_EXTENSIONS));
        return;
    }

    // Build display items with status
    let items: Vec<String> = available_extensions
        .iter()
        .map(|ext| {
            let status = if installed.contains_key(ext.name) {
                i18n::t(keys::SKILL_INSTALLER_STATUS_INSTALLED)
            } else {
                i18n::t(keys::SKILL_INSTALLER_STATUS_MISSING)
            };
            format!(
                "{} {} ({})",
                status,
                ext.display_name(),
                ext.extension_type.display_name()
            )
        })
        .collect();

    // Set defaults based on installed state
    let defaults: Vec<bool> = available_extensions
        .iter()
        .map(|ext| installed.contains_key(ext.name))
        .collect();

    console.blank_line();
    console.info(i18n::t(keys::SKILL_INSTALLER_SELECT_PROMPT));
    console.info(i18n::t(keys::SKILL_INSTALLER_SELECT_HELP));
    console.blank_line();

    let selections = prompts.multi_select(
        i18n::t(keys::SKILL_INSTALLER_SELECT_PROMPT),
        &items,
        &defaults,
    );

    // Calculate changes
    let mut to_install: Vec<&Extension> = Vec::new();
    let mut to_remove: Vec<&Extension> = Vec::new();

    for (i, ext) in available_extensions.iter().enumerate() {
        let is_selected = selections.contains(&i);
        let is_installed = installed.contains_key(ext.name);

        if is_selected && !is_installed {
            to_install.push(ext);
        } else if !is_selected && is_installed {
            to_remove.push(ext);
        }
    }

    if to_install.is_empty() && to_remove.is_empty() {
        console.blank_line();
        console.success(i18n::t(keys::SKILL_INSTALLER_NO_CHANGES));
        return;
    }

    // Display change summary
    console.blank_line();
    console.separator();
    console.info(i18n::t(keys::SKILL_INSTALLER_CHANGE_SUMMARY));

    if !to_install.is_empty() {
        console.success(i18n::t(keys::SKILL_INSTALLER_WILL_INSTALL));
        for ext in &to_install {
            console.list_item("➕", ext.display_name());
        }
    }

    if !to_remove.is_empty() {
        console.warning(i18n::t(keys::SKILL_INSTALLER_WILL_REMOVE));
        for ext in &to_remove {
            console.list_item("➖", ext.display_name());
        }
    }

    console.blank_line();
    if !prompts.confirm(i18n::t(keys::SKILL_INSTALLER_CONFIRM_CHANGES)) {
        console.warning(i18n::t(keys::SKILL_INSTALLER_CANCELLED));
        return;
    }

    console.blank_line();

    // Execute installation and removal
    let mut success_count = 0;
    let mut failed_count = 0;
    let total_operations = to_install.len() + to_remove.len();

    for (i, ext) in to_install.iter().enumerate() {
        console.show_progress(
            i + 1,
            total_operations,
            &crate::tr!(keys::SKILL_INSTALLER_DOWNLOADING, name = ext.display_name()),
        );

        match executor.install(ext) {
            Ok(()) => {
                console.success_item(&crate::tr!(
                    keys::SKILL_INSTALLER_INSTALL_SUCCESS,
                    name = ext.display_name()
                ));
                success_count += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(
                        keys::SKILL_INSTALLER_INSTALL_FAILED,
                        name = ext.display_name()
                    ),
                    &err.to_string(),
                );
                failed_count += 1;
            }
        }
    }

    for (i, ext) in to_remove.iter().enumerate() {
        console.show_progress(
            to_install.len() + i + 1,
            total_operations,
            &crate::tr!(keys::SKILL_INSTALLER_REMOVING, name = ext.display_name()),
        );

        match executor.remove(ext) {
            Ok(()) => {
                console.success_item(&crate::tr!(
                    keys::SKILL_INSTALLER_REMOVE_SUCCESS,
                    name = ext.display_name()
                ));
                success_count += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(
                        keys::SKILL_INSTALLER_REMOVE_FAILED,
                        name = ext.display_name()
                    ),
                    &err.to_string(),
                );
                failed_count += 1;
            }
        }
    }

    console.show_summary(
        i18n::t(keys::SKILL_INSTALLER_SUMMARY),
        success_count,
        failed_count,
    );
}

#[cfg(test)]
mod tests {
    use super::tools::{get_available_extensions, CliType};

    #[test]
    fn test_extensions_available() {
        let extensions = get_available_extensions(CliType::Claude);
        assert!(!extensions.is_empty());
    }
}
