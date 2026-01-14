mod config_content;
mod installers;
mod operations;
mod shell;
mod types;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use operations::{
    ensure_curl, package_definitions, update_curl, ActionContext, PackageAction, SupportedOs,
};
use std::collections::HashSet;

pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::PACKAGE_MANAGER_HEADER));

    let Some(os) = SupportedOs::detect() else {
        console.warning(i18n::t(keys::PACKAGE_MANAGER_UNSUPPORTED_OS));
        return;
    };

    let mut ctx = ActionContext::new(os);

    let options = vec![
        i18n::t(keys::PACKAGE_MANAGER_MODE_INSTALL),
        i18n::t(keys::PACKAGE_MANAGER_MODE_UPDATE),
    ];

    let Some(selection) = prompts.select(i18n::t(keys::PACKAGE_MANAGER_MODE_PROMPT), &options)
    else {
        console.warning(i18n::t(keys::PACKAGE_MANAGER_CANCELLED));
        return;
    };

    match selection {
        0 => run_install(&console, &prompts, &mut ctx),
        1 => run_update(&console, &prompts, &mut ctx),
        _ => unreachable!(),
    }
}

fn run_install(console: &Console, prompts: &Prompts, ctx: &mut ActionContext) {
    let packages = package_definitions();
    let defaults: Vec<bool> = packages
        .iter()
        .map(|pkg| operations::is_installed(pkg.id, ctx))
        .collect();

    let items: Vec<String> = packages.iter().map(|pkg| pkg.name.to_string()).collect();

    let selected = prompts.multi_select(
        i18n::t(keys::PACKAGE_MANAGER_INSTALL_PROMPT),
        &items,
        &defaults,
    );

    if selected.is_empty() && defaults.iter().all(|installed| *installed) {
        console.info(i18n::t(keys::PACKAGE_MANAGER_NO_CHANGES));
        return;
    }

    let selected_set: HashSet<usize> = selected.into_iter().collect();
    let mut actions = Vec::new();
    for (idx, pkg) in packages.iter().enumerate() {
        let installed = defaults[idx];
        let selected = selected_set.contains(&idx);
        if !installed && selected {
            actions.push((PackageAction::Install, *pkg));
        } else if installed && !selected {
            actions.push((PackageAction::Remove, *pkg));
        }
    }

    if actions.is_empty() {
        console.info(i18n::t(keys::PACKAGE_MANAGER_NO_CHANGES));
        return;
    }

    actions.sort_by_key(|(action, pkg)| {
        if *action == PackageAction::Install && pkg.id == operations::PackageId::Git {
            0
        } else {
            1
        }
    });

    if let Err(err) = ensure_curl(ctx) {
        console.error(&err.to_string());
        return;
    }

    run_actions(console, ctx, &actions);
}

fn run_update(console: &Console, prompts: &Prompts, ctx: &mut ActionContext) {
    let installed_packages: Vec<_> = package_definitions()
        .into_iter()
        .filter(|pkg| operations::is_installed(pkg.id, ctx))
        .collect();

    if installed_packages.is_empty() {
        console.warning(i18n::t(keys::PACKAGE_MANAGER_NO_INSTALLED));
        return;
    }

    let items: Vec<String> = installed_packages
        .iter()
        .map(|pkg| pkg.name.to_string())
        .collect();
    let defaults = vec![true; items.len()];

    let selected = prompts.multi_select(
        i18n::t(keys::PACKAGE_MANAGER_UPDATE_PROMPT),
        &items,
        &defaults,
    );

    if selected.is_empty() {
        console.info(i18n::t(keys::PACKAGE_MANAGER_NO_CHANGES));
        return;
    }

    if let Err(err) = ensure_curl(ctx) {
        console.error(&err.to_string());
        return;
    }

    if let Err(err) = update_curl(ctx) {
        console.warning(&crate::tr!(
            keys::PACKAGE_MANAGER_CURL_UPDATE_FAILED,
            error = err
        ));
    }

    let selected_set: HashSet<usize> = selected.into_iter().collect();
    let mut actions = Vec::new();
    for (idx, pkg) in installed_packages.iter().enumerate() {
        if selected_set.contains(&idx) {
            actions.push((PackageAction::Update, *pkg));
        }
    }

    if actions.is_empty() {
        console.info(i18n::t(keys::PACKAGE_MANAGER_NO_CHANGES));
        return;
    }

    run_actions(console, ctx, &actions);
}

fn run_actions(
    console: &Console,
    ctx: &mut ActionContext,
    actions: &[(PackageAction, operations::PackageDefinition)],
) {
    let mut success_count = 0;
    let mut failed_count = 0;

    for (idx, (action, pkg)) in actions.iter().enumerate() {
        console.show_progress(
            idx + 1,
            actions.len(),
            &crate::tr!(
                keys::PACKAGE_MANAGER_ACTION_RUNNING,
                action = action.label(),
                package = pkg.name
            ),
        );

        match operations::apply_action(*action, pkg.id, ctx) {
            Ok(()) => {
                console.success_item(&crate::tr!(
                    keys::PACKAGE_MANAGER_ACTION_SUCCESS,
                    action = action.label(),
                    package = pkg.name
                ));
                if pkg.id == operations::PackageId::Vim
                    && matches!(action, PackageAction::Install | PackageAction::Update)
                {
                    console.info(i18n::t(keys::PACKAGE_MANAGER_VIM_PLUG_HINT));
                }
                success_count += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(
                        keys::PACKAGE_MANAGER_ACTION_FAILED,
                        action = action.label(),
                        package = pkg.name
                    ),
                    &err.to_string(),
                );
                failed_count += 1;
            }
        }

        console.blank_line();
    }

    console.show_summary(
        i18n::t(keys::PACKAGE_MANAGER_SUMMARY),
        success_count,
        failed_count,
    );
}
