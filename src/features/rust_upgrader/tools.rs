/// Cargo 工具套件定義
#[derive(Debug, Clone)]
pub struct CargoTool {
    pub crate_name: &'static str,
    pub display_name: &'static str,
    pub command: &'static str,
}

impl CargoTool {
    pub const fn new(
        crate_name: &'static str,
        display_name: &'static str,
        command: &'static str,
    ) -> Self {
        Self {
            crate_name,
            display_name,
            command,
        }
    }
}

/// 必要的 Cargo 工具套件清單
pub const REQUIRED_CARGO_TOOLS: &[CargoTool] = &[
    CargoTool::new("cargo-edit", "Cargo Edit", "upgrade"),
    CargoTool::new("cargo-update", "Cargo Update", "install-update"),
    CargoTool::new("cargo-outdated", "Cargo Outdated", "outdated"),
    CargoTool::new("cargo-audit", "Cargo Audit", "audit"),
];

/// 升級步驟定義
#[derive(Debug, Clone)]
pub struct UpgradeStep {
    pub name: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub description: &'static str,
    pub requires_project: bool,
}

impl UpgradeStep {
    pub const fn new(
        name: &'static str,
        command: &'static str,
        args: &'static [&'static str],
        description: &'static str,
        requires_project: bool,
    ) -> Self {
        Self {
            name,
            command,
            args,
            description,
            requires_project,
        }
    }
}

/// 升級步驟清單
pub const UPGRADE_STEPS: &[UpgradeStep] = &[
    UpgradeStep::new(
        "Rustup Self Update",
        "rustup",
        &["self", "update"],
        "更新 rustup 本身",
        false,
    ),
    UpgradeStep::new(
        "Rustup Update",
        "rustup",
        &["update"],
        "更新 Rust 工具鏈",
        false,
    ),
    UpgradeStep::new(
        "Cargo Install Update",
        "cargo",
        &["install-update", "-a"],
        "升級所有已安裝的 cargo crates",
        false,
    ),
    UpgradeStep::new(
        "Cargo Upgrade",
        "cargo",
        &["upgrade", "--incompatible"],
        "升級專案依賴（包含破壞性更新）",
        true,
    ),
    UpgradeStep::new(
        "Cargo Outdated",
        "cargo",
        &["outdated"],
        "檢查過時的依賴",
        true,
    ),
    UpgradeStep::new("Cargo Audit", "cargo", &["audit"], "安全性漏洞掃描", true),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_required_tools_not_empty() {
        assert!(!REQUIRED_CARGO_TOOLS.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_upgrade_steps_not_empty() {
        assert!(!UPGRADE_STEPS.is_empty());
    }

    #[test]
    fn test_cargo_tool_fields() {
        let tool = &REQUIRED_CARGO_TOOLS[0];
        assert!(!tool.crate_name.is_empty());
        assert!(!tool.display_name.is_empty());
        assert!(!tool.command.is_empty());
    }

    #[test]
    fn test_upgrade_step_fields() {
        let step = &UPGRADE_STEPS[0];
        assert!(!step.name.is_empty());
        assert!(!step.command.is_empty());
        assert!(!step.description.is_empty());
    }
}
