use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    #[default]
    TraditionalChinese,
    SimplifiedChinese,
    Japanese,
}

impl Language {
    pub const ALL: [Language; 4] = [
        Language::English,
        Language::TraditionalChinese,
        Language::SimplifiedChinese,
        Language::Japanese,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::TraditionalChinese => "繁體中文",
            Language::SimplifiedChinese => "简体中文",
            Language::Japanese => "日本語",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::TraditionalChinese => "zh-TW",
            Language::SimplifiedChinese => "zh-CN",
            Language::Japanese => "ja",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Language::English => 0,
            Language::TraditionalChinese => 1,
            Language::SimplifiedChinese => 2,
            Language::Japanese => 3,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Language::English),
            1 => Some(Language::TraditionalChinese),
            2 => Some(Language::SimplifiedChinese),
            3 => Some(Language::Japanese),
            _ => None,
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim() {
            "en" | "en-US" | "en-GB" => Some(Language::English),
            "zh-TW" | "zh-Hant" | "zh-Hant-TW" => Some(Language::TraditionalChinese),
            "zh-CN" | "zh-Hans" | "zh-Hans-CN" => Some(Language::SimplifiedChinese),
            "ja" | "ja-JP" => Some(Language::Japanese),
            _ => None,
        }
    }
}

struct Bundle {
    maps: HashMap<Language, HashMap<String, String>>,
}

impl Bundle {
    fn get(&self, lang: Language, key: &str) -> Option<&str> {
        self.maps
            .get(&lang)
            .and_then(|map| map.get(key).map(|value| value.as_str()))
    }
}

static BUNDLE: OnceLock<Bundle> = OnceLock::new();
static CURRENT_LANGUAGE: OnceLock<RwLock<Language>> = OnceLock::new();

fn load_locale(raw: &str) -> HashMap<String, String> {
    toml::from_str(raw).expect("Invalid locale data")
}

fn bundle() -> &'static Bundle {
    BUNDLE.get_or_init(|| {
        let mut maps = HashMap::new();
        maps.insert(
            Language::English,
            load_locale(include_str!("locales/en.toml")),
        );
        maps.insert(
            Language::TraditionalChinese,
            load_locale(include_str!("locales/zh-TW.toml")),
        );
        maps.insert(
            Language::SimplifiedChinese,
            load_locale(include_str!("locales/zh-CN.toml")),
        );
        maps.insert(
            Language::Japanese,
            load_locale(include_str!("locales/ja.toml")),
        );
        Bundle { maps }
    })
}

fn language_lock() -> &'static RwLock<Language> {
    CURRENT_LANGUAGE.get_or_init(|| RwLock::new(Language::default()))
}

pub fn current_language() -> Language {
    *language_lock().read().expect("Language lock poisoned")
}

pub fn set_language(language: Language) {
    *language_lock().write().expect("Language lock poisoned") = language;
}

pub fn t(key: &str) -> &'static str {
    let bundle = bundle();
    let language = current_language();
    bundle
        .get(language, key)
        .or_else(|| bundle.get(Language::English, key))
        .unwrap_or("??")
}

#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        $crate::i18n::t($key).to_string()
    };
    ($key:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut output = $crate::i18n::t($key).to_string();
        $(
            output = output.replace(concat!("{", stringify!($name), "}"), &$value.to_string());
        )+
        output
    }};
}

pub mod keys {
    pub const MENU_PROMPT: &str = "menu.prompt";
    pub const MENU_TERRAFORM_CLEANER: &str = "menu.terraform_cleaner.name";
    pub const MENU_TERRAFORM_CLEANER_DESC: &str = "menu.terraform_cleaner.desc";
    pub const MENU_TOOL_UPGRADER: &str = "menu.tool_upgrader.name";
    pub const MENU_TOOL_UPGRADER_DESC: &str = "menu.tool_upgrader.desc";
    pub const MENU_PACKAGE_MANAGER: &str = "menu.package_manager.name";
    pub const MENU_PACKAGE_MANAGER_DESC: &str = "menu.package_manager.desc";
    pub const MENU_RUST_UPGRADER: &str = "menu.rust_upgrader.name";
    pub const MENU_RUST_UPGRADER_DESC: &str = "menu.rust_upgrader.desc";
    pub const MENU_SECURITY_SCANNER: &str = "menu.security_scanner.name";
    pub const MENU_SECURITY_SCANNER_DESC: &str = "menu.security_scanner.desc";
    pub const MENU_MCP_MANAGER: &str = "menu.mcp_manager.name";
    pub const MENU_MCP_MANAGER_DESC: &str = "menu.mcp_manager.desc";
    pub const MENU_PROMPT_GEN: &str = "menu.prompt_gen.name";
    pub const MENU_PROMPT_GEN_DESC: &str = "menu.prompt_gen.desc";
    pub const MENU_KUBECONFIG_MANAGER: &str = "menu.kubeconfig_manager.name";
    pub const MENU_KUBECONFIG_MANAGER_DESC: &str = "menu.kubeconfig_manager.desc";
    pub const MENU_RUST_BUILDER: &str = "menu.rust_builder.name";
    pub const MENU_RUST_BUILDER_DESC: &str = "menu.rust_builder.desc";
    pub const MENU_CATEGORY_BUILD: &str = "menu.category.build.name";
    pub const MENU_CATEGORY_BUILD_DESC: &str = "menu.category.build.desc";
    pub const MENU_CATEGORY_AI: &str = "menu.category.ai.name";
    pub const MENU_CATEGORY_AI_DESC: &str = "menu.category.ai.desc";
    pub const MENU_CATEGORY_UPGRADE: &str = "menu.category.upgrade.name";
    pub const MENU_CATEGORY_UPGRADE_DESC: &str = "menu.category.upgrade.desc";
    pub const MENU_CATEGORY_INFRA: &str = "menu.category.infra.name";
    pub const MENU_CATEGORY_INFRA_DESC: &str = "menu.category.infra.desc";
    pub const MENU_CATEGORY_SECURITY: &str = "menu.category.security.name";
    pub const MENU_CATEGORY_SECURITY_DESC: &str = "menu.category.security.desc";
    pub const MENU_COMMON: &str = "menu.common.name";
    pub const MENU_CATEGORIES: &str = "menu.categories.name";
    pub const MENU_BACK: &str = "menu.back";
    pub const MENU_CATEGORY_PROMPT: &str = "menu.category.prompt";
    pub const MENU_SETTINGS: &str = "menu.settings.name";
    pub const MENU_SETTINGS_DESC: &str = "menu.settings.desc";
    pub const MENU_LANGUAGE: &str = "menu.language.name";
    pub const MENU_LANGUAGE_DESC: &str = "menu.language.desc";
    pub const MENU_EXIT: &str = "menu.exit";
    pub const MENU_GOODBYE: &str = "menu.goodbye";
    pub const MENU_PINNED: &str = "menu.pinned.name";
    pub const MENU_PIN_MANAGE: &str = "menu.pin.manage.name";
    pub const MENU_PIN_MANAGE_DESC: &str = "menu.pin.manage.desc";
    pub const MENU_PIN_PROMPT: &str = "menu.pin.prompt";
    pub const MENU_PIN_ICON: &str = "menu.pin.icon";
    pub const MENU_PIN_COUNT: &str = "menu.pin.count";
    pub const MENU_PIN_CLEARED: &str = "menu.pin.cleared";
    pub const MENU_PIN_REORDER: &str = "menu.pin.reorder.name";
    pub const MENU_PIN_REORDER_DESC: &str = "menu.pin.reorder.desc";
    pub const MENU_PIN_REORDER_PROMPT: &str = "menu.pin.reorder.prompt";
    pub const MENU_PIN_REORDER_DONE: &str = "menu.pin.reorder.done";
    pub const MENU_PIN_REORDER_EMPTY: &str = "menu.pin.reorder.empty";

