use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::config::EnvironmentConfig;
use super::template::{TemplateManager, TemplateType};

/// 預覽結果 - 顯示將要創建和已存在的文件
#[derive(Debug, Default)]
pub struct PreviewResult {
    pub will_create: Vec<PathBuf>,
    pub already_exists: Vec<PathBuf>,
}

impl PreviewResult {
    pub fn files_to_create(&self) -> usize {
        self.will_create.len()
    }

    pub fn has_files_to_create(&self) -> bool {
        !self.will_create.is_empty()
    }
}

/// 生成結果統計
#[derive(Debug, Default)]
pub struct GenerationResult {
    pub created_files: Vec<PathBuf>,
    pub existing_files: Vec<PathBuf>,
    pub errors: Vec<String>,
}

impl GenerationResult {
    pub fn files_created(&self) -> usize {
        self.created_files.len()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Terragrunt 項目結構生成器
pub struct TerragruntGenerator {
    config: EnvironmentConfig,
    template_manager: TemplateManager,
    base_path: PathBuf,
}

impl TerragruntGenerator {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            config: EnvironmentConfig::new(),
            template_manager: TemplateManager::new(),
            base_path,
        }
    }

    #[allow(dead_code)]
    pub fn with_templates_from(mut self, templates_dir: &Path) -> Self {
        self.template_manager = TemplateManager::load_from_directory(templates_dir);
        self
    }

