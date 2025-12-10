use std::collections::HashMap;

/// 環境配置，定義各環境對應的區域列表
pub struct EnvironmentConfig {
    environments: HashMap<String, Vec<String>>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        let mut environments = HashMap::new();

        environments.insert("dev".to_string(), vec!["us-east-1".to_string()]);

        environments.insert("int".to_string(), vec!["us-east-1".to_string()]);

        environments.insert(
            "prod".to_string(),
            vec![
                "ap-northeast-1".to_string(),
                "ap-south-1".to_string(),
                "ap-southeast-1".to_string(),
                "ap-southeast-2".to_string(),
                "ca-central-1".to_string(),
                "eu-central-1".to_string(),
                "eu-west-2".to_string(),
                "me-central-1".to_string(),
                "us-east-1".to_string(),
            ],
        );

        environments.insert(
            "stg".to_string(),
            vec!["eu-central-1".to_string(), "us-east-1".to_string()],
        );

        Self { environments }
    }
}

impl EnvironmentConfig {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn environments(&self) -> &HashMap<String, Vec<String>> {
        &self.environments
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.environments.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_four_environments() {
        let config = EnvironmentConfig::default();
        assert_eq!(config.environments().len(), 4);
    }

    #[test]
    fn dev_has_one_region() {
        let config = EnvironmentConfig::default();
        let dev_regions = config.environments().get("dev").unwrap();
        assert_eq!(dev_regions.len(), 1);
        assert_eq!(dev_regions[0], "us-east-1");
    }

    #[test]
    fn prod_has_nine_regions() {
        let config = EnvironmentConfig::default();
        let prod_regions = config.environments().get("prod").unwrap();
        assert_eq!(prod_regions.len(), 9);
    }
}