    pub const LANGUAGE_SELECT_PROMPT: &str = "language.select_prompt";
    pub const LANGUAGE_CHANGED: &str = "language.changed";

    pub const CONFIG_LOAD_FAILED: &str = "config.load_failed";
    pub const CONFIG_SAVE_FAILED: &str = "config.save_failed";
    pub const CONFIG_LANGUAGE_INVALID: &str = "config.language_invalid";

    pub const CONSOLE_ERROR_PREFIX: &str = "console.error_prefix";
    pub const CONSOLE_SUMMARY: &str = "console.summary";

    pub const PROMPT_YES: &str = "prompt.yes";
    pub const PROMPT_NO: &str = "prompt.no";

    pub const ERROR_IO: &str = "error.io";
    pub const ERROR_COMMAND: &str = "error.command";
    pub const ERROR_CONFIG: &str = "error.config";
    pub const ERROR_VALIDATION: &str = "error.validation";
    pub const ERROR_CANCELLED: &str = "error.cancelled";
    pub const ERROR_UNABLE_TO_EXECUTE: &str = "error.unable_to_execute";
    pub const ERROR_UNKNOWN: &str = "error.unknown";
    pub const ERROR_COMMAND_NOT_FOUND: &str = "error.command_not_found";

    pub const TERRAFORM_CURRENT_DIR_FAILED: &str = "terraform.current_dir_failed";
    pub const TERRAFORM_SCAN_START: &str = "terraform.scan_start";
    pub const TERRAFORM_SCAN_DIR: &str = "terraform.scan_dir";
    pub const TERRAFORM_NO_CACHE: &str = "terraform.no_cache";
    pub const TERRAFORM_FOUND_ITEMS: &str = "terraform.found_items";
    pub const TERRAFORM_ITEM_DIR: &str = "terraform.item_dir";
    pub const TERRAFORM_ITEM_FILE: &str = "terraform.item_file";
    pub const TERRAFORM_CONFIRM_DELETE: &str = "terraform.confirm_delete";
    pub const TERRAFORM_DELETE_CANCELLED: &str = "terraform.delete_cancelled";
    pub const TERRAFORM_DELETED: &str = "terraform.deleted";
    pub const TERRAFORM_DELETE_FAILED: &str = "terraform.delete_failed";
    pub const TERRAFORM_SUMMARY_TITLE: &str = "terraform.summary_title";
    pub const TERRAFORM_PROGRESS_SCANNING: &str = "terraform.progress_scanning";
    pub const TERRAFORM_PROGRESS_SCANNED: &str = "terraform.progress_scanned";
    pub const TERRAFORM_PROGRESS_DELETING: &str = "terraform.progress_deleting";
    pub const TERRAFORM_PROGRESS_DELETED: &str = "terraform.progress_deleted";

    pub const TOOL_UPGRADER_HEADER: &str = "tool_upgrader.header";
    pub const TOOL_UPGRADER_LIST_TITLE: &str = "tool_upgrader.list_title";
    pub const TOOL_UPGRADER_CONFIRM: &str = "tool_upgrader.confirm";
    pub const TOOL_UPGRADER_CANCELLED: &str = "tool_upgrader.cancelled";
    pub const TOOL_UPGRADER_PROGRESS: &str = "tool_upgrader.progress";
    pub const TOOL_UPGRADER_SUCCESS: &str = "tool_upgrader.success";
    pub const TOOL_UPGRADER_FAILED: &str = "tool_upgrader.failed";
    pub const TOOL_UPGRADER_SUMMARY: &str = "tool_upgrader.summary";

