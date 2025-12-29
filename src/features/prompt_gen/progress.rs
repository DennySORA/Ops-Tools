//! 進度追蹤模組
//!
//! 負責追蹤和保存每個功能的執行進度

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

// ============================================================================
// 步驟定義
// ============================================================================

/// 執行步驟
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Step {
    /// 未開始
    #[default]
    None,
    /// 步驟 1: 需求與交付
    P1,
    /// 步驟 2: INT E2E 驗證
    P2,
    /// 步驟 3: 重構與優化
    P3,
    /// 步驟 4: INT E2E 回歸
    P4,
}

impl Step {
    /// 所有步驟（不含 None）
    pub const ALL: [Step; 4] = [Step::P1, Step::P2, Step::P3, Step::P4];

    /// 取得下一個步驟
    pub fn next(self) -> Option<Step> {
        match self {
            Step::None => Some(Step::P1),
            Step::P1 => Some(Step::P2),
            Step::P2 => Some(Step::P3),
            Step::P3 => Some(Step::P4),
            Step::P4 => None,
        }
    }

    /// 從字串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "" | "none" => Some(Step::None),
            "p1" => Some(Step::P1),
            "p2" => Some(Step::P2),
            "p3" => Some(Step::P3),
            "p4" => Some(Step::P4),
            _ => None,
        }
    }

    /// 轉為字串
    pub fn as_str(&self) -> &'static str {
        match self {
            Step::None => "",
            Step::P1 => "p1",
            Step::P2 => "p2",
            Step::P3 => "p3",
            Step::P4 => "p4",
        }
    }

    /// 取得對應的提示檔案名稱
    pub fn prompt_filename(&self) -> Option<&'static str> {
        match self {
            Step::None => None,
            Step::P1 => Some("01_requirements_and_delivery.md"),
            Step::P2 => Some("02_int_e2e_validate.md"),
            Step::P3 => Some("03_refactor_and_polish.md"),
            Step::P4 => Some("04_int_e2e_regression.md"),
        }
    }

    /// 取得步驟描述
    pub fn description(&self) -> &'static str {
        match self {
            Step::None => "未開始",
            Step::P1 => "需求、實作、部署（INT）",
            Step::P2 => "INT E2E 驗證",
            Step::P3 => "重構、流程優化、品質提升",
            Step::P4 => "INT E2E 回歸驗證",
        }
    }

    /// 是否需要 resume session
    pub fn needs_resume(&self) -> bool {
        matches!(self, Step::P2 | Step::P3 | Step::P4)
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// 功能狀態
// ============================================================================

/// 功能狀態（從 STATE.md 解析）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureStatus {
    /// 未知狀態
    Unknown,
    /// P1 完成並部署到 INT
    P1DoneIntDeployed,
    /// P2 E2E 通過
    P2E2EPassed,
    /// P3 重構完成並部署到 INT
    P3RefactoredIntDeployed,
    /// 就緒
    Ready,
}

impl FeatureStatus {
    /// 從字串解析
    pub fn from_str(s: &str) -> Self {
        let s = s.trim().to_uppercase();
        match s.as_str() {
            "P1_DONE_INT_DEPLOYED" => Self::P1DoneIntDeployed,
            "P2_E2E_PASSED" => Self::P2E2EPassed,
            "P3_REFACTORED_INT_DEPLOYED" => Self::P3RefactoredIntDeployed,
            "READY" => Self::Ready,
            _ => Self::Unknown,
        }
    }

    /// 是否就緒
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

impl fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Unknown => "UNKNOWN",
            Self::P1DoneIntDeployed => "P1_DONE_INT_DEPLOYED",
            Self::P2E2EPassed => "P2_E2E_PASSED",
            Self::P3RefactoredIntDeployed => "P3_REFACTORED_INT_DEPLOYED",
            Self::Ready => "READY",
        };
        write!(f, "{}", s)
    }
}

// ============================================================================
// 進度記錄
// ============================================================================

/// 功能進度記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Claude session ID
    pub session_id: Option<String>,
    /// 最後完成的步驟
    pub last_done: Step,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            session_id: None,
            last_done: Step::None,
        }
    }
}

