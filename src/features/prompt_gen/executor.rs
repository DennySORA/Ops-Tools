//! AI CLI 執行器
//!
//! 負責呼叫 Claude/Codex/Gemini CLI 並處理輸出

use anyhow::{bail, Context, Result};
use chrono::Local;
use console::style;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use super::progress::Step;

// ============================================================================
// CLI 類型
// ============================================================================

/// 支援的 CLI 類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CliType {
    /// Claude Code CLI
    #[default]
    Claude,
    /// OpenAI Codex CLI
    Codex,
    /// Google Gemini CLI
    Gemini,
}

impl CliType {
    /// 所有可用的 CLI 類型
    pub const ALL: [CliType; 3] = [CliType::Claude, CliType::Codex, CliType::Gemini];

    /// 取得 CLI 二進位檔案名稱
    pub fn binary_name(&self) -> &'static str {
        match self {
            CliType::Claude => "claude",
            CliType::Codex => "codex",
            CliType::Gemini => "gemini",
        }
    }

    /// 取得顯示名稱
    pub fn display_name(&self) -> &'static str {
        match self {
            CliType::Claude => "Claude Code",
            CliType::Codex => "OpenAI Codex",
            CliType::Gemini => "Google Gemini",
        }
    }

    /// 從索引取得 CLI 類型
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(CliType::Claude),
            1 => Some(CliType::Codex),
            2 => Some(CliType::Gemini),
            _ => None,
        }
    }
}

// ============================================================================
// 執行器配置
// ============================================================================

/// AI CLI 執行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// CLI 類型
    pub cli_type: CliType,
    /// 是否跳過權限提示
    pub skip_permissions: bool,
    /// 輸出格式
    pub output_format: OutputFormat,
    /// 是否自動繼續執行下一步（不詢問確認）
    pub auto_continue: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            cli_type: CliType::Claude,
            skip_permissions: true,
            output_format: OutputFormat::StreamJson,
            auto_continue: false,
        }
    }
}

/// 輸出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// 串流 JSON 格式
    StreamJson,
    /// 純文字格式
    #[allow(dead_code)]
    Text,
}

impl OutputFormat {
    pub fn as_arg(&self) -> &'static str {
        match self {
            Self::StreamJson => "stream-json",
            Self::Text => "text",
        }
    }
}

// ============================================================================
// 執行結果
// ============================================================================

/// 步驟執行結果
#[derive(Debug, Clone)]
pub struct StepResult {
    /// 是否成功
    pub success: bool,
    /// Session ID（從輸出中提取）
    pub session_id: Option<String>,
}

// ============================================================================
// 執行器
// ============================================================================

/// AI CLI 執行器
pub struct Executor {
    config: ExecutorConfig,
}

impl Executor {
    /// 建立新的執行器
    pub fn new(config: ExecutorConfig) -> Self {
        Self { config }
    }

    /// 取得當前 CLI 類型
    pub fn cli_type(&self) -> CliType {
        self.config.cli_type
    }

    /// 檢查 CLI 是否可用
    pub fn check_availability(&self) -> Result<()> {
        let bin = self.config.cli_type.binary_name();
        let output = Command::new(bin)
            .arg("--version")
            .output()
            .with_context(|| {
                format!(
                    "無法找到 {} CLI: {}",
                    self.config.cli_type.display_name(),
                    bin
                )
            })?;

        if !output.status.success() {
            bail!("{} CLI 執行失敗", self.config.cli_type.display_name());
        }

        Ok(())
    }