    pub const PACKAGE_MANAGER_HEADER: &str = "package_manager.header";
    pub const PACKAGE_MANAGER_UNSUPPORTED_OS: &str = "package_manager.unsupported_os";
    pub const PACKAGE_MANAGER_MODE_PROMPT: &str = "package_manager.mode_prompt";
    pub const PACKAGE_MANAGER_MODE_INSTALL: &str = "package_manager.mode_install";
    pub const PACKAGE_MANAGER_MODE_UPDATE: &str = "package_manager.mode_update";
    pub const PACKAGE_MANAGER_INSTALL_PROMPT: &str = "package_manager.install_prompt";
    pub const PACKAGE_MANAGER_UPDATE_PROMPT: &str = "package_manager.update_prompt";
    pub const PACKAGE_MANAGER_NO_CHANGES: &str = "package_manager.no_changes";
    pub const PACKAGE_MANAGER_NO_INSTALLED: &str = "package_manager.no_installed";
    pub const PACKAGE_MANAGER_CANCELLED: &str = "package_manager.cancelled";
    pub const PACKAGE_MANAGER_ACTION_RUNNING: &str = "package_manager.action_running";
    pub const PACKAGE_MANAGER_ACTION_SUCCESS: &str = "package_manager.action_success";
    pub const PACKAGE_MANAGER_ACTION_FAILED: &str = "package_manager.action_failed";
    pub const PACKAGE_MANAGER_SUMMARY: &str = "package_manager.summary";
    pub const PACKAGE_MANAGER_ACTION_INSTALL: &str = "package_manager.action.install";
    pub const PACKAGE_MANAGER_ACTION_UPDATE: &str = "package_manager.action.update";
    pub const PACKAGE_MANAGER_ACTION_REMOVE: &str = "package_manager.action.remove";
    pub const PACKAGE_MANAGER_CURL_UPDATE_FAILED: &str = "package_manager.curl_update_failed";
    pub const PACKAGE_MANAGER_MISSING_PM: &str = "package_manager.missing_pm";
    pub const PACKAGE_MANAGER_RUSTUP_MISSING: &str = "package_manager.rustup_missing";
    pub const PACKAGE_MANAGER_GO_VERSION_MISSING: &str = "package_manager.go_version_missing";
    pub const PACKAGE_MANAGER_GO_FILE_MISSING: &str = "package_manager.go_file_missing";
    pub const PACKAGE_MANAGER_CODENAME_MISSING: &str = "package_manager.codename_missing";
    pub const PACKAGE_MANAGER_ARCH_UNSUPPORTED: &str = "package_manager.arch_unsupported";
    pub const PACKAGE_MANAGER_GIT_REQUIRED: &str = "package_manager.git_required";
    pub const PACKAGE_MANAGER_BINARY_NOT_FOUND: &str = "package_manager.binary_not_found";
    pub const PACKAGE_MANAGER_RELEASE_ASSET_MISSING: &str = "package_manager.release_asset_missing";
    pub const PACKAGE_MANAGER_UV_MISSING: &str = "package_manager.uv_missing";
    pub const PACKAGE_MANAGER_SUDO_REQUIRED: &str = "package_manager.sudo_required";
    pub const PACKAGE_MANAGER_VIM_PLUG_HINT: &str = "package_manager.vim_plug_hint";

    pub const RUST_UPGRADER_HEADER: &str = "rust_upgrader.header";
    pub const RUST_UPGRADER_CHECKING_ENV: &str = "rust_upgrader.checking_env";
    pub const RUST_UPGRADER_ENV_INSTALLED: &str = "rust_upgrader.env_installed";
    pub const RUST_UPGRADER_ENV_MISSING: &str = "rust_upgrader.env_missing";
    pub const RUST_UPGRADER_INSTALL_RUST_HINT: &str = "rust_upgrader.install_rust_hint";
    pub const RUST_UPGRADER_CHECKING_TOOLS: &str = "rust_upgrader.checking_tools";
    pub const RUST_UPGRADER_TOOL_INSTALLED: &str = "rust_upgrader.tool_installed";
    pub const RUST_UPGRADER_TOOL_MISSING: &str = "rust_upgrader.tool_missing";
    pub const RUST_UPGRADER_MISSING_TOOLS: &str = "rust_upgrader.missing_tools";
    pub const RUST_UPGRADER_CONFIRM_INSTALL_TOOLS: &str = "rust_upgrader.confirm_install_tools";
    pub const RUST_UPGRADER_INSTALLING_TOOL: &str = "rust_upgrader.installing_tool";
    pub const RUST_UPGRADER_INSTALL_SUCCESS: &str = "rust_upgrader.install_success";
    pub const RUST_UPGRADER_INSTALL_FAILED: &str = "rust_upgrader.install_failed";
    pub const RUST_UPGRADER_SKIP_INSTALL: &str = "rust_upgrader.skip_install";
    pub const RUST_UPGRADER_ALL_TOOLS_INSTALLED: &str = "rust_upgrader.all_tools_installed";
    pub const RUST_UPGRADER_UPGRADE_STEPS: &str = "rust_upgrader.upgrade_steps";
    pub const RUST_UPGRADER_REQUIRES_PROJECT_TAG: &str = "rust_upgrader.requires_project_tag";
    pub const RUST_UPGRADER_CONFIRM_UPGRADE: &str = "rust_upgrader.confirm_upgrade";
    pub const RUST_UPGRADER_CANCELLED: &str = "rust_upgrader.cancelled";
    pub const RUST_UPGRADER_RUNNING_STEP: &str = "rust_upgrader.running_step";
    pub const RUST_UPGRADER_STEP_DONE: &str = "rust_upgrader.step_done";
    pub const RUST_UPGRADER_STEP_SKIPPED: &str = "rust_upgrader.step_skipped";
    pub const RUST_UPGRADER_STEP_FAILED: &str = "rust_upgrader.step_failed";
    pub const RUST_UPGRADER_SUMMARY: &str = "rust_upgrader.summary";
    pub const RUST_UPGRADER_SKIPPED_COUNT: &str = "rust_upgrader.skipped_count";
    pub const RUST_UPGRADER_OUTPUT_MORE_LINES: &str = "rust_upgrader.output_more_lines";

    pub const RUST_BUILDER_HEADER: &str = "rust_builder.header";
    pub const RUST_BUILDER_NO_CARGO_TOML: &str = "rust_builder.no_cargo_toml";
    pub const RUST_BUILDER_RUSTUP_MISSING: &str = "rust_builder.rustup_missing";
    pub const RUST_BUILDER_SELECT_BUILDER: &str = "rust_builder.select_builder";
    pub const RUST_BUILDER_BUILDER_CARGO: &str = "rust_builder.builder.cargo";
    pub const RUST_BUILDER_BUILDER_CROSS: &str = "rust_builder.builder.cross";
    pub const RUST_BUILDER_SELECT_PROFILE: &str = "rust_builder.select_profile";
    pub const RUST_BUILDER_PROFILE_RELEASE: &str = "rust_builder.profile.release";
    pub const RUST_BUILDER_PROFILE_DEBUG: &str = "rust_builder.profile.debug";
    pub const RUST_BUILDER_SELECT_TARGETS: &str = "rust_builder.select_targets";
    pub const RUST_BUILDER_NO_TARGET_SELECTED: &str = "rust_builder.no_target_selected";
    pub const RUST_BUILDER_MISSING_TARGETS: &str = "rust_builder.missing_targets";
    pub const RUST_BUILDER_CONFIRM_INSTALL_TARGETS: &str = "rust_builder.confirm_install_targets";
    pub const RUST_BUILDER_INSTALLING_TARGET: &str = "rust_builder.installing_target";
    pub const RUST_BUILDER_INSTALL_SUCCESS: &str = "rust_builder.install_success";
    pub const RUST_BUILDER_INSTALL_FAILED: &str = "rust_builder.install_failed";
    pub const RUST_BUILDER_SKIP_INSTALL: &str = "rust_builder.skip_install";
    pub const RUST_BUILDER_BUILDING: &str = "rust_builder.building";
    pub const RUST_BUILDER_BUILD_SUCCESS: &str = "rust_builder.build_success";
    pub const RUST_BUILDER_BUILD_FAILED: &str = "rust_builder.build_failed";
    pub const RUST_BUILDER_SUMMARY_TITLE: &str = "rust_builder.summary_title";
    pub const RUST_BUILDER_CANCELLED: &str = "rust_builder.cancelled";

