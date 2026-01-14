//! 套件管理器操作
//!
//! 此模組為公開 API，統一匯出所有套件管理功能

use crate::core::Result;

// 匯入子模組
use super::installers;
use super::shell;
use super::types;

// 重新匯出公開型別
pub use types::{
    package_definitions, ActionContext, PackageAction, PackageDefinition, PackageId, SupportedOs,
};

// 重新匯出 shell 工具
pub use shell::is_command_available;

// ============================================================================
// 公開 API
// ============================================================================

/// 確保 curl 已安裝
pub fn ensure_curl(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("curl").is_some() {
        return Ok(());
    }
    shell::install_with_manager(ctx, "curl")
}

/// 更新 curl
pub fn update_curl(ctx: &mut ActionContext) -> Result<()> {
    if is_command_available("curl").is_none() {
        return ensure_curl(ctx);
    }
    shell::update_with_manager(ctx, "curl")
}

/// 檢查套件是否已安裝
pub fn is_installed(package: PackageId, ctx: &ActionContext) -> bool {
    installers::is_installed(package, ctx)
}

/// 執行套件操作（安裝/更新/移除）
pub fn apply_action(
    action: PackageAction,
    package: PackageId,
    ctx: &mut ActionContext,
) -> Result<()> {
    match action {
        PackageAction::Install => installers::install_package(package, ctx),
        PackageAction::Update => installers::update_package(package, ctx),
        PackageAction::Remove => installers::remove_package(package, ctx),
    }
}
