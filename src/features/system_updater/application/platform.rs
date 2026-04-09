use crate::features::system_updater::domain::platform::PlatformInfo;
use crate::features::system_updater::ports::HostServices;
use std::path::Path;

const DETECTION_PATHS: [(&str, &str, bool); 5] = [
    ("/sys/class/dmi/id/product_name", "dmi.product_name", true),
    (
        "/sys/class/dmi/id/product_family",
        "dmi.product_family",
        true,
    ),
    ("/sys/class/dmi/id/board_name", "dmi.board_name", true),
    ("/proc/device-tree/model", "device_tree.model", true),
    (
        "/proc/device-tree/compatible",
        "device_tree.compatible",
        false,
    ),
];

pub fn detect(host: &impl HostServices) -> PlatformInfo {
    let mut detected_model = None;
    let mut nvidia_signal = None;

    for (path, source, use_as_model) in DETECTION_PATHS {
        let Ok(raw) = host.read_to_string(Path::new(path)) else {
            continue;
        };
        let value = normalize_detection_value(&raw);
        if value.is_empty() {
            continue;
        }

        if use_as_model && detected_model.is_none() {
            detected_model = Some(value.clone());
        }

        if is_gb10_signature(&value) {
            let model = if use_as_model {
                Some(value)
            } else {
                detected_model.clone().or(Some(value))
            };
            return PlatformInfo::gb10(model, source);
        }

        if nvidia_signal.is_none() && is_nvidia_signature(&value) {
            let model = if use_as_model {
                Some(value.clone())
            } else {
                detected_model.clone().or(Some(value.clone()))
            };
            nvidia_signal = Some((model, source));
        }
    }

    if let Some((model, source)) = nvidia_signal {
        return PlatformInfo::nvidia_linux(model, source);
    }

    if host.exists(Path::new("/proc/driver/nvidia/version")) {
        return PlatformInfo::nvidia_linux(detected_model, "proc.nvidia.version");
    }

    if host.command_path("nvidia-smi").is_some() {
        return PlatformInfo::nvidia_linux(detected_model, "command.nvidia-smi");
    }

    PlatformInfo::generic_linux(detected_model)
}

fn normalize_detection_value(raw: &str) -> String {
    raw.replace('\0', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn is_gb10_signature(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("gb10") || lower.contains("dgx spark")
}

fn is_nvidia_signature(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("nvidia")
}

#[cfg(test)]
mod tests {
    use super::detect;
    use crate::features::system_updater::domain::platform::PlatformClass;
    use crate::features::system_updater::testing::FakeHost;
    use std::path::PathBuf;

    #[test]
    fn detects_gb10_from_dmi_product_name() {
        let mut host = FakeHost::new();
        host.add_file(
            "/sys/class/dmi/id/product_name",
            "NVIDIA DGX Spark GB10 Developer System\n",
        );

        let platform = detect(&host);
        assert_eq!(platform.class, PlatformClass::Gb10);
        assert_eq!(
            platform.model.as_deref(),
            Some("NVIDIA DGX Spark GB10 Developer System")
        );
        assert_eq!(
            platform.detection_source.as_deref(),
            Some("dmi.product_name")
        );
    }

    #[test]
    fn detects_gb10_from_device_tree_signature() {
        let mut host = FakeHost::new();
        host.add_file("/proc/device-tree/model", "NVIDIA Embedded Platform");
        host.add_file(
            "/proc/device-tree/compatible",
            "nvidia,tegra\0nvidia,gb10\0",
        );

        let platform = detect(&host);
        assert_eq!(platform.class, PlatformClass::Gb10);
        assert_eq!(platform.model.as_deref(), Some("NVIDIA Embedded Platform"));
        assert_eq!(
            platform.detection_source.as_deref(),
            Some("device_tree.compatible")
        );
    }

    #[test]
    fn falls_back_to_generic_linux_when_no_gb10_markers_exist() {
        let mut host = FakeHost::new();
        host.add_file("/sys/class/dmi/id/product_name", "Dell PowerEdge R760");

        let platform = detect(&host);
        assert_eq!(platform.class, PlatformClass::GenericLinux);
        assert_eq!(platform.model.as_deref(), Some("Dell PowerEdge R760"));
        assert!(platform.detection_source.is_none());
    }

    #[test]
    fn detects_generic_nvidia_linux_from_driver_command() {
        let mut host = FakeHost::new();
        host.add_file("/sys/class/dmi/id/product_name", "Dell PowerEdge R760");
        host.add_command("nvidia-smi", PathBuf::from("/usr/bin/nvidia-smi"), false);

        let platform = detect(&host);
        assert_eq!(platform.class, PlatformClass::NvidiaLinux);
        assert_eq!(platform.model.as_deref(), Some("Dell PowerEdge R760"));
        assert_eq!(
            platform.detection_source.as_deref(),
            Some("command.nvidia-smi")
        );
    }

    #[test]
    fn detects_generic_nvidia_linux_from_dmi_signature() {
        let mut host = FakeHost::new();
        host.add_file("/sys/class/dmi/id/product_name", "NVIDIA HGX Server");

        let platform = detect(&host);
        assert_eq!(platform.class, PlatformClass::NvidiaLinux);
        assert_eq!(platform.model.as_deref(), Some("NVIDIA HGX Server"));
        assert_eq!(
            platform.detection_source.as_deref(),
            Some("dmi.product_name")
        );
    }
}