    pub const RUST_BUILDER_TARGET_LINUX_X86_64_GNU: &str = "rust_builder.target.linux_x86_64_gnu";
    pub const RUST_BUILDER_TARGET_LINUX_ARM64_GNU: &str = "rust_builder.target.linux_arm64_gnu";
    pub const RUST_BUILDER_TARGET_LINUX_I686_GNU: &str = "rust_builder.target.linux_i686_gnu";
    pub const RUST_BUILDER_TARGET_LINUX_ARMV7_GNU: &str = "rust_builder.target.linux_armv7_gnu";
    pub const RUST_BUILDER_TARGET_LINUX_RISCV64_GNU: &str = "rust_builder.target.linux_riscv64_gnu";
    pub const RUST_BUILDER_TARGET_LINUX_PPC64LE_GNU: &str = "rust_builder.target.linux_ppc64le_gnu";
    pub const RUST_BUILDER_TARGET_LINUX_X86_64_MUSL: &str = "rust_builder.target.linux_x86_64_musl";
    pub const RUST_BUILDER_TARGET_LINUX_ARM64_MUSL: &str = "rust_builder.target.linux_arm64_musl";
    pub const RUST_BUILDER_TARGET_LINUX_I686_MUSL: &str = "rust_builder.target.linux_i686_musl";
    pub const RUST_BUILDER_TARGET_LINUX_ARMV7_MUSL: &str = "rust_builder.target.linux_armv7_musl";
    pub const RUST_BUILDER_TARGET_MACOS_X86_64: &str = "rust_builder.target.macos_x86_64";
    pub const RUST_BUILDER_TARGET_MACOS_ARM64: &str = "rust_builder.target.macos_arm64";
    pub const RUST_BUILDER_TARGET_WINDOWS_X86_64: &str = "rust_builder.target.windows_x86_64";
    pub const RUST_BUILDER_TARGET_WINDOWS_ARM64: &str = "rust_builder.target.windows_arm64";
    pub const RUST_BUILDER_TARGET_WASM32_UNKNOWN: &str = "rust_builder.target.wasm32_unknown";
    pub const RUST_UPGRADER_VALIDATION_MISSING_CARGO: &str =
        "rust_upgrader.validation_missing_cargo";
    pub const RUST_UPGRADER_RUST_MISSING_OR_UNAVAILABLE: &str =
        "rust_upgrader.rust_missing_or_unavailable";
    pub const RUST_UPGRADER_VERSION_UNAVAILABLE: &str = "rust_upgrader.version_unavailable";
    pub const RUST_UPGRADER_STEP_DESC_RUSTUP_SELF_UPDATE: &str =
        "rust_upgrader.step_desc.rustup_self_update";
    pub const RUST_UPGRADER_STEP_DESC_RUSTUP_UPDATE: &str = "rust_upgrader.step_desc.rustup_update";
    pub const RUST_UPGRADER_STEP_DESC_CARGO_INSTALL_UPDATE: &str =
        "rust_upgrader.step_desc.cargo_install_update";
    pub const RUST_UPGRADER_STEP_DESC_CARGO_UPGRADE: &str = "rust_upgrader.step_desc.cargo_upgrade";
    pub const RUST_UPGRADER_STEP_DESC_CARGO_OUTDATED: &str =
        "rust_upgrader.step_desc.cargo_outdated";
    pub const RUST_UPGRADER_STEP_DESC_CARGO_AUDIT: &str = "rust_upgrader.step_desc.cargo_audit";

