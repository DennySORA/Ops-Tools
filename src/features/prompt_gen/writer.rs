//! 檔案寫入器
//!
//! 負責將生成的提示寫入檔案系統

use super::models::FeaturePrompts;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

// ============================================================================
// 寫入器
// ============================================================================

/// 提示檔案寫入器
pub struct PromptWriter {
    /// 輸出基礎目錄
    base_dir: PathBuf,
    /// 是否覆蓋現有檔案
    overwrite: bool,
}

impl PromptWriter {
    /// 建立新的寫入器
    pub fn new(base_dir: PathBuf, overwrite: bool) -> Self {
        Self {
            base_dir,
            overwrite,
        }
    }

    /// 寫入單一功能的所有提示
    pub fn write_feature_prompts(&self, feature_prompts: &FeaturePrompts) -> Result<()> {
        let feature_dir = self.base_dir.join(feature_prompts.feature_key.as_str());

        // 建立功能目錄
        std::fs::create_dir_all(&feature_dir)
            .with_context(|| format!("無法建立目錄：{}", feature_dir.display()))?;

        // 寫入每個提示檔案
        for prompt in &feature_prompts.prompts {
            let file_path = feature_dir.join(&prompt.filename);
            self.safe_write(&file_path, &prompt.content)?;
        }

        Ok(())
    }

    /// 安全寫入檔案（檢查是否覆蓋）
    fn safe_write(&self, path: &PathBuf, content: &str) -> Result<()> {
        // 檢查檔案是否存在
        if path.exists() && !self.overwrite {
            bail!("檔案已存在且未指定 --overwrite：{}", path.display());
        }

        // 確保父目錄存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 寫入內容（確保結尾有換行）
        let final_content = if content.ends_with('\n') {
            content.to_string()
        } else {
            format!("{}\n", content.trim_end())
        };

        std::fs::write(path, &final_content)
            .with_context(|| format!("無法寫入檔案：{}", path.display()))?;

        Ok(())
    }
}

// ============================================================================
// 測試
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::prompt_gen::models::{FeatureKey, GeneratedPrompt};
    use tempfile::tempdir;

    #[test]
    fn test_write_feature_prompts() {
        let temp_dir = tempdir().unwrap();
        let writer = PromptWriter::new(temp_dir.path().to_path_buf(), false);

        let mut prompts = FeaturePrompts::new(FeatureKey::new("test-feature").unwrap());
        prompts.add_prompt("01_test.md", "Test content");

        let result = writer.write_feature_prompts(&prompts);
        assert!(result.is_ok());

        let file_path = temp_dir.path().join("test-feature").join("01_test.md");
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content\n");
    }

    #[test]
    fn test_write_no_overwrite() {
        let temp_dir = tempdir().unwrap();
        let writer = PromptWriter::new(temp_dir.path().to_path_buf(), false);

        let mut prompts = FeaturePrompts::new(FeatureKey::new("test-feature").unwrap());
        prompts.add_prompt("01_test.md", "Test content");

        // 第一次寫入應該成功
        assert!(writer.write_feature_prompts(&prompts).is_ok());

        // 第二次寫入應該失敗（檔案已存在）
        assert!(writer.write_feature_prompts(&prompts).is_err());
    }

    #[test]
    fn test_write_with_overwrite() {
        let temp_dir = tempdir().unwrap();
        let writer = PromptWriter::new(temp_dir.path().to_path_buf(), true);

        let mut prompts = FeaturePrompts::new(FeatureKey::new("test-feature").unwrap());
        prompts.add_prompt("01_test.md", "Original content");

        writer.write_feature_prompts(&prompts).unwrap();

        // 修改內容
        prompts.prompts[0] = GeneratedPrompt {
            filename: "01_test.md".to_string(),
            content: "Updated content".to_string(),
        };

        // 第二次寫入應該成功（overwrite = true）
        assert!(writer.write_feature_prompts(&prompts).is_ok());

        let file_path = temp_dir.path().join("test-feature").join("01_test.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Updated content\n");
    }
}
