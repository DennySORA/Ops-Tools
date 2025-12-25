use super::tools::{CliType, McpTool};
use crate::core::{OperationError, Result};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
        self.maybe_migrate_gemini_settings()?;
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
        self.maybe_migrate_gemini_settings()?;
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
            self.maybe_migrate_gemini_settings()?;
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
        self.maybe_migrate_gemini_settings()?;
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

    fn maybe_migrate_gemini_settings(&self) -> Result<()> {
        if self.cli != CliType::Gemini {
            return Ok(());
        }

        for path in gemini_settings_paths() {
            if !path.exists() {
                continue;
            }
            migrate_gemini_settings_file(&path)?;
        }

        Ok(())
    }
}

/// 解析 mcp list 的輸出
fn parse_mcp_list(output: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in output.lines() {
        let stripped = strip_ansi_codes(line);
        let trimmed = stripped.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("mcp ")
            || lower.starts_with("mcp servers")
            || lower.starts_with("configured mcp")
            || lower.starts_with("---")
        {
            continue;
        }
        if lower.starts_with("name") && (lower.contains("status") || lower.contains("command")) {
            continue;
        }

        for token in trimmed.split_whitespace() {
            let clean_name =
                token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
            if clean_name.is_empty() {
                continue;
            }

            let clean_lower = clean_name.to_ascii_lowercase();
            if is_ignored_token(&clean_lower) {
                continue;
            }

            if !names.iter().any(|name| name == clean_name) {
                names.push(clean_name.to_string());
            }
            break;
        }
    }

    names
}

fn gemini_settings_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join(".gemini").join("settings.json"));
    }

    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(home);
        paths.push(home_path.join(".gemini").join("settings.json"));
        paths.push(home_path.join(".config").join("gemini").join("settings.json"));
        paths.push(home_path.join(".config").join("gemini-cli").join("settings.json"));
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        let xdg_path = PathBuf::from(xdg);
        paths.push(xdg_path.join("gemini").join("settings.json"));
        paths.push(xdg_path.join("gemini-cli").join("settings.json"));
    }

    let mut unique = Vec::new();
    for path in paths {
        if !unique.contains(&path) {
            unique.push(path);
        }
    }

    unique
}

fn migrate_gemini_settings_file(path: &Path) -> Result<bool> {
    let raw = fs::read_to_string(path).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;

    if !raw.contains("\"type\"") {
        return Ok(false);
    }

    let sanitized = strip_json_comments(&raw);
    let mut root: Value = serde_json::from_str(&sanitized).map_err(|err| {
        OperationError::Config {
            key: path.display().to_string(),
            message: format!("設定檔解析失敗: {}", err),
        }
    })?;

    let changed = migrate_gemini_mcp_servers(&mut root);
    if changed {
        let formatted = serde_json::to_string_pretty(&root).map_err(|err| {
            OperationError::Config {
                key: path.display().to_string(),
                message: format!("設定檔序列化失敗: {}", err),
            }
        })?;
        fs::write(path, format!("{}\n", formatted)).map_err(|err| OperationError::Io {
            path: path.display().to_string(),
            source: err,
        })?;
    }

    Ok(changed)
}

fn migrate_gemini_mcp_servers(root: &mut Value) -> bool {
    let Some(servers) = root
        .get_mut("mcpServers")
        .and_then(|value| value.as_object_mut())
    else {
        return false;
    };

    let mut changed = false;

    for server in servers.values_mut() {
        let Some(server_obj) = server.as_object_mut() else {
            continue;
        };

        let transport = match server_obj.remove("type") {
            Some(value) => {
                changed = true;
                value.as_str().unwrap_or("").to_ascii_lowercase()
            }
            None => continue,
        };

        if transport == "http" {
            if server_obj.get("httpUrl").is_none() {
                if let Some(url_value) = server_obj.remove("url") {
                    server_obj.insert("httpUrl".to_string(), url_value);
                    changed = true;
                }
            }
            if server_obj.get("httpUrl").is_some() {
                server_obj.remove("url");
            }
        }
    }

    changed
}

fn is_ignored_token(token: &str) -> bool {
    matches!(
        token,
        "mcp"
            | "server"
            | "servers"
            | "name"
            | "status"
            | "command"
            | "configured"
            | "enabled"
            | "disabled"
            | "running"
            | "stopped"
            | "connected"
    )
}

fn strip_ansi_codes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek().copied() == Some('[') {
                chars.next();
                while let Some(code_ch) = chars.next() {
                    if code_ch.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        output.push(ch);
    }

    output
}

fn strip_json_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            output.push(ch);
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\n' {
                            output.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '*' && matches!(chars.peek(), Some('/')) {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }

        output.push(ch);
    }

    output
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

    #[test]
    fn test_parse_mcp_list_with_checkmark_prefix() {
        let output = concat!(
            "Configured MCP servers:\n",
            "\u{2713} sequential-thinking: npx -y tool (stdio) - Connected"
        );
        let result = parse_mcp_list(output);
        assert_eq!(result, vec!["sequential-thinking".to_string()]);
    }

    #[test]
    fn test_parse_mcp_list_with_ansi_colors() {
        let output = concat!(
            "Configured MCP servers:\n",
            "\u{1b}[32m\u{2713}\u{1b}[0m sequential-thinking: npx -y tool (stdio) - Connected"
        );
        let result = parse_mcp_list(output);
        assert_eq!(result, vec!["sequential-thinking".to_string()]);
    }

    #[test]
    fn test_migrate_gemini_settings_http_type() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let content = r#"{"mcpServers":{"context7":{"url":"https://example.com","type":"http"}}}"#;

        fs::write(&path, content).unwrap();

        let changed = migrate_gemini_settings_file(&path).unwrap();
        assert!(changed);

        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let server = &value["mcpServers"]["context7"];
        assert_eq!(server["httpUrl"], "https://example.com");
        assert!(server.get("url").is_none());
        assert!(server.get("type").is_none());
    }

    #[test]
    fn test_migrate_gemini_settings_sse_type() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let content = r#"{"mcpServers":{"context7":{"url":"https://example.com","type":"sse"}}}"#;

        fs::write(&path, content).unwrap();

        let changed = migrate_gemini_settings_file(&path).unwrap();
        assert!(changed);

        let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let server = &value["mcpServers"]["context7"];
        assert_eq!(server["url"], "https://example.com");
        assert!(server.get("httpUrl").is_none());
        assert!(server.get("type").is_none());
    }
}
