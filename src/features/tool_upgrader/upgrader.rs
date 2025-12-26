use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use std::process::Command;

/// 套件升級器
pub struct PackageUpgrader {
    package_manager: String,
}

impl PackageUpgrader {
    pub fn new() -> Self {
        Self {
            package_manager: "pnpm".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_package_manager(package_manager: &str) -> Self {
        Self {
            package_manager: package_manager.to_string(),
        }
    }

    /// 升級指定套件到最新版本
    pub fn upgrade(&self, package: &str) -> Result<String> {
        let full_package = format!("{}@latest", package);

        let output = Command::new(&self.package_manager)
            .args(["add", "-g", &full_package])
            .output()
            .map_err(|e| OperationError::Command {
                command: self.package_manager.clone(),
                message: crate::tr!(keys::ERROR_UNABLE_TO_EXECUTE, error = e),
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(OperationError::Command {
                command: format!("{} add -g {}", self.package_manager, full_package),
                message: stderr
                    .lines()
                    .next()
                    .unwrap_or(i18n::t(keys::ERROR_UNKNOWN))
                    .to_string(),
            })
        }
    }
}

impl Default for PackageUpgrader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrader_creation() {
        let upgrader = PackageUpgrader::new();
        assert_eq!(upgrader.package_manager, "pnpm");
    }

    #[test]
    fn test_custom_package_manager() {
        let upgrader = PackageUpgrader::with_package_manager("npm");
        assert_eq!(upgrader.package_manager, "npm");
    }
}
