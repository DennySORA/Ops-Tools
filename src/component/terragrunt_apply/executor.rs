use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

/// Terragrunt 執行器 - 負責執行 terragrunt 指令
pub struct TerragruntExecutor;

impl TerragruntExecutor {
    pub fn new() -> Self {
        Self
    }

    /// 執行 terragrunt apply
    pub fn apply(&self, directory: &Path) -> Result<(), String> {
        println!("\n進入目錄: {}", directory.display());

        // 檢查目錄是否存在
        if !directory.exists() {
            return Err(format!("目錄不存在: {}", directory.display()));
        }

        if !directory.is_dir() {
            return Err(format!("路徑不是目錄: {}", directory.display()));
        }

        // 執行 terragrunt apply
        println!("執行 terragrunt apply...");

        let mut child = Command::new("terragrunt")
            .arg("apply")
            .arg("-auto-approve") // 自動批准，避免需要手動確認
            .current_dir(directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("無法啟動 terragrunt: {}", e))?;

        // 即時顯示輸出
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("  {}", line);
                }
            }
        }

        // 等待命令完成
        let status = child
            .wait()
            .map_err(|e| format!("等待命令完成時發生錯誤: {}", e))?;

        if status.success() {
            println!("✓ terragrunt apply 成功");
            Ok(())
        } else {
            Err(format!(
                "terragrunt apply 失敗，退出碼: {}",
                status.code().unwrap_or(-1)
            ))
        }
    }

    /// 檢查 terragrunt 是否已安裝
    pub fn check_installed() -> bool {
        Command::new("terragrunt").arg("--version").output().is_ok()
    }

    /// 執行 terragrunt plan（供未來擴展使用）
    #[allow(dead_code)]
    pub fn plan(&self, directory: &Path) -> Result<(), String> {
        println!("\n進入目錄: {}", directory.display());

        let mut child = Command::new("terragrunt")
            .arg("plan")
            .current_dir(directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("無法啟動 terragrunt: {}", e))?;

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("  {}", line);
                }
            }
        }

        let status = child
            .wait()
            .map_err(|e| format!("等待命令完成時發生錯誤: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "terragrunt plan 失敗，退出碼: {}",
                status.code().unwrap_or(-1)
            ))
        }
    }
}

impl Default for TerragruntExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let _executor = TerragruntExecutor::new();
    }

    #[test]
    fn test_check_installed() {
        // 這個測試會檢查系統是否安裝了 terragrunt
        // 如果沒安裝，測試不會失敗，只是返回 false
        let _is_installed = TerragruntExecutor::check_installed();
    }
}