    /// 執行單一步驟
    pub fn run_step(
        &self,
        feature_key: &str,
        step: Step,
        prompt_path: &Path,
        logs_dir: &Path,
        resume_session: Option<&str>,
    ) -> Result<StepResult> {
        if !prompt_path.exists() {
            bail!("提示檔案不存在：{}", prompt_path.display());
        }

        // 建立日誌目錄
        std::fs::create_dir_all(logs_dir)?;

        // 生成時間戳記
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let step_name = step.as_str();

        let raw_log_file = logs_dir.join(format!("{}_{}_raw.jsonl", timestamp, step_name));
        let stderr_log_file = logs_dir.join(format!("{}_{}_stderr.log", timestamp, step_name));

        // 讀取提示內容
        let prompt_content = std::fs::read_to_string(prompt_path)
            .with_context(|| format!("無法讀取提示檔案：{}", prompt_path.display()))?;

        // 建構命令（根據 CLI 類型）
        let bin = self.config.cli_type.binary_name();
        let mut cmd = Command::new(bin);

        match self.config.cli_type {
            CliType::Claude => {
                cmd.arg("-p")
                    .arg("--output-format")
                    .arg(self.config.output_format.as_arg())
                    .arg("--verbose");

                if self.config.skip_permissions {
                    cmd.arg("--dangerously-skip-permissions");
                }

                if let Some(session_id) = resume_session {
                    cmd.arg("--resume").arg(session_id);
                }
            }
            CliType::Codex => {
                // Codex CLI 使用 --full-auto 模式
                cmd.arg("--full-auto").arg("--quiet");

                if let Some(session_id) = resume_session {
                    cmd.arg("--resume").arg(session_id);
                }
            }
            CliType::Gemini => {
                // Gemini CLI 使用 --non-interactive 模式
                cmd.arg("--non-interactive");

                if let Some(session_id) = resume_session {
                    cmd.arg("--resume").arg(session_id);
                }
            }
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 啟動程序
        let cli_name = self.config.cli_type.display_name();
        let mut child = cmd
            .spawn()
            .with_context(|| format!("無法啟動 {} CLI: {}", cli_name, bin))?;

        // 寫入提示到 stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt_content.as_bytes())?;
        }

        // 開啟日誌檔案
        let mut raw_log = std::fs::File::create(&raw_log_file)?;
        let mut stderr_log = std::fs::File::create(&stderr_log_file)?;

        // 處理 stdout（串流輸出）
        let stdout = child.stdout.take().expect("stdout should be piped");
        let reader = BufReader::new(stdout);

        let mut session_id = None;

        println!(
            "\n{} 開始執行步驟 {} for {}",
            style("[執行]").cyan().bold(),
            style(step.as_str()).yellow(),
            style(feature_key).green()
        );
        println!("{}", style("─".repeat(60)).dim());

        for line in reader.lines() {
            let line = line?;

            // 寫入原始日誌
            writeln!(raw_log, "{}", line)?;

            // 解析並顯示
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                // 嘗試提取 session_id
                if let Some(sid) = json.get("session_id").and_then(|v| v.as_str()) {
                    session_id = Some(sid.to_string());
                }

                // 提取並顯示文字內容
                self.display_json_content(&json);
            }
        }

        println!("{}", style("─".repeat(60)).dim());

        // 處理 stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                writeln!(stderr_log, "{}", line)?;
                if !line.is_empty() {
                    eprintln!("{} {}", style("[stderr]").red(), line);
                }
            }
        }

        // 等待程序結束
        let status = child.wait()?;

        let success = status.success();

        if success {
            println!(
                "{} 步驟 {} 完成",
                style("[完成]").green().bold(),
                style(step.as_str()).yellow()
            );
        } else {
            println!(
                "{} 步驟 {} 失敗 (exit code: {})",
                style("[失敗]").red().bold(),
                style(step.as_str()).yellow(),
                status.code().unwrap_or(-1)
            );
        }

        Ok(StepResult {
            success,
            session_id,
        })
    }

    /// 從 JSON 輸出中提取並顯示文字內容
    fn display_json_content(&self, json: &serde_json::Value) {
        // 處理 delta 格式
        if let Some(delta) = json.get("delta") {
            if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                print!("{}", text);
                let _ = std::io::stdout().flush();
                return;
            }
        }

        // 處理 message 格式
        if let Some(message) = json.get("message") {
            if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                for item in content {
                    if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                        if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                            print!("{}", text);
                            let _ = std::io::stdout().flush();
                        }
                    }
                }
            }
        }

        // 處理 result 類型（包含最終 session_id）
        if json.get("type").and_then(|v| v.as_str()) == Some("result") {
            println!();
            if let Some(stats) = json.get("stats") {
                if let Some(tokens) = stats.get("total_tokens").and_then(|v| v.as_i64()) {
                    println!("{} Total tokens: {}", style("[統計]").blue(), tokens);
                }
            }
        }
    }
}

// ============================================================================
// 測試
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format() {
        assert_eq!(OutputFormat::StreamJson.as_arg(), "stream-json");
        assert_eq!(OutputFormat::Text.as_arg(), "text");
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.cli_type, CliType::Claude);
        assert!(config.skip_permissions);
    }

    #[test]
    fn test_cli_type_binary_names() {
        assert_eq!(CliType::Claude.binary_name(), "claude");
        assert_eq!(CliType::Codex.binary_name(), "codex");
        assert_eq!(CliType::Gemini.binary_name(), "gemini");
    }

    #[test]
    fn test_cli_type_display_names() {
        assert_eq!(CliType::Claude.display_name(), "Claude Code");
        assert_eq!(CliType::Codex.display_name(), "OpenAI Codex");
        assert_eq!(CliType::Gemini.display_name(), "Google Gemini");
    }
}
