#[allow(dead_code)]
pub mod application;
#[allow(dead_code)]
pub mod domain;
#[allow(dead_code)]
pub mod infrastructure;
#[allow(dead_code)]
pub mod ports;

#[cfg(test)]
#[allow(dead_code)]
pub mod testing;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use application::cli::{CliCommand, CliOptions};
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::PathBuf;

pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::SYSTEM_UPDATER_HEADER));

    let mode_options = [
        i18n::t(keys::SYSTEM_UPDATER_MODE_RUN),
        i18n::t(keys::SYSTEM_UPDATER_MODE_SCAN),
        i18n::t(keys::SYSTEM_UPDATER_MODE_CLEANUP),
        i18n::t(keys::SYSTEM_UPDATER_MODE_VERIFY),
        i18n::t(keys::SYSTEM_UPDATER_MODE_BACKUP),
    ];
    let option_refs: Vec<&str> = mode_options.iter().map(|s| s.as_ref()).collect();

    let mode_index = match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n::t(keys::SYSTEM_UPDATER_SELECT_MODE))
        .items(&option_refs)
        .default(0)
        .interact_opt()
        .unwrap()
    {
        Some(index) => index,
        None => {
            console.info(i18n::t(keys::SYSTEM_UPDATER_CANCELLED));
            return;
        }
    };

    let command = match mode_index {
        0 => CliCommand::Run,
        1 => CliCommand::Scan,
        2 => CliCommand::Cleanup,
        3 => CliCommand::Verify,
        4 => CliCommand::Backup,
        _ => unreachable!(),
    };

    let dry_run = if !matches!(command, CliCommand::Scan) {
        let dry_run_options = [
            i18n::t(keys::SYSTEM_UPDATER_EXECUTE),
            i18n::t(keys::SYSTEM_UPDATER_DRY_RUN),
        ];
        let dry_refs: Vec<&str> = dry_run_options.iter().map(|s| s.as_ref()).collect();
        match prompts.select_with_default(
            i18n::t(keys::SYSTEM_UPDATER_DRY_RUN_PROMPT),
            &dry_refs,
            0,
        ) {
            Some(1) => true,
            Some(0) => false,
            _ => {
                console.info(i18n::t(keys::SYSTEM_UPDATER_CANCELLED));
                return;
            }
        }
    } else {
        false
    };

    let profile = select_profile(&prompts, &console);

    let config_path = resolve_config_path();

    let options = CliOptions {
        command,
        dry_run,
        config_path,
        profile,
    };

    application::cli::execute(options);
}

fn select_profile(prompts: &Prompts, console: &Console) -> Option<String> {
    let profile_options = [
        i18n::t(keys::SYSTEM_UPDATER_PROFILE_DEFAULT),
        i18n::t(keys::SYSTEM_UPDATER_PROFILE_SAFE),
        i18n::t(keys::SYSTEM_UPDATER_PROFILE_AGGRESSIVE),
    ];
    let refs: Vec<&str> = profile_options.iter().map(|s| s.as_ref()).collect();

    match prompts.select_with_default(i18n::t(keys::SYSTEM_UPDATER_SELECT_PROFILE), &refs, 0) {
        Some(0) => None,
        Some(1) => Some("safe".to_string()),
        Some(2) => Some("aggressive".to_string()),
        _ => {
            console.info(i18n::t(keys::SYSTEM_UPDATER_CANCELLED));
            None
        }
    }
}

fn resolve_config_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("update.toml"),
        dirs::config_dir()
            .map(|dir| dir.join("update/config.toml"))
            .unwrap_or_default(),
    ];
    candidates.into_iter().find(|path| path.exists())
}
