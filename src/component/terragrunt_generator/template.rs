use std::fs;
use std::io;
use std::path::Path;

/// 模版類型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    Env,
    Terragrunt,
    Locals,
}

impl TemplateType {
    #[allow(dead_code)]
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Env => "env_template.hcl",
            Self::Terragrunt => "terragrunt_template.hcl",
            Self::Locals => "locals_template.tf",
        }
    }
}

/// 模版管理器
pub struct TemplateManager {
    env_template: String,
    terragrunt_template: String,
    locals_template: String,
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self {
            env_template: Self::default_env_template(),
            terragrunt_template: Self::default_terragrunt_template(),
            locals_template: Self::default_locals_template(),
        }
    }
}

impl TemplateManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 從指定目錄載入模版，如果失敗則使用預設模版
    #[allow(dead_code)]
    pub fn load_from_directory(templates_dir: &Path) -> Self {
        let mut manager = Self::default();

        if let Ok(content) = fs::read_to_string(templates_dir.join("env_template.hcl")) {
            manager.env_template = content;
        }

        if let Ok(content) = fs::read_to_string(templates_dir.join("terragrunt_template.hcl")) {
            manager.terragrunt_template = content;
        }

        if let Ok(content) = fs::read_to_string(templates_dir.join("locals_template.tf")) {
            manager.locals_template = content;
        }

        manager
    }

    /// 渲染模版，將 $service_name 替換為實際的服務名稱
    pub fn render(&self, template_type: TemplateType, service_name: &str) -> String {
        let template = match template_type {
            TemplateType::Env => &self.env_template,
            TemplateType::Terragrunt => &self.terragrunt_template,
            TemplateType::Locals => &self.locals_template,
        };

        template.replace("$service_name", service_name)
    }

    /// 將模版保存到指定目錄
    pub fn save_to_directory(&self, templates_dir: &Path) -> io::Result<()> {
        fs::create_dir_all(templates_dir)?;

        fs::write(templates_dir.join("env_template.hcl"), &self.env_template)?;

        fs::write(
            templates_dir.join("terragrunt_template.hcl"),
            &self.terragrunt_template,
        )?;

        fs::write(
            templates_dir.join("locals_template.tf"),
            &self.locals_template,
        )?;

        Ok(())
    }

    fn default_env_template() -> String {
        r#"terraform {
  source = "../../../../modules//infrastructure-modules/$service_name"
}

generate "provider" {
  path      = "provider.tf"
  if_exists = "overwrite_terragrunt"
  contents  = <<EOF

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.64.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.17.0"
    }
  }
}

provider "aws" {
  region = var.region
}

provider "kubernetes" {
  host                   = data.terraform_remote_state.eks.outputs.eks.cluster_endpoint
  cluster_ca_certificate = base64decode(data.terraform_remote_state.eks.outputs.eks.cluster_certificate_authority_data)
  exec {
    api_version = "client.authentication.k8s.io/v1beta1"
    command     = "aws"
    # This requires the awscli to be installed locally where Terraform is executed
    args = ["eks", "get-token", "--cluster-name", data.terraform_remote_state.eks.outputs.eks.cluster_name]
  }
}

EOF
}

locals {
  env_vars = read_terragrunt_config(find_in_parent_folders("env.hcl"))
  owner    = local.env_vars.locals.owner
  env      = local.env_vars.locals.env
  sys      = local.env_vars.locals.sys

  region_vars = read_terragrunt_config(find_in_parent_folders("region.hcl"))
  region      = local.region_vars.locals.region
}

inputs = merge(
  local.env_vars.locals,
  local.region_vars.locals,
  {
    resource_prefix             = "${local.owner}-${local.env}-${local.region}-${local.sys}"
    resource_prefix_without_sys = "${local.owner}-${local.env}-${local.region}"
  }
)
"#
        .to_string()
    }

    fn default_terragrunt_template() -> String {
        r#"include "root" {
  path = find_in_parent_folders()
}

include "env" {
  path = "${get_terragrunt_dir()}/../../../_env/$service_name/$service_name.hcl"
}
"#
        .to_string()
    }

    fn default_locals_template() -> String {
        "locals {}\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_replaces_service_name() {
        let manager = TemplateManager::new();
        let rendered = manager.render(TemplateType::Terragrunt, "my-service");

        assert!(rendered.contains("my-service"));
        assert!(!rendered.contains("$service_name"));
    }

    #[test]
    fn default_templates_are_not_empty() {
        let manager = TemplateManager::new();

        assert!(!manager.env_template.is_empty());
        assert!(!manager.terragrunt_template.is_empty());
        assert!(!manager.locals_template.is_empty());
    }

    #[test]
    fn env_template_contains_terraform_source() {
        let manager = TemplateManager::new();
        let rendered = manager.render(TemplateType::Env, "test-service");

        assert!(rendered.contains("terraform {"));
        assert!(rendered
            .contains("source = \"../../../../modules//infrastructure-modules/test-service\""));
    }
}
