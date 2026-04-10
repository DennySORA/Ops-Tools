use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::platform::{OperatingSystem, PlatformInfo};
use crate::features::system_updater::ports::{CommandExecutor, HostServices};
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

pub fn detect(host: &impl HostServices, executor: &impl CommandExecutor) -> PlatformInfo {
    let os = detect_operating_system(executor);
    let arch = capture_trimmed(executor, "uname", ["-m"]);

    match os {
        OperatingSystem::Macos => detect_macos(executor, arch),
        OperatingSystem::Linux => detect_linux(host, arch),
    }
}

fn detect_operating_system(executor: &impl CommandExecutor) -> OperatingSystem {
    match capture_trimmed(executor, "uname", ["-s"])
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "darwin" => OperatingSystem::Macos,
        _ => OperatingSystem::Linux,
    }
}

fn detect_macos(executor: &impl CommandExecutor, arch: Option<String>) -> PlatformInfo {
    let model = capture_trimmed(executor, "sysctl", ["-n", "hw.model"]);
    let version = capture_trimmed(executor, "sw_vers", ["-productVersion"]);
    PlatformInfo::macos(model, version, arch)
}

fn detect_linux(host: &impl HostServices, arch: Option<String>) -> PlatformInfo {
    let version = linux_pretty_name(host);
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
            return PlatformInfo::gb10(model, version, arch, source);
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
        return PlatformInfo::nvidia_linux(model, version, arch, source);
    }

    if host.exists(Path::new("/proc/driver/nvidia/version")) {
        return PlatformInfo::nvidia_linux(detected_model, version, arch, "proc.nvidia.version");
    }

    if host.command_path("nvidia-smi").is_some() {
        return PlatformInfo::nvidia_linux(detected_model, version, arch, "command.nvidia-smi");
    }

    PlatformInfo::generic_linux(detected_model, version, arch)
}

fn linux_pretty_name(host: &impl HostServices) -> Option<String> {
    let content = host.read_to_string(Path::new("/etc/os-release")).ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|value| value.trim_matches('"').to_string())
}

fn capture_trimmed(
    executor: &impl CommandExecutor,
    program: &str,
    args: impl IntoIterator<Item = &'static str>,
) -> Option<String> {
    executor
        .capture(&CommandSpec::new(program, args))
        .ok()
        .map(|output| output.trim().to_string())
        .filter(|output| !output.is_empty())
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
    use crate::features::system_updater::domain::platform::{OperatingSystem, PlatformClass};
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost};
    use std::path::PathBuf;

    #[test]
    fn detects_macos_from_uname_and_sysctl() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("uname -s", "Darwin\n");
        executor.push_capture_ok("uname -m", "arm64\n");
        executor.push_capture_ok("sysctl -n hw.model", "Mac14,15\n");
        executor.push_capture_ok("sw_vers -productVersion", "15.4\n");

        let platform = detect(&host, &executor);
        assert_eq!(platform.os, OperatingSystem::Macos);
        assert_eq!(platform.class, PlatformClass::Macos);
        assert_eq!(platform.model.as_deref(), Some("Mac14,15"));
        assert_eq!(platform.version.as_deref(), Some("15.4"));
        assert_eq!(platform.arch.as_deref(), Some("arm64"));
    }

    #[test]
    fn detects_gb10_from_dmi_product_name() {
        let mut host = FakeHost::new();
        host.add_file("/etc/os-release", "PRETTY_NAME=\"Ubuntu 24.04.2 LTS\"\n");
        host.add_file(
            "/sys/class/dmi/id/product_name",
            "NVIDIA DGX Spark GB10 Developer System\n",
        );

        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("uname -s", "Linux\n");
        executor.push_capture_ok("uname -m", "aarch64\n");

        let platform = detect(&host, &executor);
        assert_eq!(platform.os, OperatingSystem::Linux);
        assert_eq!(platform.class, PlatformClass::Gb10);
        assert_eq!(
            platform.model.as_deref(),
            Some("NVIDIA DGX Spark GB10 Developer System")
        );
        assert_eq!(
            platform.detection_source.as_deref(),
            Some("dmi.product_name")
        );
        assert_eq!(platform.version.as_deref(), Some("Ubuntu 24.04.2 LTS"));
        assert_eq!(platform.arch.as_deref(), Some("aarch64"));
    }

    #[test]
    fn detects_gb10_from_device_tree_signature() {
        let mut host = FakeHost::new();
        host.add_file("/proc/device-tree/model", "NVIDIA Embedded Platform");
        host.add_file(
            "/proc/device-tree/compatible",
            "nvidia,tegra\0nvidia,gb10\0",
        );

        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("uname -s", "Linux\n");
        executor.push_capture_ok("uname -m", "aarch64\n");

        let platform = detect(&host, &executor);
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

        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("uname -s", "Linux\n");
        executor.push_capture_ok("uname -m", "x86_64\n");

        let platform = detect(&host, &executor);
        assert_eq!(platform.class, PlatformClass::GenericLinux);
        assert_eq!(platform.model.as_deref(), Some("Dell PowerEdge R760"));
        assert!(platform.detection_source.is_none());
    }

    #[test]
    fn detects_generic_nvidia_linux_from_driver_command() {
        let mut host = FakeHost::new();
        host.add_file("/sys/class/dmi/id/product_name", "Dell PowerEdge R760");
        host.add_command("nvidia-smi", PathBuf::from("/usr/bin/nvidia-smi"), false);

        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("uname -s", "Linux\n");
        executor.push_capture_ok("uname -m", "x86_64\n");

        let platform = detect(&host, &executor);
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

        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("uname -s", "Linux\n");
        executor.push_capture_ok("uname -m", "x86_64\n");

        let platform = detect(&host, &executor);
        assert_eq!(platform.class, PlatformClass::NvidiaLinux);
        assert_eq!(platform.model.as_deref(), Some("NVIDIA HGX Server"));
        assert_eq!(
            platform.detection_source.as_deref(),
            Some("dmi.product_name")
        );
    }
}