    pub const SECURITY_SCANNER_HEADER: &str = "security_scanner.header";
    pub const SECURITY_SCANNER_CURRENT_DIR_FAILED: &str = "security_scanner.current_dir_failed";
    pub const SECURITY_SCANNER_NOT_GIT_REPO: &str = "security_scanner.not_git_repo";
    pub const SECURITY_SCANNER_GIT_NOT_FOUND: &str = "security_scanner.git_not_found";
    pub const SECURITY_SCANNER_SCAN_DIR: &str = "security_scanner.scan_dir";
    pub const SECURITY_SCANNER_STRICT_MODE: &str = "security_scanner.strict_mode";
    pub const SECURITY_SCANNER_TOOLS_INTRO: &str = "security_scanner.tools_intro";
    pub const SECURITY_SCANNER_STATUS_INSTALLED: &str = "security_scanner.status_installed";
    pub const SECURITY_SCANNER_STATUS_MISSING: &str = "security_scanner.status_missing";
    pub const SECURITY_SCANNER_CONFIRM_INSTALL: &str = "security_scanner.confirm_install";
    pub const SECURITY_SCANNER_CANCELLED: &str = "security_scanner.cancelled";
    pub const SECURITY_SCANNER_INSTALLING: &str = "security_scanner.installing";
    pub const SECURITY_SCANNER_INSTALL_DONE: &str = "security_scanner.install_done";
    pub const SECURITY_SCANNER_INSTALL_ALREADY: &str = "security_scanner.install_already";
    pub const SECURITY_SCANNER_INSTALL_FAILED: &str = "security_scanner.install_failed";
    pub const SECURITY_SCANNER_INSTALL_SUMMARY: &str = "security_scanner.install_summary";
    pub const SECURITY_SCANNER_SKIP_TOOL: &str = "security_scanner.skip_tool";
    pub const SECURITY_SCANNER_START_SCAN: &str = "security_scanner.start_scan";
    pub const SECURITY_SCANNER_STDOUT_TITLE: &str = "security_scanner.stdout_title";
    pub const SECURITY_SCANNER_STDERR_TITLE: &str = "security_scanner.stderr_title";
    pub const SECURITY_SCANNER_NO_OUTPUT: &str = "security_scanner.no_output";
    pub const SECURITY_SCANNER_PASSED: &str = "security_scanner.passed";
    pub const SECURITY_SCANNER_FINDINGS: &str = "security_scanner.findings";
    pub const SECURITY_SCANNER_SCAN_FAILED: &str = "security_scanner.scan_failed";
    pub const SECURITY_SCANNER_SCAN_SUMMARY: &str = "security_scanner.scan_summary";
    pub const SECURITY_SCANNER_FINDINGS_WARNING: &str = "security_scanner.findings_warning";
    pub const SECURITY_SCANNER_EXIT_CODE: &str = "security_scanner.exit_code";
    pub const SECURITY_SCANNER_EXIT_CODE_UNKNOWN: &str = "security_scanner.exit_code_unknown";
    pub const SECURITY_SCANNER_NO_TRACKED_FILES: &str = "security_scanner.no_tracked_files";
    pub const SECURITY_SCANNER_ALL_IGNORED: &str = "security_scanner.all_ignored";
    pub const SECURITY_SCANNER_SCOPE_GIT_HISTORY: &str = "security_scanner.scope.git_history";
    pub const SECURITY_SCANNER_SCOPE_WORKTREE: &str = "security_scanner.scope.worktree";
    pub const SECURITY_SCANNER_COMMAND_LABEL: &str = "security_scanner.command_label";
    pub const SECURITY_SCANNER_INSTALL_MISSING_AFTER: &str =
        "security_scanner.install_missing_after";
    pub const SECURITY_SCANNER_INSTALL_STRATEGY_FAILED: &str =
        "security_scanner.install_strategy_failed";
    pub const SECURITY_SCANNER_INSTALL_NO_STRATEGY: &str = "security_scanner.install_no_strategy";
    pub const SECURITY_SCANNER_UNSUPPORTED_PLATFORM: &str = "security_scanner.unsupported_platform";
    pub const SECURITY_SCANNER_RELEASE_NOT_FOUND: &str = "security_scanner.release_not_found";
    pub const SECURITY_SCANNER_EXTRACT_MISSING_BINARY: &str =
        "security_scanner.extract_missing_binary";
    pub const SECURITY_SCANNER_RELEASE_PARSE_FAILED: &str = "security_scanner.release_parse_failed";
    pub const SECURITY_SCANNER_RELEASE_MISSING_ASSETS: &str =
        "security_scanner.release_missing_assets";
    pub const SECURITY_SCANNER_DOWNLOAD_TOOL_MISSING: &str =
        "security_scanner.download_tool_missing";
    pub const SECURITY_SCANNER_TAR_MISSING: &str = "security_scanner.tar_missing";
    pub const SECURITY_SCANNER_UNZIP_MISSING: &str = "security_scanner.unzip_missing";
    pub const SECURITY_SCANNER_INSTALL_DIR_MISSING: &str = "security_scanner.install_dir_missing";

    pub const MCP_MANAGER_HEADER: &str = "mcp_manager.header";
    pub const MCP_MANAGER_SELECT_CLI: &str = "mcp_manager.select_cli";
    pub const MCP_MANAGER_CANCELLED: &str = "mcp_manager.cancelled";
    pub const MCP_MANAGER_USING_CLI: &str = "mcp_manager.using_cli";
    pub const MCP_MANAGER_SCANNING: &str = "mcp_manager.scanning";
    pub const MCP_MANAGER_NONE_INSTALLED: &str = "mcp_manager.none_installed";
    pub const MCP_MANAGER_FOUND_INSTALLED: &str = "mcp_manager.found_installed";
    pub const MCP_MANAGER_STATUS_INSTALLED: &str = "mcp_manager.status_installed";
    pub const MCP_MANAGER_STATUS_MISSING: &str = "mcp_manager.status_missing";
    pub const MCP_MANAGER_SELECT_INSTALL: &str = "mcp_manager.select_install";
    pub const MCP_MANAGER_SELECT_HELP: &str = "mcp_manager.select_help";
    pub const MCP_MANAGER_SELECT_PROMPT: &str = "mcp_manager.select_prompt";
    pub const MCP_MANAGER_NO_CHANGES: &str = "mcp_manager.no_changes";
    pub const MCP_MANAGER_CHANGE_SUMMARY: &str = "mcp_manager.change_summary";
    pub const MCP_MANAGER_WILL_INSTALL: &str = "mcp_manager.will_install";
    pub const MCP_MANAGER_WILL_REMOVE: &str = "mcp_manager.will_remove";
    pub const MCP_MANAGER_CONFIRM_CHANGES: &str = "mcp_manager.confirm_changes";
    pub const MCP_MANAGER_OAUTH_HINT: &str = "mcp_manager.oauth_hint";
    pub const MCP_MANAGER_WSL_HINT: &str = "mcp_manager.wsl_hint";
    pub const MCP_MANAGER_INSTALLING: &str = "mcp_manager.installing";
    pub const MCP_MANAGER_INSTALL_SUCCESS: &str = "mcp_manager.install_success";
    pub const MCP_MANAGER_INSTALL_FAILED: &str = "mcp_manager.install_failed";
    pub const MCP_MANAGER_REMOVING: &str = "mcp_manager.removing";
    pub const MCP_MANAGER_REMOVE_SUCCESS: &str = "mcp_manager.remove_success";
    pub const MCP_MANAGER_REMOVE_FAILED: &str = "mcp_manager.remove_failed";
    pub const MCP_MANAGER_SUMMARY: &str = "mcp_manager.summary";

    pub const MCP_EXECUTOR_INTERACTIVE_FAILED: &str = "mcp_executor.interactive_failed";
    pub const MCP_EXECUTOR_CONFIG_PARSE_FAILED: &str = "mcp_executor.config_parse_failed";
    pub const MCP_EXECUTOR_CONFIG_SERIALIZE_FAILED: &str = "mcp_executor.config_serialize_failed";

