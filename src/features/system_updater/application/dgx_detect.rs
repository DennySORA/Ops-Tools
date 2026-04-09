use crate::features::system_updater::domain::config::DgxConfig;
use crate::features::system_updater::ports::{CommandExecutor, HostServices};

/// Auto-detect DGX hardware configuration from the running system.
///
/// Fallback chain per field: detected value → config file value → empty.
/// Empty fields are safe because the DGX steps only run on GB10 hosts,
/// and if detection fails the user can still set values in update.toml.
pub fn detect_and_merge<H, E>(host: &H, executor: &E, config: &mut DgxConfig)
where
    H: HostServices,
    E: CommandExecutor,
{
    if let Some(value) = detect_cuda_major(executor) {
        config.cuda_major = value;
    }
    if let Some(value) = detect_cuda_arch(executor) {
        config.cuda_arch = value;
    }
    if let Some(value) = detect_installed_package(executor, "nvidia-driver-") {
        config.driver_pkg = value;
    }
    if let Some(value) = detect_installed_package(executor, "linux-modules-nvidia-") {
        config.modules_pkg = value;
    }
    if let Some(value) = detect_installed_package(executor, "linux-image-nvidia-") {
        config.kernel_meta = value;
    }
    if config.headers_meta.is_empty() {
        if !config.kernel_meta.is_empty() {
            config.headers_meta = config.kernel_meta.replace("linux-image-", "linux-headers-");
        }
    } else if let Some(value) = detect_installed_package(executor, "linux-headers-nvidia-") {
        config.headers_meta = value;
    }

    if has_any_detected(config) {
        println!("  DGX auto-detect:");
        if !config.cuda_major.is_empty() {
            println!("    CUDA toolkit: cuda-toolkit-{}", config.cuda_major);
        }
        if !config.cuda_arch.is_empty() {
            println!("    GPU arch:     sm_{}", config.cuda_arch);
        }
        if !config.driver_pkg.is_empty() {
            println!("    Driver:       {}", config.driver_pkg);
        }
        if !config.kernel_meta.is_empty() {
            println!("    Kernel meta:  {}", config.kernel_meta);
        }
    }

    let _ = (host,); // suppress unused warning; host reserved for future probes
}

/// Detect CUDA major version from nvcc (e.g. "13-0" from "release 13.0").
fn detect_cuda_major<E: CommandExecutor>(executor: &E) -> Option<String> {
    let output = executor
        .capture(
            &crate::features::system_updater::domain::command::CommandSpec::new(
                "nvcc",
                ["--version"],
            ),
        )
        .ok()?;

    // nvcc output contains a line like: "Cuda compilation tools, release 13.0, V13.0.76"
    for line in output.lines() {
        if let Some(pos) = line.find("release ") {
            let version_part = &line[pos + "release ".len()..];
            let version = version_part.split(',').next()?.trim();
            // "13.0" → "13-0"
            let major_minor = version.replace('.', "-");
            return Some(major_minor);
        }
    }
    None
}

/// Detect GPU compute capability from nvidia-smi (e.g. "121" from "12.1").
fn detect_cuda_arch<E: CommandExecutor>(executor: &E) -> Option<String> {
    let output = executor
        .capture(
            &crate::features::system_updater::domain::command::CommandSpec::new(
                "nvidia-smi",
                ["--query-gpu=compute_cap", "--format=csv,noheader,nounits"],
            ),
        )
        .ok()?;

    // Output: "12.1\n" or "12.1\n12.1\n" (multi-GPU)
    let first_line = output.lines().next()?.trim();
    // "12.1" → "121"
    let arch = first_line.replace('.', "");
    if arch.chars().all(|c| c.is_ascii_digit()) && !arch.is_empty() {
        Some(arch)
    } else {
        None
    }
}

/// Detect an installed package by prefix from dpkg (e.g. "nvidia-driver-580-open").
fn detect_installed_package<E: CommandExecutor>(executor: &E, prefix: &str) -> Option<String> {
    let pattern = format!("{prefix}*");
    let output = executor
        .capture(
            &crate::features::system_updater::domain::command::CommandSpec::new(
                "dpkg",
                ["-l", pattern.as_str()],
            ),
        )
        .ok()?;

    // dpkg -l output: "ii  nvidia-driver-580-open  580.xxx  ..."
    // Find lines starting with "ii " and extract the package name.
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("ii ") || trimmed.starts_with("ii\t") {
            let fields: Vec<&str> = trimmed.split_whitespace().collect();
            if fields.len() >= 2 && fields[1].starts_with(prefix) {
                return Some(fields[1].to_string());
            }
        }
    }
    None
}

