use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Container build engine type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    Docker,
    Buildah,
}

impl EngineType {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            EngineType::Docker => "Docker",
            EngineType::Buildah => "Buildah",
        }
    }
}

/// Target architecture for container image
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    Amd64,
    Arm64,
    ArmV7,
    JetsonNano,
}

impl Architecture {
    /// Get all supported architectures
    pub fn all() -> Vec<Architecture> {
        vec![
            Architecture::Amd64,
            Architecture::Arm64,
            Architecture::ArmV7,
            Architecture::JetsonNano,
        ]
    }

    /// Display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Architecture::Amd64 => "x86_64 / amd64",
            Architecture::Arm64 => "arm64 / aarch64",
            Architecture::ArmV7 => "armv7 / arm/v7",
            Architecture::JetsonNano => "Jetson Nano (aarch64)",
        }
    }

    /// Description for UI
    pub fn description(&self) -> &'static str {
        match self {
            Architecture::Amd64 => "Intel/AMD 64-bit processors",
            Architecture::Arm64 => "ARM 64-bit (Apple Silicon, AWS Graviton)",
            Architecture::ArmV7 => "ARM 32-bit (Raspberry Pi 2/3)",
            Architecture::JetsonNano => "NVIDIA Jetson Nano (L4T based)",
        }
    }

    /// Platform string for docker buildx / buildah
    pub fn platform(&self) -> &'static str {
        match self {
            Architecture::Amd64 => "linux/amd64",
            Architecture::Arm64 => "linux/arm64",
            Architecture::ArmV7 => "linux/arm/v7",
            Architecture::JetsonNano => "linux/arm64",
        }
    }

    /// Architecture string for buildah
    pub fn buildah_arch(&self) -> &'static str {
        match self {
            Architecture::Amd64 => "amd64",
            Architecture::Arm64 => "arm64",
            Architecture::ArmV7 => "arm",
            Architecture::JetsonNano => "arm64",
        }
    }

    /// Variant for ARM architectures (for buildah)
    pub fn buildah_variant(&self) -> Option<&'static str> {
        match self {
            Architecture::ArmV7 => Some("v7"),
            _ => None,
        }
    }

    /// Check if this is a Jetson-specific build
    pub fn is_jetson(&self) -> bool {
        matches!(self, Architecture::JetsonNano)
    }
}

/// Build context containing all build parameters
#[derive(Debug, Clone)]
pub struct BuildContext {
    pub dockerfile: PathBuf,
    pub context_dir: PathBuf,
    pub image_name: String,
    pub tag: String,
    pub architecture: Architecture,
    pub push: bool,
    pub registry: Option<String>,
}

impl BuildContext {
    /// Get full image reference (registry/name:tag)
    pub fn full_image_ref(&self) -> String {
        match &self.registry {
            Some(registry) => format!("{}/{}:{}", registry, self.image_name, self.tag),
            None => format!("{}:{}", self.image_name, self.tag),
        }
    }

    /// Get local image reference (name:tag)
    pub fn local_image_ref(&self) -> String {
        format!("{}:{}", self.image_name, self.tag)
    }
}

/// Result of a build or push operation
#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    #[allow(dead_code)]
    pub exit_code: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_platforms() {
        assert_eq!(Architecture::Amd64.platform(), "linux/amd64");
        assert_eq!(Architecture::Arm64.platform(), "linux/arm64");
        assert_eq!(Architecture::ArmV7.platform(), "linux/arm/v7");
        assert_eq!(Architecture::JetsonNano.platform(), "linux/arm64");
    }

    #[test]
    fn test_architecture_buildah() {
        assert_eq!(Architecture::Amd64.buildah_arch(), "amd64");
        assert_eq!(Architecture::ArmV7.buildah_arch(), "arm");
        assert_eq!(Architecture::ArmV7.buildah_variant(), Some("v7"));
        assert_eq!(Architecture::Arm64.buildah_variant(), None);
    }

    #[test]
    fn test_build_context_image_ref() {
        let context = BuildContext {
            dockerfile: PathBuf::from("Dockerfile"),
            context_dir: PathBuf::from("."),
            image_name: "myapp".to_string(),
            tag: "v1.0".to_string(),
            architecture: Architecture::Amd64,
            push: false,
            registry: None,
        };
        assert_eq!(context.local_image_ref(), "myapp:v1.0");
        assert_eq!(context.full_image_ref(), "myapp:v1.0");

        let context_with_registry = BuildContext {
            registry: Some("docker.io/myuser".to_string()),
            ..context
        };
        assert_eq!(
            context_with_registry.full_image_ref(),
            "docker.io/myuser/myapp:v1.0"
        );
    }
}
