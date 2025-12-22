use super::tools::{CliType, McpTool};
use crate::core::{OperationError, Result};
use std::process::Command;

/// MCP CLI 執行器
pub struct McpExecutor {
    cli: CliType,
}

impl McpExecutor {
    pub fn new(cli: CliType) -> Self {
        Self { cli }
    }

    /// 取得已安裝的 MCP 清單
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let output = Command::new(self.cli.command())
            .args(["mcp", "list"])
            .output()
            .map_err(|e| OperationError::Command {
                command: self.cli.command().to_string(),
                message: format!("無法執行: {}", e),
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(parse_mcp_list(&stdout))
        } else {
            Ok(Vec::new())
        }
    }

    /// 安裝 MCP
    pub fn install(&self, tool: &McpTool) -> Result<()> {
        let mut args: Vec<&str> = vec!["mcp", "add"];
        let string_refs: Vec<&str> = tool.install_args.iter().map(|s| s.as_str()).collect();
        args.extend(string_refs);

        let output = Command::new(self.cli.command())
            .args(&args)
            .output()
            .map_err(|e| OperationError::Command {
                command: self.cli.command().to_string(),
                message: format!("無法執行: {}", e),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(OperationError::Command {
                command: format!("{} mcp add", self.cli.command()),
                message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
            })
        }
    }

    /// 移除 MCP
    pub fn remove(&self, name: &str) -> Result<()> {
        let output = Command::new(self.cli.command())
            .args(["mcp", "remove", name])
            .output()
            .map_err(|e| OperationError::Command {
                command: self.cli.command().to_string(),
                message: format!("無法執行: {}", e),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(OperationError::Command {
                command: format!("{} mcp remove", self.cli.command()),
                message: stderr.lines().next().unwrap_or("未知錯誤").to_string(),
            })
        }
    }
}

/// 解析 mcp list 的輸出
fn parse_mcp_list(output: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("MCP") || trimmed.starts_with("---") {
            continue;
        }
        if let Some(name) = trimmed.split_whitespace().next() {
            let clean_name =
                name.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
            if !clean_name.is_empty() {
                names.push(clean_name.to_string());
            }
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mcp_list_empty() {
        let output = "";
        let result = parse_mcp_list(output);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_mcp_list_with_names() {
        let output = "MCP Servers\n---\nsequential-thinking  running\ncontext7  running";
        let result = parse_mcp_list(output);
        assert!(result.contains(&"sequential-thinking".to_string()));
        assert!(result.contains(&"context7".to_string()));
    }
}