fn has_any_detected(config: &DgxConfig) -> bool {
    !config.cuda_major.is_empty()
        || !config.cuda_arch.is_empty()
        || !config.driver_pkg.is_empty()
        || !config.kernel_meta.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::system_updater::domain::error::InfrastructureError;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost};

    fn spawn_err(cmd: &str) -> InfrastructureError {
        InfrastructureError::CommandSpawn {
            code: "TEST",
            command: cmd.to_string(),
            detail: "not available".to_string(),
        }
    }

    #[test]
    fn detects_cuda_major_from_nvcc() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        executor.push_capture_ok(
            "nvcc --version",
            "nvcc: NVIDIA (R) Cuda compiler driver\n\
             Cuda compilation tools, release 13.0, V13.0.76\n\
             Build cuda_13.0.r13.0/compiler.123456_0",
        );

        let mut config = DgxConfig::default();
        detect_and_merge(&host, &executor, &mut config);
        assert_eq!(config.cuda_major, "13-0");
    }

    #[test]
    fn detects_cuda_arch_from_nvidia_smi() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        executor.push_capture_error("nvcc --version", spawn_err("nvcc"));
        executor.push_capture_ok(
            "nvidia-smi --query-gpu=compute_cap --format=csv,noheader,nounits",
            "12.1\n",
        );
        executor.push_capture_error("dpkg -l nvidia-driver-*", spawn_err("dpkg"));
        executor.push_capture_error("dpkg -l linux-modules-nvidia-*", spawn_err("dpkg"));
        executor.push_capture_error("dpkg -l linux-image-nvidia-*", spawn_err("dpkg"));
        executor.push_capture_error("dpkg -l linux-headers-nvidia-*", spawn_err("dpkg"));

        let mut config = DgxConfig::default();
        detect_and_merge(&host, &executor, &mut config);
        assert_eq!(config.cuda_arch, "121");
    }

    #[test]
    fn detects_driver_package_from_dpkg() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        executor.push_capture_error("nvcc --version", spawn_err("nvcc"));
        executor.push_capture_error(
            "nvidia-smi --query-gpu=compute_cap --format=csv,noheader,nounits",
            spawn_err("nvidia-smi"),
        );
        executor.push_capture_ok(
            "dpkg -l nvidia-driver-*",
            "ii  nvidia-driver-580-open  580.87.02-0ubuntu1  arm64  NVIDIA driver\n",
        );
        executor.push_capture_error("dpkg -l linux-modules-nvidia-*", spawn_err("dpkg"));
        executor.push_capture_error("dpkg -l linux-image-nvidia-*", spawn_err("dpkg"));
        executor.push_capture_error("dpkg -l linux-headers-nvidia-*", spawn_err("dpkg"));

        let mut config = DgxConfig::default();
        detect_and_merge(&host, &executor, &mut config);
        assert_eq!(config.driver_pkg, "nvidia-driver-580-open");
    }

    #[test]
    fn derives_headers_from_kernel_meta() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        executor.push_capture_error("nvcc --version", spawn_err("nvcc"));
        executor.push_capture_error(
            "nvidia-smi --query-gpu=compute_cap --format=csv,noheader,nounits",
            spawn_err("nvidia-smi"),
        );
        executor.push_capture_error("dpkg -l nvidia-driver-*", spawn_err("dpkg"));
        executor.push_capture_error("dpkg -l linux-modules-nvidia-*", spawn_err("dpkg"));
        executor.push_capture_ok(
            "dpkg -l linux-image-nvidia-*",
            "ii  linux-image-nvidia-hwe-24.04  6.8.0  arm64  kernel\n",
        );
        // headers derived from kernel_meta, no dpkg call needed

        let mut config = DgxConfig::default();
        detect_and_merge(&host, &executor, &mut config);
        assert_eq!(config.kernel_meta, "linux-image-nvidia-hwe-24.04");
        assert_eq!(config.headers_meta, "linux-headers-nvidia-hwe-24.04");
    }
}
