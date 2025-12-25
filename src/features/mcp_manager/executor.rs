use super::config::ENV_CONFIG;
use super::tools::{cloudflare_tool_names, CliType, McpTool};
use crate::core::{OperationError, Result};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value as TomlValue;

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
        self.maybe_migrate_cli_settings()?;
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
        self.maybe_migrate_cli_settings()?;
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
            self.maybe_migrate_cli_settings()?;
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
        self.maybe_migrate_cli_settings()?;
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

    fn maybe_migrate_cli_settings(&self) -> Result<()> {
        self.maybe_migrate_gemini_settings()?;
        self.maybe_configure_codex_context7_headers()?;
        self.maybe_configure_codex_github_env()?;
        self.maybe_configure_codex_cloudflare_headers()?;
        Ok(())
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

    fn maybe_configure_codex_context7_headers(&self) -> Result<()> {
        if self.cli != CliType::Codex {
            return Ok(());
        }

        let Some(key) = ENV_CONFIG.context7_api_key else {
            return Ok(());
        };

        let Some(path) = codex_config_path() else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }

        // Codex CLI 不支援 --header，改寫設定檔的 http_headers。
        update_codex_context7_config(&path, key)?;
        Ok(())
    }

    fn maybe_configure_codex_github_env(&self) -> Result<()> {
        if self.cli != CliType::Codex {
            return Ok(());
        }

        let (Some(token), Some(host)) = (ENV_CONFIG.github_token, ENV_CONFIG.github_host) else {
            return Ok(());
        };

        let Some(path) = codex_config_path() else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }

        // Codex CLI 將 stdio MCP 的 env 寫入設定檔以避免執行期環境變數。
        update_codex_github_config(&path, token, host)?;
        Ok(())
    }

    fn maybe_configure_codex_cloudflare_headers(&self) -> Result<()> {
        if self.cli != CliType::Codex {
            return Ok(());
        }

        if !ENV_CONFIG.enable_cloudflare_mcp() {
            return Ok(());
        }

        let Some(token) = ENV_CONFIG.cloudflare_api_token else {
            return Ok(());
        };

        let Some(path) = codex_config_path() else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }

        update_codex_cloudflare_config(&path, token, &cloudflare_tool_names())?;
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

fn codex_config_path() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".codex").join("config.toml"))
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

fn update_codex_context7_config(path: &Path, api_key: &str) -> Result<bool> {
    let raw = fs::read_to_string(path).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;

    let mut root: toml::Table = toml::from_str(&raw).map_err(|err| OperationError::Config {
        key: path.display().to_string(),
        message: format!("設定檔解析失敗: {}", err),
    })?;

    let Some(servers) = root
        .get_mut("mcp_servers")
        .and_then(|value| value.as_table_mut())
    else {
        return Ok(false);
    };

    let Some(context7) = servers.get_mut("context7").and_then(|value| value.as_table_mut()) else {
        return Ok(false);
    };

    let mut changed = false;
    let mut headers = match context7.get("http_headers") {
        Some(value) => match value.as_table() {
            Some(table) => table.clone(),
            None => {
                changed = true;
                toml::map::Map::new()
            }
        },
        None => {
            changed = true;
            toml::map::Map::new()
        }
    };

    if headers
        .get("CONTEXT7_API_KEY")
        .and_then(|value| value.as_str())
        != Some(api_key)
    {
        headers.insert(
            "CONTEXT7_API_KEY".to_string(),
            TomlValue::String(api_key.to_string()),
        );
        changed = true;
    }

    if context7.remove("bearer_token_env_var").is_some() {
        changed = true;
    }
    if context7.remove("env_http_headers").is_some() {
        changed = true;
    }

    if changed {
        context7.insert("http_headers".to_string(), TomlValue::Table(headers));
        let formatted = toml::to_string(&root).map_err(|err| OperationError::Config {
            key: path.display().to_string(),
            message: format!("設定檔序列化失敗: {}", err),
        })?;
        fs::write(path, format!("{}\n", formatted)).map_err(|err| OperationError::Io {
            path: path.display().to_string(),
            source: err,
        })?;
    }

    Ok(changed)
}

