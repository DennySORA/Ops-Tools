use std::path::{Path, PathBuf};
use std::process::Command;

/// Kubeconfig 視窗隔離服務
pub struct KubeconfigService {
    /// 預設的 kubeconfig 路徑
    base_kubeconfig: PathBuf,
    /// 視窗專屬 kubeconfig 的目錄
    configs_dir: PathBuf,
}

impl KubeconfigService {
    /// 建立新的 KubeconfigService 實例
    pub fn new() -> Result<Self, String> {
        let home = dirs::home_dir().ok_or("Unable to determine home directory")?;
        let base_kubeconfig = home.join(".kube").join("config");
        let configs_dir = home.join(".kube").join("window-configs");

        Ok(Self {
            base_kubeconfig,
            configs_dir,
        })
    }

    /// 檢查是否在 tmux 環境中
    pub fn is_in_tmux(&self) -> bool {
        std::env::var("TMUX").is_ok()
    }

    /// 取得目前 tmux 視窗的唯一識別 ID
    /// 格式: session_name:window_index
    pub fn get_tmux_window_id(&self) -> Result<String, String> {
        let output = Command::new("tmux")
            .args(["display-message", "-p", "#{session_name}:#{window_index}"])
            .output()
            .map_err(|e| format!("Failed to execute tmux: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 取得視窗專屬 kubeconfig 的路徑
    pub fn get_window_kubeconfig_path(&self, window_id: &str) -> PathBuf {
        let safe_name = window_id.replace([':', '/'], "-");
        self.configs_dir.join(format!("{}.yaml", safe_name))
    }

    /// 建立視窗專屬的 kubeconfig
    pub fn setup_window_kubeconfig(&self, window_id: &str) -> Result<PathBuf, String> {
        // 確保目錄存在
        if !self.configs_dir.exists() {
            std::fs::create_dir_all(&self.configs_dir)
                .map_err(|e| format!("Failed to create configs directory: {}", e))?;
        }

        let config_path = self.get_window_kubeconfig_path(window_id);

        // 如果已存在，直接返回
        if config_path.exists() {
            return Ok(config_path);
        }

        // 檢查 base kubeconfig 是否存在
        if !self.base_kubeconfig.exists() {
            return Err(format!(
                "Base kubeconfig not found: {}",
                self.base_kubeconfig.display()
            ));
        }

        // 複製 base kubeconfig 到新的位置
        std::fs::copy(&self.base_kubeconfig, &config_path)
            .map_err(|e| format!("Failed to copy kubeconfig: {}", e))?;

        Ok(config_path)
    }

    /// 設定 tmux 視窗的環境變數
    pub fn set_tmux_env(&self, window_id: &str, config_path: &Path) -> Result<(), String> {
        // 取得 session 名稱
        let parts: Vec<&str> = window_id.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid window ID format: {}", window_id));
        }

        let target = format!("{}:{}", parts[0], parts[1]);

        let output = Command::new("tmux")
            .args([
                "set-environment",
                "-t",
                &target,
                "KUBECONFIG",
                &config_path.display().to_string(),
            ])
            .output()
            .map_err(|e| format!("Failed to execute tmux: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    /// 透過 tmux send-keys 在當前 shell 自動執行 export 指令
    pub fn apply_shell_env(&self, config_path: &Path) -> Result<(), String> {
        let export_cmd = format!("export KUBECONFIG=\"{}\"", config_path.display());

        let output = Command::new("tmux")
            .args(["send-keys", &export_cmd, "Enter"])
            .output()
            .map_err(|e| format!("Failed to execute tmux send-keys: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    /// 透過 tmux send-keys 在當前 shell 自動執行 unset 指令
    pub fn unapply_shell_env(&self) -> Result<(), String> {
        let output = Command::new("tmux")
            .args(["send-keys", "unset KUBECONFIG", "Enter"])
            .output()
            .map_err(|e| format!("Failed to execute tmux send-keys: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    /// 移除 tmux 視窗的環境變數
    pub fn unset_tmux_env(&self, window_id: &str) -> Result<(), String> {
        let parts: Vec<&str> = window_id.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid window ID format: {}", window_id));
        }

        let target = format!("{}:{}", parts[0], parts[1]);

        let output = Command::new("tmux")
            .args(["set-environment", "-t", &target, "-u", "KUBECONFIG"])
            .output()
            .map_err(|e| format!("Failed to execute tmux: {}", e))?;

        if !output.status.success() {
            // tmux 可能會因為變數不存在而失敗，這不是嚴重錯誤
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("unknown variable") {
                return Err(stderr.to_string());
            }
        }

        Ok(())
    }

    /// 清理視窗專屬的 kubeconfig
    pub fn cleanup_window_kubeconfig(&self, window_id: &str) -> Result<(), String> {
        let config_path = self.get_window_kubeconfig_path(window_id);

        if config_path.exists() {
            std::fs::remove_file(&config_path)
                .map_err(|e| format!("Failed to remove kubeconfig: {}", e))?;
        }

        Ok(())
    }

    /// 列出所有視窗專屬的 kubeconfig 檔案
    pub fn list_window_kubeconfigs(&self) -> Vec<PathBuf> {
        if !self.configs_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(&self.configs_dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .filter(|path| {
                        path.extension()
                            .is_some_and(|ext| ext == "yaml" || ext == "yml")
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 清理所有視窗專屬的 kubeconfig 檔案
    pub fn cleanup_all_kubeconfigs(&self) -> (usize, usize) {
        let configs = self.list_window_kubeconfigs();
        let mut success = 0;
        let mut failed = 0;

        for config in configs {
            match std::fs::remove_file(&config) {
                Ok(()) => success += 1,
                Err(_) => failed += 1,
            }
        }

        (success, failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestService {
        service: KubeconfigService,
        _temp_dir: TempDir,
    }

    impl TestService {
        fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let base_kubeconfig = temp_dir.path().join("config");
            let configs_dir = temp_dir.path().join("window-configs");

            // 建立假的 base kubeconfig
            std::fs::write(&base_kubeconfig, "apiVersion: v1\nkind: Config\n")
                .expect("Failed to write base config");

            let service = KubeconfigService {
                base_kubeconfig,
                configs_dir,
            };

            Self {
                service,
                _temp_dir: temp_dir,
            }
        }
    }

    #[test]
    fn test_service_creation() {
        let test = TestService::new();
        assert!(test.service.base_kubeconfig.exists());
    }

    #[test]
    fn test_get_window_kubeconfig_path() {
        let test = TestService::new();
        let path = test.service.get_window_kubeconfig_path("mysession:1");
        assert!(path.to_string_lossy().contains("mysession-1.yaml"));
    }

    #[test]
    fn test_setup_window_kubeconfig() {
        let test = TestService::new();
        let result = test.service.setup_window_kubeconfig("test:0");
        assert!(result.is_ok());

        let config_path = result.unwrap();
        assert!(config_path.exists());
    }

    #[test]
    fn test_cleanup_window_kubeconfig() {
        let test = TestService::new();

        // 先建立
        let config_path = test
            .service
            .setup_window_kubeconfig("test:0")
            .expect("Setup failed");
        assert!(config_path.exists());

        // 再清理
        let result = test.service.cleanup_window_kubeconfig("test:0");
        assert!(result.is_ok());
        assert!(!config_path.exists());
    }

    #[test]
    fn test_list_window_kubeconfigs() {
        let test = TestService::new();

        // 建立幾個 kubeconfig
        test.service
            .setup_window_kubeconfig("session1:0")
            .expect("Setup failed");
        test.service
            .setup_window_kubeconfig("session2:1")
            .expect("Setup failed");

        let configs = test.service.list_window_kubeconfigs();
        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn test_cleanup_all_kubeconfigs() {
        let test = TestService::new();

        // 建立幾個 kubeconfig
        test.service
            .setup_window_kubeconfig("session1:0")
            .expect("Setup failed");
        test.service
            .setup_window_kubeconfig("session2:1")
            .expect("Setup failed");

        let (success, failed) = test.service.cleanup_all_kubeconfigs();
        assert_eq!(success, 2);
        assert_eq!(failed, 0);

        let configs = test.service.list_window_kubeconfigs();
        assert!(configs.is_empty());
    }
}
