use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "kebab-case")]
pub enum OperatingSystem {
    Macos,
    #[default]
    Linux,
}

impl OperatingSystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Linux => "linux",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PlatformClass {
    Macos,
    Gb10,
    NvidiaLinux,
    #[default]
    GenericLinux,
}

impl PlatformClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Gb10 => "gb10",
            Self::NvidiaLinux => "nvidia-linux",
            Self::GenericLinux => "generic-linux",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Macos => "macOS",
            Self::Gb10 => "GB10 / DGX Spark",
            Self::NvidiaLinux => "NVIDIA Linux",
            Self::GenericLinux => "Generic Linux",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct PlatformInfo {
    pub os: OperatingSystem,
    pub class: PlatformClass,
    pub version: Option<String>,
    pub arch: Option<String>,
    pub model: Option<String>,
    pub detection_source: Option<String>,
}

impl PlatformInfo {
    pub fn generic_linux(
        model: Option<String>,
        version: Option<String>,
        arch: Option<String>,
    ) -> Self {
        Self {
            os: OperatingSystem::Linux,
            class: PlatformClass::GenericLinux,
            version: normalize(version),
            arch: normalize(arch),
            model: normalize(model),
            detection_source: None,
        }
    }

    pub fn macos(model: Option<String>, version: Option<String>, arch: Option<String>) -> Self {
        Self {
            os: OperatingSystem::Macos,
            class: PlatformClass::Macos,
            version: normalize(version),
            arch: normalize(arch),
            model: normalize(model),
            detection_source: Some("uname/sysctl/sw_vers".to_string()),
        }
    }

    pub fn gb10(
        model: Option<String>,
        version: Option<String>,
        arch: Option<String>,
        detection_source: impl Into<String>,
    ) -> Self {
        Self {
            os: OperatingSystem::Linux,
            class: PlatformClass::Gb10,
            version: normalize(version),
            arch: normalize(arch),
            model: normalize(model),
            detection_source: Some(detection_source.into()),
        }
    }

    pub fn nvidia_linux(
        model: Option<String>,
        version: Option<String>,
        arch: Option<String>,
        detection_source: impl Into<String>,
    ) -> Self {
        Self {
            os: OperatingSystem::Linux,
            class: PlatformClass::NvidiaLinux,
            version: normalize(version),
            arch: normalize(arch),
            model: normalize(model),
            detection_source: Some(detection_source.into()),
        }
    }

    pub fn is_linux(&self) -> bool {
        matches!(self.os, OperatingSystem::Linux)
    }

    pub fn is_macos(&self) -> bool {
        matches!(self.os, OperatingSystem::Macos)
    }

    pub fn is_gb10(&self) -> bool {
        matches!(self.class, PlatformClass::Gb10)
    }

    pub fn is_nvidia_linux(&self) -> bool {
        matches!(self.class, PlatformClass::Gb10 | PlatformClass::NvidiaLinux)
    }

    pub fn supports_gb10_tuning(&self) -> bool {
        self.is_gb10()
    }

    pub fn expects_nvidia_tooling(&self) -> bool {
        self.is_nvidia_linux()
    }

    pub fn supports_apt(&self) -> bool {
        self.is_linux()
    }

    pub fn supports_homebrew(&self) -> bool {
        self.is_macos()
    }

    pub fn supports_linux_service_management(&self) -> bool {
        self.is_linux()
    }

    pub fn supports_needrestart(&self) -> bool {
        self.is_linux()
    }

    pub fn supports_reboot_workflow(&self) -> bool {
        self.is_linux()
    }

    pub fn label(&self) -> &'static str {
        self.class.label()
    }

    pub fn summary(&self) -> String {
        let base = match self.version.as_deref() {
            Some(version) if self.is_macos() => format!("{} {version}", self.label()),
            _ => self.label().to_string(),
        };

        match self.model.as_deref() {
            Some(model) if !model.eq_ignore_ascii_case(self.label()) => {
                format!("{base} ({model})")
            }
            _ => base,
        }
    }

    pub fn detection_note(&self) -> String {
        match self.detection_source.as_deref() {
            Some(source) => format!("{} via {source}", self.summary()),
            None => self.summary(),
        }
    }
}

fn normalize(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}
