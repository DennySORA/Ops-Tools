use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PlatformClass {
    Gb10,
    NvidiaLinux,
    #[default]
    GenericLinux,
}

impl PlatformClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gb10 => "gb10",
            Self::NvidiaLinux => "nvidia-linux",
            Self::GenericLinux => "generic-linux",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Gb10 => "GB10 / DGX Spark",
            Self::NvidiaLinux => "NVIDIA Linux",
            Self::GenericLinux => "Generic Linux",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct PlatformInfo {
    pub class: PlatformClass,
    pub model: Option<String>,
    pub detection_source: Option<String>,
}

impl PlatformInfo {
    pub fn generic_linux(model: Option<String>) -> Self {
        Self {
            class: PlatformClass::GenericLinux,
            model: normalize(model),
            detection_source: None,
        }
    }

    pub fn gb10(model: Option<String>, detection_source: impl Into<String>) -> Self {
        Self {
            class: PlatformClass::Gb10,
            model: normalize(model),
            detection_source: Some(detection_source.into()),
        }
    }

    pub fn nvidia_linux(model: Option<String>, detection_source: impl Into<String>) -> Self {
        Self {
            class: PlatformClass::NvidiaLinux,
            model: normalize(model),
            detection_source: Some(detection_source.into()),
        }
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

    pub fn label(&self) -> &'static str {
        self.class.label()
    }

    pub fn summary(&self) -> String {
        match self.model.as_deref() {
            Some(model) if !model.eq_ignore_ascii_case(self.label()) => {
                format!("{} ({model})", self.label())
            }
            _ => self.label().to_string(),
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
