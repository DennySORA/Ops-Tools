mod service;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use service::KubeconfigService;

/// 執行 Kubeconfig 視窗隔離管理功能
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::KUBECONFIG_HEADER));

    let service = match KubeconfigService::new() {
        Ok(svc) => svc,
        Err(err) => {
            console.error(&err);
            return;
        }
    };

    let options = vec![
        i18n::t(keys::KUBECONFIG_ACTION_SETUP),
        i18n::t(keys::KUBECONFIG_ACTION_CLEANUP),
        i18n::t(keys::KUBECONFIG_ACTION_LIST),
        i18n::t(keys::KUBECONFIG_ACTION_CLEANUP_ALL),
    ];

    let selection = match prompts.select(i18n::t(keys::KUBECONFIG_SELECT_ACTION), &options) {
        Some(idx) => idx,
        None => {
            console.warning(i18n::t(keys::KUBECONFIG_CANCELLED));
            return;
        }
    };

    match selection {
        0 => execute_setup(&service, &console),
        1 => execute_cleanup(&service, &console, &prompts),
        2 => execute_list(&service, &console),
        3 => execute_cleanup_all(&service, &console, &prompts),
        _ => unreachable!(),
    }
}

fn execute_setup(service: &KubeconfigService, console: &Console) {
    // 檢查是否在 tmux 中
    if !service.is_in_tmux() {
        console.error(i18n::t(keys::KUBECONFIG_NOT_IN_TMUX));
        return;
    }

    // 取得 tmux 視窗 ID
    let window_id = match service.get_tmux_window_id() {
        Ok(id) => id,
        Err(err) => {
            console.error(&crate::tr!(keys::KUBECONFIG_WINDOW_ID_FAILED, error = err));
            return;
        }
    };

    console.info(&crate::tr!(keys::KUBECONFIG_WINDOW_ID, id = &window_id));

    // 建立視窗專屬的 kubeconfig
    match service.setup_window_kubeconfig(&window_id) {
        Ok(config_path) => {
            console.success(&crate::tr!(
                keys::KUBECONFIG_SETUP_SUCCESS,
                path = config_path.display()
            ));

            // 設定 tmux 環境變數
            if let Err(err) = service.set_tmux_env(&window_id, &config_path) {
                console.warning(&crate::tr!(keys::KUBECONFIG_TMUX_ENV_FAILED, error = err));
            } else {
                console.success(i18n::t(keys::KUBECONFIG_TMUX_ENV_SET));
            }

            // 自動在當前 shell 執行 export 指令
            console.blank_line();
            if let Err(err) = service.apply_shell_env(&config_path) {
                console.warning(&crate::tr!(
                    keys::KUBECONFIG_SHELL_APPLY_FAILED,
                    error = err
                ));
                console.info(i18n::t(keys::KUBECONFIG_SHELL_HINT));
                console.raw(&format!(
                    "\n  export KUBECONFIG=\"{}\"\n\n",
                    config_path.display()
                ));
            } else {
                console.success(i18n::t(keys::KUBECONFIG_SHELL_APPLIED));
            }
        }
        Err(err) => {
            console.error(&crate::tr!(keys::KUBECONFIG_SETUP_FAILED, error = err));
        }
    }
}

fn execute_cleanup(service: &KubeconfigService, console: &Console, prompts: &Prompts) {
    // 檢查是否在 tmux 中
    if !service.is_in_tmux() {
        console.error(i18n::t(keys::KUBECONFIG_NOT_IN_TMUX));
        return;
    }

    // 取得 tmux 視窗 ID
    let window_id = match service.get_tmux_window_id() {
        Ok(id) => id,
        Err(err) => {
            console.error(&crate::tr!(keys::KUBECONFIG_WINDOW_ID_FAILED, error = err));
            return;
        }
    };

    // 檢查是否有對應的 kubeconfig
    let config_path = service.get_window_kubeconfig_path(&window_id);
    if !config_path.exists() {
        console.warning(&crate::tr!(keys::KUBECONFIG_NO_CONFIG, id = &window_id));
        return;
    }

    console.info(&crate::tr!(
        keys::KUBECONFIG_FOUND_CONFIG,
        path = config_path.display()
    ));

    if !prompts.confirm_with_options(i18n::t(keys::KUBECONFIG_CONFIRM_CLEANUP), false) {
        console.warning(i18n::t(keys::KUBECONFIG_CANCELLED));
        return;
    }

    // 移除 kubeconfig 檔案
    match service.cleanup_window_kubeconfig(&window_id) {
        Ok(()) => {
            console.success(&crate::tr!(
                keys::KUBECONFIG_CLEANUP_SUCCESS,
                path = config_path.display()
            ));

            // 移除 tmux 環境變數
            if let Err(err) = service.unset_tmux_env(&window_id) {
                console.warning(&crate::tr!(
                    keys::KUBECONFIG_TMUX_ENV_UNSET_FAILED,
                    error = err
                ));
            }

            // 自動在當前 shell 執行 unset 指令
            console.blank_line();
            if let Err(err) = service.unapply_shell_env() {
                console.warning(&crate::tr!(
                    keys::KUBECONFIG_SHELL_UNAPPLY_FAILED,
                    error = err
                ));
                console.info(i18n::t(keys::KUBECONFIG_UNSET_HINT));
                console.raw("\n  unset KUBECONFIG\n\n");
            } else {
                console.success(i18n::t(keys::KUBECONFIG_SHELL_UNAPPLIED));
            }
        }
        Err(err) => {
            console.error(&crate::tr!(keys::KUBECONFIG_CLEANUP_FAILED, error = err));
        }
    }
}

fn execute_list(service: &KubeconfigService, console: &Console) {
    let configs = service.list_window_kubeconfigs();

    if configs.is_empty() {
        console.warning(i18n::t(keys::KUBECONFIG_NO_CONFIGS));
        return;
    }

    console.info(&crate::tr!(
        keys::KUBECONFIG_LIST_TITLE,
        count = configs.len()
    ));

    for config in &configs {
        console.list_item("📄", &config.display().to_string());
    }
}

fn execute_cleanup_all(service: &KubeconfigService, console: &Console, prompts: &Prompts) {
    let configs = service.list_window_kubeconfigs();

    if configs.is_empty() {
        console.warning(i18n::t(keys::KUBECONFIG_NO_CONFIGS));
        return;
    }

    console.info(&crate::tr!(
        keys::KUBECONFIG_LIST_TITLE,
        count = configs.len()
    ));

    for config in &configs {
        console.list_item("📄", &config.display().to_string());
    }

    if !prompts.confirm_with_options(i18n::t(keys::KUBECONFIG_CONFIRM_CLEANUP_ALL), false) {
        console.warning(i18n::t(keys::KUBECONFIG_CANCELLED));
        return;
    }

    let (success, failed) = service.cleanup_all_kubeconfigs();

    console.show_summary(
        i18n::t(keys::KUBECONFIG_CLEANUP_ALL_SUMMARY),
        success,
        failed,
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // 確保模組可以編譯
    }
}