    pub const MCP_TOOL_SEQUENTIAL_THINKING: &str = "mcp.tool.sequential_thinking";
    pub const MCP_TOOL_CHROME_DEVTOOLS: &str = "mcp.tool.chrome_devtools";
    pub const MCP_TOOL_KUBERNETES: &str = "mcp.tool.kubernetes";
    pub const MCP_TOOL_CONTEXT7: &str = "mcp.tool.context7";
    pub const MCP_TOOL_GITHUB: &str = "mcp.tool.github";
    pub const MCP_TOOL_CLOUDFLARE_DOCS: &str = "mcp.tool.cloudflare_docs";
    pub const MCP_TOOL_CLOUDFLARE_WORKERS_BINDINGS: &str = "mcp.tool.cloudflare_workers_bindings";
    pub const MCP_TOOL_CLOUDFLARE_WORKERS_BUILDS: &str = "mcp.tool.cloudflare_workers_builds";
    pub const MCP_TOOL_CLOUDFLARE_OBSERVABILITY: &str = "mcp.tool.cloudflare_observability";
    pub const MCP_TOOL_CLOUDFLARE_RADAR: &str = "mcp.tool.cloudflare_radar";
    pub const MCP_TOOL_CLOUDFLARE_CONTAINERS: &str = "mcp.tool.cloudflare_containers";
    pub const MCP_TOOL_CLOUDFLARE_BROWSER: &str = "mcp.tool.cloudflare_browser";
    pub const MCP_TOOL_CLOUDFLARE_LOGPUSH: &str = "mcp.tool.cloudflare_logpush";
    pub const MCP_TOOL_CLOUDFLARE_AI_GATEWAY: &str = "mcp.tool.cloudflare_ai_gateway";
    pub const MCP_TOOL_CLOUDFLARE_AUTORAG: &str = "mcp.tool.cloudflare_autorag";
    pub const MCP_TOOL_CLOUDFLARE_AUDITLOGS: &str = "mcp.tool.cloudflare_auditlogs";
    pub const MCP_TOOL_CLOUDFLARE_DNS_ANALYTICS: &str = "mcp.tool.cloudflare_dns_analytics";
    pub const MCP_TOOL_CLOUDFLARE_DEX: &str = "mcp.tool.cloudflare_dex";
    pub const MCP_TOOL_CLOUDFLARE_CASB: &str = "mcp.tool.cloudflare_casb";
    pub const MCP_TOOL_CLOUDFLARE_GRAPHQL: &str = "mcp.tool.cloudflare_graphql";
    pub const MCP_TOOL_TAILWINDCSS: &str = "mcp.tool.tailwindcss";
    pub const MCP_TOOL_ARXIV: &str = "mcp.tool.arxiv";

    // Kubeconfig Manager
    pub const KUBECONFIG_HEADER: &str = "kubeconfig.header";
    pub const KUBECONFIG_SELECT_ACTION: &str = "kubeconfig.select_action";
    pub const KUBECONFIG_ACTION_SETUP: &str = "kubeconfig.action_setup";
    pub const KUBECONFIG_ACTION_CLEANUP: &str = "kubeconfig.action_cleanup";
    pub const KUBECONFIG_ACTION_LIST: &str = "kubeconfig.action_list";
    pub const KUBECONFIG_ACTION_CLEANUP_ALL: &str = "kubeconfig.action_cleanup_all";
    pub const KUBECONFIG_CANCELLED: &str = "kubeconfig.cancelled";
    pub const KUBECONFIG_NOT_IN_TMUX: &str = "kubeconfig.not_in_tmux";
    pub const KUBECONFIG_WINDOW_ID: &str = "kubeconfig.window_id";
    pub const KUBECONFIG_WINDOW_ID_FAILED: &str = "kubeconfig.window_id_failed";
    pub const KUBECONFIG_SETUP_SUCCESS: &str = "kubeconfig.setup_success";
    pub const KUBECONFIG_SETUP_FAILED: &str = "kubeconfig.setup_failed";
    pub const KUBECONFIG_TMUX_ENV_SET: &str = "kubeconfig.tmux_env_set";
    pub const KUBECONFIG_TMUX_ENV_FAILED: &str = "kubeconfig.tmux_env_failed";
    pub const KUBECONFIG_SHELL_HINT: &str = "kubeconfig.shell_hint";
    pub const KUBECONFIG_SHELL_APPLIED: &str = "kubeconfig.shell_applied";
    pub const KUBECONFIG_SHELL_APPLY_FAILED: &str = "kubeconfig.shell_apply_failed";
    pub const KUBECONFIG_SHELL_UNAPPLIED: &str = "kubeconfig.shell_unapplied";
    pub const KUBECONFIG_SHELL_UNAPPLY_FAILED: &str = "kubeconfig.shell_unapply_failed";
    pub const KUBECONFIG_NO_CONFIG: &str = "kubeconfig.no_config";
    pub const KUBECONFIG_FOUND_CONFIG: &str = "kubeconfig.found_config";
    pub const KUBECONFIG_CONFIRM_CLEANUP: &str = "kubeconfig.confirm_cleanup";
    pub const KUBECONFIG_CLEANUP_SUCCESS: &str = "kubeconfig.cleanup_success";
    pub const KUBECONFIG_CLEANUP_FAILED: &str = "kubeconfig.cleanup_failed";
    pub const KUBECONFIG_TMUX_ENV_UNSET_FAILED: &str = "kubeconfig.tmux_env_unset_failed";
    pub const KUBECONFIG_UNSET_HINT: &str = "kubeconfig.unset_hint";
    pub const KUBECONFIG_NO_CONFIGS: &str = "kubeconfig.no_configs";
    pub const KUBECONFIG_LIST_TITLE: &str = "kubeconfig.list_title";
    pub const KUBECONFIG_CONFIRM_CLEANUP_ALL: &str = "kubeconfig.confirm_cleanup_all";
    pub const KUBECONFIG_CLEANUP_ALL_SUMMARY: &str = "kubeconfig.cleanup_all_summary";