impl Progress {
    /// 從檔案載入
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("無法讀取進度檔案：{}", path.display()))?;

        Self::parse(&content)
    }

    /// 從字串解析（相容 shell 格式）
    pub fn parse(content: &str) -> Result<Self> {
        let mut session_id = None;
        let mut last_done = Step::None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "SESSION_ID" => {
                        if !value.is_empty() {
                            session_id = Some(value.to_string());
                        }
                    }
                    "LAST_DONE" => {
                        last_done = Step::from_str(value).unwrap_or(Step::None);
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            session_id,
            last_done,
        })
    }

    /// 儲存到檔案（相容 shell 格式）
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        // 確保父目錄存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = format!(
            "SESSION_ID=\"{}\"\nLAST_DONE=\"{}\"\n",
            self.session_id.as_deref().unwrap_or(""),
            self.last_done.as_str()
        );

        std::fs::write(path, &content)
            .with_context(|| format!("無法寫入進度檔案：{}", path.display()))?;

        Ok(())
    }

    /// 取得下一個要執行的步驟
    pub fn next_step(&self) -> Option<Step> {
        self.last_done.next()
    }

    /// 更新進度
    pub fn mark_done(&mut self, step: Step, session_id: Option<String>) {
        self.last_done = step;
        if let Some(sid) = session_id {
            self.session_id = Some(sid);
        }
    }
}

// ============================================================================
// 狀態檔案解析
// ============================================================================

/// 從 STATE.md 檔案讀取狀態
pub fn read_state_status<P: AsRef<Path>>(state_file: P) -> Result<FeatureStatus> {
    let path = state_file.as_ref();
    if !path.exists() {
        return Ok(FeatureStatus::Unknown);
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("無法讀取狀態檔案：{}", path.display()))?;

    // 使用正則表達式匹配 STATUS: xxx 或 STATUS：xxx
    let re = Regex::new(r"STATUS[：:]\s*(\S+)")?;

    if let Some(caps) = re.captures(&content) {
        if let Some(status_match) = caps.get(1) {
            return Ok(FeatureStatus::from_str(status_match.as_str()));
        }
    }

    Ok(FeatureStatus::Unknown)
}

// ============================================================================
// 功能資訊
// ============================================================================

/// 單一功能的完整資訊
#[derive(Debug, Clone)]
pub struct FeatureInfo {
    /// 功能鍵值
    pub feature_key: String,
    /// 功能目錄路徑
    pub feature_dir: PathBuf,
    /// 當前進度
    pub progress: Progress,
    /// 當前狀態
    pub status: FeatureStatus,
}

impl FeatureInfo {
    /// 從功能目錄載入
    pub fn load_from_dir<P: AsRef<Path>>(feature_dir: P, feature_key: &str) -> Result<Self> {
        let feature_dir = feature_dir.as_ref().to_path_buf();

        let progress_file = feature_dir.join("progress.env");
        let progress = Progress::load_from_file(&progress_file)?;

        let state_file = feature_dir.join("STATE.md");
        let status = read_state_status(&state_file)?;

        Ok(Self {
            feature_key: feature_key.to_string(),
            feature_dir,
            progress,
            status,
        })
    }

    /// 取得進度檔案路徑
    pub fn progress_file(&self) -> PathBuf {
        self.feature_dir.join("progress.env")
    }

    /// 取得狀態檔案路徑
    pub fn state_file(&self) -> PathBuf {
        self.feature_dir.join("STATE.md")
    }

    /// 取得指定步驟的提示檔案路徑
    pub fn prompt_file(&self, step: Step) -> Option<PathBuf> {
        step.prompt_filename().map(|f| self.feature_dir.join(f))
    }

    /// 取得日誌目錄路徑
    pub fn logs_dir(&self) -> PathBuf {
        self.feature_dir.join("runner_logs")
    }
}

// ============================================================================
// 測試
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_order() {
        assert!(Step::None < Step::P1);
        assert!(Step::P1 < Step::P2);
        assert!(Step::P2 < Step::P3);
        assert!(Step::P3 < Step::P4);
    }

    #[test]
    fn test_step_next() {
        assert_eq!(Step::None.next(), Some(Step::P1));
        assert_eq!(Step::P1.next(), Some(Step::P2));
        assert_eq!(Step::P4.next(), None);
    }

    #[test]
    fn test_progress_parse() {
        let content = r#"
SESSION_ID="abc123"
LAST_DONE="p2"
"#;
        let progress = Progress::parse(content).unwrap();
        assert_eq!(progress.session_id, Some("abc123".to_string()));
        assert_eq!(progress.last_done, Step::P2);
    }

    #[test]
    fn test_progress_empty() {
        let content = "";
        let progress = Progress::parse(content).unwrap();
        assert_eq!(progress.session_id, None);
        assert_eq!(progress.last_done, Step::None);
    }

    #[test]
    fn test_feature_status_parse() {
        assert_eq!(FeatureStatus::from_str("READY"), FeatureStatus::Ready);
        assert_eq!(
            FeatureStatus::from_str("P1_DONE_INT_DEPLOYED"),
            FeatureStatus::P1DoneIntDeployed
        );
    }
}