    /// 列出現有的服務（從 stacks/_env 讀取）
    pub fn list_existing_services(&self) -> io::Result<Vec<String>> {
        let env_dir = self.base_path.join("stacks").join("_env");

        if !env_dir.exists() {
            return Ok(Vec::new());
        }

        let mut services = Vec::new();

        for entry in fs::read_dir(&env_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    services.push(name.to_string());
                }
            }
        }

        services.sort();
        Ok(services)
    }

    /// 驗證服務名稱是否有效
    pub fn validate_service_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("服務名稱不能為空".to_string());
        }

        let is_valid = name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_');

        if !is_valid {
            return Err("服務名稱只能包含字母、數字、連字符和下劃線".to_string());
        }

        Ok(())
    }

    /// 取得 stacks 目錄路徑
    fn stacks_path(&self) -> PathBuf {
        self.base_path.join("stacks")
    }

    /// 預覽將要生成的結構（不實際創建文件）
    pub fn preview_structure(&self, service_name: &str) -> PreviewResult {
        let mut result = PreviewResult::default();
        let stacks_path = self.stacks_path();

        // 預覽 _env 結構: stacks/_env/{service}/{service}.hcl
        let env_file = stacks_path
            .join("_env")
            .join(service_name)
            .join(format!("{service_name}.hcl"));

        if env_file.exists() {
            result.already_exists.push(env_file);
        } else {
            result.will_create.push(env_file);
        }

        // 預覽各環境和區域的結構: stacks/{env}/{region}/{service}/
        for (env, regions) in self.config.iter() {
            for region in regions {
                let service_dir = stacks_path.join(env).join(region).join(service_name);

                // terragrunt.hcl
                let terragrunt_file = service_dir.join("terragrunt.hcl");
                if terragrunt_file.exists() {
                    result.already_exists.push(terragrunt_file);
                } else {
                    result.will_create.push(terragrunt_file);
                }

                // locals.tf
                let locals_file = service_dir.join("locals.tf");
                if locals_file.exists() {
                    result.already_exists.push(locals_file);
                } else {
                    result.will_create.push(locals_file);
                }
            }
        }

        result
    }

    /// 生成完整的服務結構
    pub fn generate_structure(&self, service_name: &str) -> GenerationResult {
        let mut result = GenerationResult::default();

        // 創建 _env 結構
        if let Err(err) = self.create_env_structure(service_name, &mut result) {
            result.errors.push(format!("創建 _env 結構失敗: {err}"));
        }

        // 為每個環境和區域創建結構
        for (env, regions) in self.config.iter() {
            for region in regions {
                if let Err(err) =
                    self.create_service_structure(service_name, env, region, &mut result)
                {
                    result.errors.push(format!(
                        "創建 {env}/{region}/{service_name} 結構失敗: {err}"
                    ));
                }
            }
        }

        result
    }

    /// 在 stacks/_env 目錄中創建服務結構
    fn create_env_structure(
        &self,
        service_name: &str,
        result: &mut GenerationResult,
    ) -> io::Result<()> {
        let stacks_path = self.stacks_path();
        let env_dir = stacks_path.join("_env").join(service_name);
        let hcl_file = env_dir.join(format!("{service_name}.hcl"));

        fs::create_dir_all(&env_dir)?;

        if hcl_file.exists() {
            result.existing_files.push(hcl_file);
        } else {
            let content = self
                .template_manager
                .render(TemplateType::Env, service_name);
            fs::write(&hcl_file, content)?;
            result.created_files.push(hcl_file);
        }

        Ok(())
    }

    /// 在 stacks/{env}/{region}/{service}/ 中創建服務結構
    fn create_service_structure(
        &self,
        service_name: &str,
        env: &str,
        region: &str,
        result: &mut GenerationResult,
    ) -> io::Result<()> {
        let stacks_path = self.stacks_path();
        let service_dir = stacks_path.join(env).join(region).join(service_name);

        fs::create_dir_all(&service_dir)?;

        // 創建 terragrunt.hcl
        let terragrunt_file = service_dir.join("terragrunt.hcl");
        if terragrunt_file.exists() {
            result.existing_files.push(terragrunt_file);
        } else {
            let content = self
                .template_manager
                .render(TemplateType::Terragrunt, service_name);
            fs::write(&terragrunt_file, content)?;
            result.created_files.push(terragrunt_file);
        }

        // 創建 locals.tf
        let locals_file = service_dir.join("locals.tf");
        if locals_file.exists() {
            result.existing_files.push(locals_file);
        } else {
            let content = self
                .template_manager
                .render(TemplateType::Locals, service_name);
            fs::write(&locals_file, content)?;
            result.created_files.push(locals_file);
        }

        Ok(())
    }

    /// 保存模版到指定目錄
    pub fn save_templates(&self, templates_dir: &Path) -> io::Result<()> {
        self.template_manager.save_to_directory(templates_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn validate_service_name_accepts_valid_names() {
        assert!(TerragruntGenerator::validate_service_name("my-service").is_ok());
        assert!(TerragruntGenerator::validate_service_name("my_service").is_ok());
        assert!(TerragruntGenerator::validate_service_name("service123").is_ok());
    }

    #[test]
    fn validate_service_name_rejects_empty() {
        assert!(TerragruntGenerator::validate_service_name("").is_err());
    }

    #[test]
    fn validate_service_name_rejects_invalid_chars() {
        assert!(TerragruntGenerator::validate_service_name("my service").is_err());
        assert!(TerragruntGenerator::validate_service_name("my.service").is_err());
        assert!(TerragruntGenerator::validate_service_name("my/service").is_err());
    }

    #[test]
    fn generate_structure_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let generator = TerragruntGenerator::new(temp_dir.path().to_path_buf());

        let result = generator.generate_structure("test-service");

        assert!(!result.has_errors());
        assert!(result.files_created() > 0);

        // 驗證 _env 結構: stacks/_env/{service}/{service}.hcl
        let env_file = temp_dir
            .path()
            .join("stacks/_env/test-service/test-service.hcl");
        assert!(env_file.exists());

        // 驗證環境結構: stacks/{env}/{region}/{service}/terragrunt.hcl
        let dev_file = temp_dir
            .path()
            .join("stacks/dev/us-east-1/test-service/terragrunt.hcl");
        assert!(dev_file.exists());
    }

    #[test]
    fn list_existing_services_returns_empty_when_no_env_dir() {
        let temp_dir = TempDir::new().unwrap();
        let generator = TerragruntGenerator::new(temp_dir.path().to_path_buf());

        let services = generator.list_existing_services().unwrap();
        assert!(services.is_empty());
    }

    #[test]
    fn list_existing_services_finds_created_services() {
        let temp_dir = TempDir::new().unwrap();
        let generator = TerragruntGenerator::new(temp_dir.path().to_path_buf());

        generator.generate_structure("service-a");
        generator.generate_structure("service-b");

        let services = generator.list_existing_services().unwrap();
        assert_eq!(services.len(), 2);
        assert!(services.contains(&"service-a".to_string()));
        assert!(services.contains(&"service-b".to_string()));
    }

    #[test]
    fn preview_shows_files_to_create() {
        let temp_dir = TempDir::new().unwrap();
        let generator = TerragruntGenerator::new(temp_dir.path().to_path_buf());

        let preview = generator.preview_structure("new-service");

        // 應該有文件要創建
        assert!(preview.has_files_to_create());
        assert!(preview.files_to_create() > 0);
        // 沒有已存在的文件
        assert!(preview.already_exists.is_empty());
    }

    #[test]
    fn preview_shows_existing_files_after_generation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = TerragruntGenerator::new(temp_dir.path().to_path_buf());

        // 先生成結構
        generator.generate_structure("existing-service");

        // 再預覽相同服務
        let preview = generator.preview_structure("existing-service");

        // 不應該有文件要創建
        assert!(!preview.has_files_to_create());
        // 所有文件都已存在
        assert!(!preview.already_exists.is_empty());
    }
}