    // Prompt Generator
    pub const PROMPT_GEN_HEADER: &str = "prompt_gen.header";
    pub const PROMPT_GEN_SELECT_ACTION: &str = "prompt_gen.select_action";
    pub const PROMPT_GEN_ACTION_GENERATE: &str = "prompt_gen.action_generate";
    pub const PROMPT_GEN_ACTION_RUN: &str = "prompt_gen.action_run";
    pub const PROMPT_GEN_ACTION_STATUS: &str = "prompt_gen.action_status";
    pub const PROMPT_GEN_CANCELLED: &str = "prompt_gen.cancelled";
    pub const PROMPT_GEN_INPUT_SPEC_FILE: &str = "prompt_gen.input_spec_file";
    pub const PROMPT_GEN_INPUT_OUTPUT_DIR: &str = "prompt_gen.input_output_dir";
    pub const PROMPT_GEN_INPUT_FEATURES_DIR: &str = "prompt_gen.input_features_dir";
    pub const PROMPT_GEN_CONFIRM_OVERWRITE: &str = "prompt_gen.confirm_overwrite";
    pub const PROMPT_GEN_GENERATING: &str = "prompt_gen.generating";
    pub const PROMPT_GEN_GENERATED: &str = "prompt_gen.generated";
    pub const PROMPT_GEN_LOADED_FEATURES: &str = "prompt_gen.loaded_features";
    pub const PROMPT_GEN_FEATURE_GENERATED: &str = "prompt_gen.feature_generated";
    pub const PROMPT_GEN_RUNNING: &str = "prompt_gen.running";
    pub const PROMPT_GEN_FILE_NOT_FOUND: &str = "prompt_gen.file_not_found";
    pub const PROMPT_GEN_DIR_NOT_FOUND: &str = "prompt_gen.dir_not_found";
    pub const PROMPT_GEN_STATUS_TOTAL: &str = "prompt_gen.status_total";
    pub const PROMPT_GEN_STATUS_READY: &str = "prompt_gen.status_ready";
    pub const PROMPT_GEN_STATUS_IN_PROGRESS: &str = "prompt_gen.status_in_progress";
    pub const PROMPT_GEN_STATUS_NOT_STARTED: &str = "prompt_gen.status_not_started";
    pub const PROMPT_GEN_SELECT_CLI: &str = "prompt_gen.select_cli";
    pub const PROMPT_GEN_USING_CLI: &str = "prompt_gen.using_cli";
    pub const PROMPT_GEN_ACTION_VALIDATE: &str = "prompt_gen.action_validate";
    pub const PROMPT_GEN_VALIDATING: &str = "prompt_gen.validating";
    pub const PROMPT_GEN_VALIDATE_SUCCESS: &str = "prompt_gen.validate_success";
    pub const PROMPT_GEN_VALIDATE_FAILED: &str = "prompt_gen.validate_failed";

    // YAML Prompt 生成
    pub const PROMPT_GEN_ACTION_YAML_PROMPT: &str = "prompt_gen.action_yaml_prompt";
    pub const PROMPT_GEN_SELECT_PROJECT_TYPE: &str = "prompt_gen.select_project_type";
    pub const PROMPT_GEN_HAS_VERIFICATION_ENV: &str = "prompt_gen.has_verification_env";
    pub const PROMPT_GEN_NEEDS_DEPLOYMENT: &str = "prompt_gen.needs_deployment";
    pub const PROMPT_GEN_CUSTOM_VALIDATION: &str = "prompt_gen.custom_validation";
    pub const PROMPT_GEN_YAML_PROMPT_GENERATED: &str = "prompt_gen.yaml_prompt_generated";
    #[allow(dead_code)] // 預留給未來剪貼簿功能
    pub const PROMPT_GEN_YAML_PROMPT_COPIED: &str = "prompt_gen.yaml_prompt_copied";
    pub const PROMPT_GEN_INPUT_OUTPUT_FILE: &str = "prompt_gen.input_output_file";

    // Container Builder
    pub const MENU_CONTAINER_BUILDER: &str = "menu.container_builder.name";
    pub const MENU_CONTAINER_BUILDER_DESC: &str = "menu.container_builder.desc";
    pub const CONTAINER_BUILDER_HEADER: &str = "container_builder.header";
    pub const CONTAINER_BUILDER_CURRENT_DIR_FAILED: &str = "container_builder.current_dir_failed";
    pub const CONTAINER_BUILDER_CANCELLED: &str = "container_builder.cancelled";
    pub const CONTAINER_BUILDER_SELECT_ENGINE: &str = "container_builder.select_engine";
    pub const CONTAINER_BUILDER_ENGINE_DOCKER_DESC: &str = "container_builder.engine_docker_desc";
    pub const CONTAINER_BUILDER_ENGINE_BUILDAH_DESC: &str = "container_builder.engine_buildah_desc";
    pub const CONTAINER_BUILDER_ENGINE_NOT_FOUND: &str = "container_builder.engine_not_found";
    pub const CONTAINER_BUILDER_USING_ENGINE: &str = "container_builder.using_engine";
    pub const CONTAINER_BUILDER_SCANNING_DOCKERFILES: &str =
        "container_builder.scanning_dockerfiles";
    pub const CONTAINER_BUILDER_NO_DOCKERFILE: &str = "container_builder.no_dockerfile";
    pub const CONTAINER_BUILDER_FOUND_DOCKERFILES: &str = "container_builder.found_dockerfiles";
    pub const CONTAINER_BUILDER_SELECT_DOCKERFILE: &str = "container_builder.select_dockerfile";
    pub const CONTAINER_BUILDER_SELECTED_DOCKERFILE: &str = "container_builder.selected_dockerfile";
    pub const CONTAINER_BUILDER_SELECT_ARCH: &str = "container_builder.select_arch";
    pub const CONTAINER_BUILDER_SELECTED_ARCH: &str = "container_builder.selected_arch";
    pub const CONTAINER_BUILDER_SELECT_IMAGE_NAME: &str = "container_builder.select_image_name";
    pub const CONTAINER_BUILDER_INPUT_IMAGE_NAME: &str = "container_builder.input_image_name";
    pub const CONTAINER_BUILDER_NEW_IMAGE: &str = "container_builder.new_image";
    pub const CONTAINER_BUILDER_SELECT_TAG: &str = "container_builder.select_tag";
    pub const CONTAINER_BUILDER_INPUT_TAG: &str = "container_builder.input_tag";
    pub const CONTAINER_BUILDER_NEW_TAG: &str = "container_builder.new_tag";
    pub const CONTAINER_BUILDER_ASK_PUSH: &str = "container_builder.ask_push";
    pub const CONTAINER_BUILDER_SELECT_REGISTRY: &str = "container_builder.select_registry";
    pub const CONTAINER_BUILDER_INPUT_REGISTRY: &str = "container_builder.input_registry";
    pub const CONTAINER_BUILDER_NEW_REGISTRY: &str = "container_builder.new_registry";
    pub const CONTAINER_BUILDER_BUILD_SUMMARY: &str = "container_builder.build_summary";
    pub const CONTAINER_BUILDER_CONFIRM_BUILD: &str = "container_builder.confirm_build";
    pub const CONTAINER_BUILDER_BUILDING: &str = "container_builder.building";
    pub const CONTAINER_BUILDER_BUILD_SUCCESS: &str = "container_builder.build_success";
    pub const CONTAINER_BUILDER_BUILD_FAILED: &str = "container_builder.build_failed";