fn update_codex_github_config(path: &Path, token: &str, host: &str) -> Result<bool> {
    let raw = fs::read_to_string(path).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;

    let mut root: toml::Table = toml::from_str(&raw).map_err(|err| OperationError::Config {
        key: path.display().to_string(),
        message: format!("設定檔解析失敗: {}", err),
    })?;

    let Some(servers) = root
        .get_mut("mcp_servers")
        .and_then(|value| value.as_table_mut())
    else {
        return Ok(false);
    };

    let Some(github) = servers.get_mut("github").and_then(|value| value.as_table_mut()) else {
        return Ok(false);
    };

    let mut changed = false;
    let mut env_map = match github.get("env") {
        Some(value) => match value.as_table() {
            Some(table) => table.clone(),
            None => {
                changed = true;
                toml::map::Map::new()
            }
        },
        None => {
            changed = true;
            toml::map::Map::new()
        }
    };

    let updates = [
        ("GITHUB_PERSONAL_ACCESS_TOKEN", token),
        ("GITHUB_HOST", host),
    ];
    for (key, value) in updates {
        if env_map.get(key).and_then(|val| val.as_str()) != Some(value) {
            env_map.insert(key.to_string(), TomlValue::String(value.to_string()));
            changed = true;
        }
    }

    if let Some(env_vars) = github.get_mut("env_vars").and_then(|val| val.as_array_mut()) {
        let before = env_vars.len();
        env_vars.retain(|item| {
            item.as_str()
                .map(|name| name != "GITHUB_PERSONAL_ACCESS_TOKEN" && name != "GITHUB_HOST")
                .unwrap_or(true)
        });
        if env_vars.len() != before {
            changed = true;
        }
    }

    if changed {
        github.insert("env".to_string(), TomlValue::Table(env_map));
        let formatted = toml::to_string(&root).map_err(|err| OperationError::Config {
            key: path.display().to_string(),
            message: format!("設定檔序列化失敗: {}", err),
        })?;
        fs::write(path, format!("{}\n", formatted)).map_err(|err| OperationError::Io {
            path: path.display().to_string(),
            source: err,
        })?;
    }

    Ok(changed)
}