    // Settings
    pub const SETTINGS_COMMON_COUNT_NAME: &str = "settings.common_count.name";
    pub const SETTINGS_COMMON_COUNT_DESC: &str = "settings.common_count.desc";
    pub const SETTINGS_COMMON_COUNT_PROMPT: &str = "settings.common_count.prompt";
    pub const SETTINGS_COMMON_COUNT_SAVED: &str = "settings.common_count.saved";
    pub const SETTINGS_MENU_PROMPT: &str = "settings.menu.prompt";
    pub const CONTAINER_BUILDER_BUILD_ERROR: &str = "container_builder.build_error";
    pub const CONTAINER_BUILDER_PUSHING: &str = "container_builder.pushing";
    pub const CONTAINER_BUILDER_PUSH_SUCCESS: &str = "container_builder.push_success";
    pub const CONTAINER_BUILDER_PUSH_FAILED: &str = "container_builder.push_failed";
    pub const CONTAINER_BUILDER_PUSH_ERROR: &str = "container_builder.push_error";

    // Skill Installer - Menu
    pub const MENU_SKILL_INSTALLER: &str = "menu.skill_installer.name";
    pub const MENU_SKILL_INSTALLER_DESC: &str = "menu.skill_installer.desc";

    // Skill Installer - UI
    pub const SKILL_INSTALLER_HEADER: &str = "skill_installer.header";
    pub const SKILL_INSTALLER_SELECT_CLI: &str = "skill_installer.select_cli";
    pub const SKILL_INSTALLER_CANCELLED: &str = "skill_installer.cancelled";
    pub const SKILL_INSTALLER_USING_CLI: &str = "skill_installer.using_cli";
    pub const SKILL_INSTALLER_SCANNING: &str = "skill_installer.scanning";
    pub const SKILL_INSTALLER_NONE_INSTALLED: &str = "skill_installer.none_installed";
    pub const SKILL_INSTALLER_FOUND_INSTALLED: &str = "skill_installer.found_installed";
    pub const SKILL_INSTALLER_STATUS_INSTALLED: &str = "skill_installer.status_installed";
    pub const SKILL_INSTALLER_STATUS_MISSING: &str = "skill_installer.status_missing";
    pub const SKILL_INSTALLER_SELECT_PROMPT: &str = "skill_installer.select_prompt";
    pub const SKILL_INSTALLER_SELECT_HELP: &str = "skill_installer.select_help";
    pub const SKILL_INSTALLER_NO_CHANGES: &str = "skill_installer.no_changes";
    pub const SKILL_INSTALLER_NO_EXTENSIONS: &str = "skill_installer.no_extensions";
    pub const SKILL_INSTALLER_CHANGE_SUMMARY: &str = "skill_installer.change_summary";
    pub const SKILL_INSTALLER_WILL_INSTALL: &str = "skill_installer.will_install";
    pub const SKILL_INSTALLER_WILL_REMOVE: &str = "skill_installer.will_remove";
    pub const SKILL_INSTALLER_CONFIRM_CHANGES: &str = "skill_installer.confirm_changes";
    pub const SKILL_INSTALLER_DOWNLOADING: &str = "skill_installer.downloading";
    pub const SKILL_INSTALLER_INSTALL_SUCCESS: &str = "skill_installer.install_success";
    pub const SKILL_INSTALLER_INSTALL_FAILED: &str = "skill_installer.install_failed";
    pub const SKILL_INSTALLER_REMOVING: &str = "skill_installer.removing";
    pub const SKILL_INSTALLER_REMOVE_SUCCESS: &str = "skill_installer.remove_success";
    pub const SKILL_INSTALLER_REMOVE_FAILED: &str = "skill_installer.remove_failed";
    pub const SKILL_INSTALLER_SUMMARY: &str = "skill_installer.summary";
    pub const SKILL_INSTALLER_DOWNLOAD_FAILED: &str = "skill_installer.download_failed";
    pub const SKILL_INSTALLER_EXTRACT_FAILED: &str = "skill_installer.extract_failed";

    // Extension names
    pub const SKILL_RALPH_WIGGUM: &str = "skill.ralph_wiggum";
    pub const SKILL_FRONTEND_DESIGN: &str = "skill.frontend_design";
    pub const SKILL_CODE_REVIEW: &str = "skill.code_review";
    pub const SKILL_PR_REVIEW_TOOLKIT: &str = "skill.pr_review_toolkit";
    pub const SKILL_COMMIT_COMMANDS: &str = "skill.commit_commands";
    pub const SKILL_SECURITY_GUIDANCE: &str = "skill.security_guidance";
    pub const SKILL_WRITING_RULES: &str = "skill.writing_rules";
    pub const SKILL_CLAUDE_MEM: &str = "skill.claude_mem";
}

#[cfg(test)]
pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::Mutex;
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("Language test lock poisoned")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn locales_share_keys() {
        let _guard = test_lock();
        let bundle = bundle();
        let reference = bundle
            .maps
            .get(&Language::English)
            .expect("Missing English locale");
        let reference_keys: HashSet<&String> = reference.keys().collect();

        for language in [
            Language::TraditionalChinese,
            Language::SimplifiedChinese,
            Language::Japanese,
        ] {
            let locale = bundle.maps.get(&language).expect("Missing locale data");
            let locale_keys: HashSet<&String> = locale.keys().collect();
            assert_eq!(
                locale_keys, reference_keys,
                "Locale {:?} does not match English keys",
                language
            );
        }
    }

    #[test]
    fn set_language_updates_translation() {
        let _guard = test_lock();
        let previous = current_language();

        set_language(Language::English);
        assert_eq!(t(keys::MENU_EXIT), "Exit");

        set_language(Language::TraditionalChinese);
        assert_eq!(t(keys::MENU_EXIT), "退出");

        set_language(previous);
    }

    #[test]
    fn unknown_key_returns_placeholder() {
        let _guard = test_lock();
        assert_eq!(t("missing.key"), "??");
    }
}