fn update_codex_cloudflare_config(
    path: &Path,
    api_token: &str,
    tool_names: &[&'static str],
) -> Result<bool> {
    let raw = fs::read_to_string(path).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })?;

    let mut root: toml::Table = toml::from_str(&raw).map_err(|err| OperationError::Config {
        key: path.display().to_string(),
        message: format!("設定檔解析失敗: {}", err),
    })?;

    let Some(servers) = root
        .get_mut("mcp_servers")
        .and_then(|value| value.as_table_mut())
    else {
        return Ok(false);
    };

    let mut changed = false;
    let auth_value = format!("Bearer {}", api_token);

    for name in tool_names {
        let Some(server) = servers.get_mut(*name).and_then(|value| value.as_table_mut()) else {
            continue;
        };

        let mut headers = match server.get("http_headers") {
            Some(value) => match value.as_table() {
                Some(table) => table.clone(),
                None => {
                    changed = true;
                    toml::map::Map::new()
                }
            },
            None => {
                changed = true;
                toml::map::Map::new()
            }
        };

        if headers
            .get("Authorization")
            .and_then(|value| value.as_str())
            != Some(auth_value.as_str())
        {
            headers.insert(
                "Authorization".to_string(),
                TomlValue::String(auth_value.clone()),
            );
            changed = true;
        }

        if server.remove("bearer_token_env_var").is_some() {
            changed = true;
        }
        if server.remove("env_http_headers").is_some() {
            changed = true;
        }

        server.insert("http_headers".to_string(), TomlValue::Table(headers));
    }

    if changed {
        let formatted = toml::to_string(&root).map_err(|err| OperationError::Config {
            key: path.display().to_string(),
            message: format!("設定檔序列化失敗: {}", err),
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

    #[test]
    fn test_update_codex_context7_config_sets_http_headers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let content = r#"[mcp_servers.context7]
url = "https://mcp.context7.com/mcp"
bearer_token_env_var = "CONTEXT7_API_KEY"
"#;

        fs::write(&path, content).unwrap();

        let changed = update_codex_context7_config(&path, "test-key").unwrap();
        assert!(changed);

        let root: toml::Table =
            toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let servers = root.get("mcp_servers").unwrap().as_table().unwrap();
        let context7 = servers.get("context7").unwrap().as_table().unwrap();
        assert!(context7.get("bearer_token_env_var").is_none());
        let headers = context7.get("http_headers").unwrap().as_table().unwrap();
        assert_eq!(
            headers
                .get("CONTEXT7_API_KEY")
                .and_then(|value| value.as_str()),
            Some("test-key")
        );
    }

    #[test]
    fn test_update_codex_context7_config_missing_context7_no_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let content = r#"[mcp_servers.sequential-thinking]
command = "npx"
"#;

        fs::write(&path, content).unwrap();

        let changed = update_codex_context7_config(&path, "test-key").unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_update_codex_github_config_sets_env() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let content = r#"[mcp_servers.github]
command = "docker"
args = ["run"]
env_vars = ["GITHUB_PERSONAL_ACCESS_TOKEN", "OTHER"]
"#;

        fs::write(&path, content).unwrap();

        let changed = update_codex_github_config(&path, "token-1", "github.com").unwrap();
        assert!(changed);

        let root: toml::Table =
            toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let servers = root.get("mcp_servers").unwrap().as_table().unwrap();
        let github = servers.get("github").unwrap().as_table().unwrap();
        let env = github.get("env").unwrap().as_table().unwrap();
        assert_eq!(
            env.get("GITHUB_PERSONAL_ACCESS_TOKEN")
                .and_then(|value| value.as_str()),
            Some("token-1")
        );
        assert_eq!(
            env.get("GITHUB_HOST").and_then(|value| value.as_str()),
            Some("github.com")
        );
        let env_vars = github.get("env_vars").unwrap().as_array().unwrap();
        assert_eq!(env_vars.len(), 1);
        assert_eq!(env_vars[0].as_str(), Some("OTHER"));
    }

    #[test]
    fn test_update_codex_github_config_missing_github_no_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let content = r#"[mcp_servers.sequential-thinking]
command = "npx"
"#;

        fs::write(&path, content).unwrap();

        let changed = update_codex_github_config(&path, "token-1", "github.com").unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_update_codex_cloudflare_config_sets_authorization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let content = r#"[mcp_servers.cloudflare-docs]
url = "https://docs.mcp.cloudflare.com/mcp"
bearer_token_env_var = "CLOUDFLARE_API_TOKEN"
"#;

        fs::write(&path, content).unwrap();

        let names = vec!["cloudflare-docs"];
        let changed = update_codex_cloudflare_config(&path, "token-123", &names).unwrap();
        assert!(changed);

        let root: toml::Table =
            toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let servers = root.get("mcp_servers").unwrap().as_table().unwrap();
        let server = servers.get("cloudflare-docs").unwrap().as_table().unwrap();
        assert!(server.get("bearer_token_env_var").is_none());
        let headers = server.get("http_headers").unwrap().as_table().unwrap();
        assert_eq!(
            headers
                .get("Authorization")
                .and_then(|value| value.as_str()),
            Some("Bearer token-123")
        );
    }

    #[test]
    fn test_update_codex_cloudflare_config_missing_server_no_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let content = r#"[mcp_servers.sequential-thinking]
command = "npx"
"#;

        fs::write(&path, content).unwrap();

        let names = vec!["cloudflare-docs"];
        let changed = update_codex_cloudflare_config(&path, "token-123", &names).unwrap();
        assert!(!changed);
    }
}
