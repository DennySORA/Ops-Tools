use crate::features::system_updater::application::maintenance::MaintenanceContext;
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::{AppResult, DomainError, InfrastructureError};
use crate::features::system_updater::domain::report::StepOutcome;
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};
use serde::Deserialize;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

const CUDA_BLOCK_START: &str = "# >>> CUDA Toolkit (Ops-Tools) >>>";
const CUDA_BLOCK_END: &str = "# <<< CUDA Toolkit (Ops-Tools) <<<";
const LEGACY_DGX_BLOCK_START: &str = "# >>> CUDA DGX Spark >>>";
const LEGACY_DGX_BLOCK_END: &str = "# <<< CUDA DGX Spark <<<";

#[derive(Clone, Debug, PartialEq, Eq)]
struct CudaRunfile {
    version: String,
    driver_version: String,
    cuda_component_version: String,
    repo_arch: &'static str,
    url: String,
    filename: String,
}

impl CudaRunfile {
    fn major_minor(&self) -> String {
        self.version
            .split('.')
            .take(2)
            .collect::<Vec<_>>()
            .join(".")
    }
}

#[derive(Debug, Deserialize)]
struct RunfileMetadata {
    cuda: MetadataComponent,
    nvidia_driver: MetadataComponent,
}

#[derive(Debug, Deserialize)]
struct MetadataComponent {
    version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VersionParts(u64, u64, u64);

impl VersionParts {
    fn parse(value: &str) -> Option<Self> {
        let mut parts = value.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next().unwrap_or("0").parse().ok()?;
        let patch = parts.next().unwrap_or("0").parse().ok()?;
        Some(Self(major, minor, patch))
    }
}

impl Ord for VersionParts {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0, self.1, self.2).cmp(&(other.0, other.1, other.2))
    }
}

impl PartialOrd for VersionParts {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn upgrade_toolkit_and_configure<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if !context.config.cuda.enabled {
        return Ok(StepOutcome::skipped("CUDA toolkit maintenance is disabled"));
    }

    if !is_cuda_host(context) {
        return Ok(StepOutcome::skipped(format!(
            "CUDA toolkit maintenance requires NVIDIA CUDA signals; detected {}",
            context.platform.summary()
        )));
    }

    if context.platform.supports_gb10_tuning() {
        return upgrade_dgx_spark_toolkit_and_configure(context);
    }

    if context.executor.is_dry_run() {
        println!("  [dry-run] would detect the latest NVIDIA CUDA runfile.");
        println!("  [dry-run] would download and run the CUDA toolkit installer if needed.");
        if context.config.cuda.configure_zshrc {
            configure_cuda_zshrc(context, None)?;
        }
        return Ok(StepOutcome::dry_run(
            "CUDA toolkit upgrade and shell configuration previewed",
        ));
    }

    let runfile = resolve_latest_runfile(context)?;
    println!(
        "  Latest CUDA runfile: CUDA {} (driver {}, {})",
        runfile.version, runfile.driver_version, runfile.repo_arch
    );
    println!("  Download URL: {}", runfile.url);

    let installed_component = installed_cuda_component_version(context, &runfile);
    if installed_component.as_deref() == Some(runfile.cuda_component_version.as_str()) {
        println!(
            "  CUDA SDK component {} already installed.",
            runfile.cuda_component_version
        );
    } else {
        if let Some(current) = installed_component {
            println!("  Installed CUDA SDK component: {current}");
        }
        let installer_path = context.config.cuda.installer_dir.join(&runfile.filename);
        download_installer(context, &runfile, &installer_path)?;
        install_runfile(context, &installer_path)?;
        update_cuda_symlink(context, &runfile)?;
    }

    if context.config.cuda.configure_zshrc {
        configure_cuda_zshrc(context, Some(&runfile))?;
    }

    Ok(StepOutcome::ok())
}

fn is_cuda_host<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> bool
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    context.platform.expects_nvidia_tooling()
        || context.host.command_path("nvcc").is_some()
        || context.host.exists(Path::new("/usr/local/cuda"))
        || is_wsl_cuda_host(context.host)
}

fn upgrade_dgx_spark_toolkit_and_configure<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.executor.is_dry_run() {
        let selection = resolve_dgx_spark_cuda_toolkit_package(context);
        println!("  [dry-run] DGX Spark detected; would use the official DGX OS APT CUDA path.");
        println!(
            "  [dry-run] selected CUDA toolkit package: {} ({})",
            selection.package, selection.source
        );
        println!("  [dry-run] would run: sudo apt update");
        println!("  [dry-run] would run: sudo apt install {}", selection.package);
        if context.config.cuda.configure_zshrc {
            configure_cuda_zshrc(context, None)?;
        }
        return Ok(StepOutcome::dry_run(format!(
            "DGX Spark CUDA toolkit APT upgrade previewed ({})",
            selection.package
        )));
    }

    println!("  DGX Spark detected; using the official DGX OS APT CUDA path.");
    context
        .executor
        .run(&CommandSpec::new("apt", ["update"]).with_sudo())?;

    let selection = resolve_dgx_spark_cuda_toolkit_package(context);
    println!(
        "  CUDA toolkit package: {} ({})",
        selection.package, selection.source
    );
    context.executor.run(
        &CommandSpec::new("apt", ["-y", "install", selection.package.as_str()]).with_sudo(),
    )?;

    if context.config.cuda.configure_zshrc {
        configure_cuda_zshrc(context, None)?;
    }

    Ok(StepOutcome::ok())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DgxSparkCudaToolkitSelection {
    package: String,
    source: &'static str,
}

fn resolve_dgx_spark_cuda_toolkit_package<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> DgxSparkCudaToolkitSelection
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if let Some(package) = configured_dgx_spark_cuda_toolkit_package(&context.config.dgx.cuda_major)
    {
        return DgxSparkCudaToolkitSelection {
            package,
            source: "configured/detected dgx.cuda_major",
        };
    }

    if let Some(package) =
        apt_cuda_toolkit_package(context, ["list", "--installed", "cuda-toolkit-*"])
    {
        return DgxSparkCudaToolkitSelection {
            package,
            source: "installed APT package",
        };
    }

    if let Some(package) = apt_cuda_toolkit_package(context, ["list", "cuda-toolkit-*"]) {
        return DgxSparkCudaToolkitSelection {
            package,
            source: "latest APT package",
        };
    }

    DgxSparkCudaToolkitSelection {
        package: "cuda-toolkit-13-0".to_string(),
        source: "DGX Spark fallback",
    }
}

fn configured_dgx_spark_cuda_toolkit_package(cuda_major: &str) -> Option<String> {
    let value = cuda_major.trim();
    if value.is_empty() {
        None
    } else if value.starts_with("cuda-toolkit-") {
        Some(value.to_string())
    } else {
        Some(format!("cuda-toolkit-{value}"))
    }
}

fn apt_cuda_toolkit_package<H, E, R, const N: usize>(
    context: &MaintenanceContext<'_, H, E, R>,
    args: [&'static str; N],
) -> Option<String>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    context
        .executor
        .capture(&CommandSpec::new("apt", args))
        .ok()
        .and_then(|output| latest_cuda_toolkit_package(&output))
}

fn latest_cuda_toolkit_package(apt_list_output: &str) -> Option<String> {
    apt_list_output
        .lines()
        .filter_map(parse_cuda_toolkit_package_line)
        .max_by_key(|(_, key)| *key)
        .map(|(package, _)| package)
}

fn parse_cuda_toolkit_package_line(line: &str) -> Option<(String, (u64, u64))> {
    let package = line.split('/').next()?.trim();
    let key = cuda_toolkit_package_version_key(package)?;
    Some((package.to_string(), key))
}

fn cuda_toolkit_package_version_key(package: &str) -> Option<(u64, u64)> {
    let rest = package.strip_prefix("cuda-toolkit-")?;
    let mut parts = rest.split('-');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor))
}

fn is_wsl_cuda_host(host: &impl HostServices) -> bool {
    is_wsl(host)
        && (host.exists(Path::new("/usr/lib/wsl/lib/nvidia-smi"))
            || host.exists(Path::new("/usr/lib/wsl/lib/libcuda.so"))
            || host.command_path("nvidia-smi").is_some())
}

fn is_wsl(host: &impl HostServices) -> bool {
    ["/proc/sys/kernel/osrelease", "/proc/version"]
        .into_iter()
        .any(|path| {
            host.read_to_string(Path::new(path))
                .map(|content| content.to_ascii_lowercase().contains("microsoft"))
                .unwrap_or(false)
        })
}

fn resolve_latest_runfile<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<CudaRunfile>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let (repo_arch, filename_arch_suffix) = runfile_arch(context)?;
    let index_base = context
        .config
        .cuda
        .runfile_index_base_url
        .trim_end_matches('/');
    let index_url = format!("{index_base}/{repo_arch}");

    let version = match &context.config.cuda.version {
        Some(version) => version.clone(),
        None => {
            let index = fetch_text(context, &index_url)?;
            discover_latest_runfile_version(&index).ok_or_else(|| {
                DomainError::validation(
                    "DOMAIN_CUDA_VERSION_DETECT",
                    format!("unable to find version_*.json entries in {index_url}"),
                )
            })?
        }
    };

    let metadata_url = format!("{index_url}/version_{version}.json");
    let metadata_text = fetch_text(context, &metadata_url)?;
    let metadata: RunfileMetadata = serde_json::from_str(&metadata_text).map_err(|err| {
        InfrastructureError::serialization("INFRA_CUDA_METADATA_PARSE", err.to_string())
    })?;
    let driver_version = context
        .config
        .cuda
        .driver_version
        .clone()
        .unwrap_or(metadata.nvidia_driver.version);
    let filename = format!("cuda_{version}_{driver_version}_linux{filename_arch_suffix}.run");
    let download_base = context.config.cuda.download_base_url.trim_end_matches('/');
    let url = format!("{download_base}/{version}/local_installers/{filename}");

    Ok(CudaRunfile {
        version,
        driver_version,
        cuda_component_version: metadata.cuda.version,
        repo_arch,
        url,
        filename,
    })
}

fn runfile_arch<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
) -> AppResult<(&'static str, &'static str)>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let arch = context
        .platform
        .arch
        .clone()
        .or_else(|| {
            context
                .executor
                .capture(&CommandSpec::new("uname", ["-m"]))
                .ok()
                .map(|value| value.trim().to_string())
        })
        .unwrap_or_default()
        .to_ascii_lowercase();

    match arch.as_str() {
        "x86_64" | "amd64" => Ok(("x86_64", "")),
        "aarch64" | "arm64" => Ok(("sbsa", "_sbsa")),
        _ => Err(DomainError::validation(
            "DOMAIN_CUDA_ARCH_UNSUPPORTED",
            format!("unsupported CUDA runfile architecture: {arch}"),
        )
        .into()),
    }
}

fn fetch_text<H, E, R>(context: &MaintenanceContext<'_, H, E, R>, url: &str) -> AppResult<String>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("curl").is_some() {
        return Ok(context
            .executor
            .capture(&CommandSpec::new("curl", ["-fsSL", "--retry", "2", url]))?);
    }
    if context.host.command_path("wget").is_some() {
        return Ok(context
            .executor
            .capture(&CommandSpec::new("wget", ["-qO-", url]))?);
    }

    Err(DomainError::validation(
        "DOMAIN_CUDA_DOWNLOAD_TOOL",
        "curl or wget is required to resolve CUDA downloads",
    )
    .into())
}

fn discover_latest_runfile_version(index_html: &str) -> Option<String> {
    index_html
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')))
        .filter_map(|token| {
            token
                .strip_prefix("version_")
                .and_then(|rest| rest.strip_suffix(".json"))
        })
        .filter_map(|version| VersionParts::parse(version).map(|parts| (version, parts)))
        .max_by_key(|(_, parts)| *parts)
        .map(|(version, _)| version.to_string())
}

fn installed_cuda_component_version<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    runfile: &CudaRunfile,
) -> Option<String>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let current_link = PathBuf::from("/usr/local/cuda/version.json");
    let target = PathBuf::from(format!(
        "/usr/local/cuda-{}/version.json",
        runfile.major_minor()
    ));

    [current_link, target].into_iter().find_map(|path| {
        context
            .host
            .read_to_string(&path)
            .ok()
            .and_then(|content| parse_cuda_component_version(&content))
    })
}

fn parse_cuda_component_version(content: &str) -> Option<String> {
    serde_json::from_str::<RunfileMetadata>(content)
        .ok()
        .map(|metadata| metadata.cuda.version)
}

fn download_installer<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    runfile: &CudaRunfile,
    installer_path: &Path,
) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    context
        .host
        .create_dir_all(&context.config.cuda.installer_dir)?;

    let installer = shell_quote(&installer_path.display().to_string());
    let url = shell_quote(&runfile.url);
    let script = if context.host.command_path("wget").is_some() {
        format!(
            "set -e\nif [ ! -s {installer} ]; then wget -O {installer} {url}; else echo \"Using cached CUDA installer: {path}\"; fi",
            path = installer_path.display(),
        )
    } else {
        format!(
            "set -e\nif [ ! -s {installer} ]; then curl -fL --retry 2 -o {installer} {url}; else echo \"Using cached CUDA installer: {path}\"; fi",
            path = installer_path.display(),
        )
    };

    context
        .executor
        .run(&CommandSpec::new("bash", ["-lc", &script]))?;
    Ok(())
}

fn install_runfile<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    installer_path: &Path,
) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    context.executor.run(
        &CommandSpec::new(
            "sh",
            [
                installer_path.display().to_string(),
                "--silent".to_string(),
                "--toolkit".to_string(),
                "--override".to_string(),
            ],
        )
        .with_sudo(),
    )?;
    Ok(())
}

fn update_cuda_symlink<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    runfile: &CudaRunfile,
) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let target = format!("/usr/local/cuda-{}", runfile.major_minor());
    let target_quoted = shell_quote(&target);
    let script = format!(
        "set -e\nif [ -d {target_quoted} ]; then ln -sfn {target_quoted} /usr/local/cuda; else echo \"CUDA target directory not found: {target}\"; fi"
    );
    context
        .executor
        .run(&CommandSpec::new("bash", ["-lc", &script]).with_sudo())?;
    Ok(())
}

fn configure_cuda_zshrc<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    runfile: Option<&CudaRunfile>,
) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let home = context
        .host
        .var("HOME")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            DomainError::validation(
                "DOMAIN_CUDA_HOME_MISSING",
                "HOME is required to update .zshrc",
            )
        })?;
    let zshrc = PathBuf::from(&home).join(".zshrc");
    let backup = PathBuf::from(&home).join(".zshrc.bak");
    let temp = PathBuf::from(&home).join(".zshrc.tmp");
    let cuda_block = build_cuda_shell_block(context.config.dgx.cuda_arch.as_str(), runfile);

    if context.executor.is_dry_run() {
        println!(
            "  [dry-run] would update CUDA shell block in {}",
            zshrc.display()
        );
        return Ok(());
    }

    let existing = if context.host.exists(&zshrc) {
        context.host.read_to_string(&zshrc)?
    } else {
        String::new()
    };

    if context.host.exists(&zshrc) {
        context.host.copy_file(&zshrc, &backup)?;
        println!("  Backed up .zshrc -> .zshrc.bak");
    }

    let new_content = rewrite_cuda_shell_content(
        &existing,
        &cuda_block,
        context.config.cuda.remove_legacy_zshrc_entries,
    );
    context.host.write_string(&temp, &new_content)?;
    context.host.rename(&temp, &zshrc)?;
    println!("  CUDA environment configured in .zshrc");
    Ok(())
}

fn build_cuda_shell_block(cuda_arch: &str, runfile: Option<&CudaRunfile>) -> String {
    let mut lines = vec![
        CUDA_BLOCK_START.to_string(),
        "# Managed by Ops-Tools System Updater.".to_string(),
    ];

    if let Some(runfile) = runfile {
        lines.push(format!("# CUDA toolkit: {}", runfile.version));
    }

    lines.extend([
        r#"export CUDA_HOME="/usr/local/cuda""#.to_string(),
        r#"export CUDA_PATH="$CUDA_HOME""#.to_string(),
        r#"export PATH="$CUDA_HOME/bin${PATH:+:$PATH}""#.to_string(),
        r#"export LD_LIBRARY_PATH="$CUDA_HOME/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}""#
            .to_string(),
    ]);

    if !cuda_arch.trim().is_empty() {
        lines.push(format!(r#"export CUDAARCHS="{}-real""#, cuda_arch.trim()));
        lines.push(format!(
            r#"export CMAKE_CUDA_ARCHITECTURES="{}-real""#,
            cuda_arch.trim()
        ));
    }

    lines.push(CUDA_BLOCK_END.to_string());
    lines.join("\n")
}

fn rewrite_cuda_shell_content(
    existing: &str,
    replacement: &str,
    remove_legacy_entries: bool,
) -> String {
    let without_blocks = remove_cuda_blocks(existing);
    let lines: Vec<&str> = without_blocks
        .lines()
        .filter(|line| !remove_legacy_entries || !is_legacy_cuda_export(line))
        .collect();
    let cleaned = lines.join("\n");

    if cleaned.trim().is_empty() {
        format!("{replacement}\n")
    } else {
        format!("{}\n\n{}\n", cleaned.trim_end(), replacement)
    }
}

fn remove_cuda_blocks(existing: &str) -> String {
    let markers = [
        (CUDA_BLOCK_START, CUDA_BLOCK_END),
        (LEGACY_DGX_BLOCK_START, LEGACY_DGX_BLOCK_END),
    ];
    let mut retained = Vec::new();
    let mut active_end: Option<&str> = None;

    for line in existing.lines() {
        if let Some(end) = active_end {
            if line.contains(end) {
                active_end = None;
            }
            continue;
        }

        if let Some((_, end)) = markers.iter().find(|(start, _)| line.contains(start)) {
            active_end = Some(*end);
            continue;
        }

        retained.push(line);
    }

    retained.join("\n")
}

fn is_legacy_cuda_export(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(rest) = trimmed.strip_prefix("export ") else {
        return false;
    };
    let lower = rest.trim_start().to_ascii_lowercase();
    if !lower.contains("cuda") {
        return false;
    }

    [
        "cuda_home=",
        "cuda_path=",
        "cuda_root=",
        "cudaarchs=",
        "cmake_cuda_architectures=",
        "path=",
        "ld_library_path=",
        "library_path=",
        "cpath=",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::system_updater::application::maintenance::MaintenanceContext;
    use crate::features::system_updater::domain::config::Config;
    use crate::features::system_updater::domain::platform::PlatformInfo;
    use crate::features::system_updater::domain::report::StepStatus;
    use crate::features::system_updater::ports::FileSystem;
    use crate::features::system_updater::testing::{FakeExecutor, FakeHost, FakeReporter};
    use std::path::PathBuf;

    #[test]
    fn discovers_latest_runfile_version_from_index() {
        let index = r#"
            <a href="version_13.1.1.json">version_13.1.1.json</a>
            <a href="version_13.2.0.json">version_13.2.0.json</a>
            <a href="version_13.2.1.json">version_13.2.1.json</a>
        "#;

        assert_eq!(
            discover_latest_runfile_version(index).as_deref(),
            Some("13.2.1")
        );
    }

    #[test]
    fn rewrites_cuda_shell_content_and_removes_legacy_exports() {
        let replacement = build_cuda_shell_block("121", None);
        let existing = r#"# user config
export PATH="/usr/local/cuda-12.8/bin:$PATH"
export LD_LIBRARY_PATH="/usr/local/cuda-12.8/lib64:$LD_LIBRARY_PATH"
# >>> CUDA DGX Spark >>>
export CUDA_HOME="/usr/local/cuda"
# <<< CUDA DGX Spark <<<
export EDITOR="vim"
"#;

        let rewritten = rewrite_cuda_shell_content(existing, &replacement, true);

        assert!(rewritten.contains("export EDITOR=\"vim\""));
        assert!(!rewritten.contains("cuda-12.8"));
        assert!(!rewritten.contains("CUDA DGX Spark"));
        assert!(rewritten.contains(CUDA_BLOCK_START));
        assert!(rewritten.contains("CUDAARCHS=\"121-real\""));
    }

    #[test]
    fn resolves_x86_64_runfile_from_nvidia_metadata() {
        let mut host = FakeHost::new();
        host.add_command("curl", PathBuf::from("/usr/bin/curl"), false);
        let executor = FakeExecutor::new(false);
        executor.push_capture_ok("curl -fsSL --retry 2 https://developer.download.nvidia.com/compute/cuda/repos/runfile/x86_64", "version_13.2.1.json\n");
        executor.push_capture_ok(
            "curl -fsSL --retry 2 https://developer.download.nvidia.com/compute/cuda/repos/runfile/x86_64/version_13.2.1.json",
            r#"{ "cuda": { "version": "13.2.20260407" }, "nvidia_driver": { "version": "595.58.03" } }"#,
        );
        let config = Config::default();
        let platform = PlatformInfo::nvidia_linux(
            Some("RTX Workstation".into()),
            None,
            Some("x86_64".into()),
            "test",
        );
        let reporter = FakeReporter::new();
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let runfile = resolve_latest_runfile(&context).expect("runfile");

        assert_eq!(runfile.version, "13.2.1");
        assert_eq!(runfile.driver_version, "595.58.03");
        assert_eq!(runfile.filename, "cuda_13.2.1_595.58.03_linux.run");
        assert_eq!(
            runfile.url,
            "https://developer.download.nvidia.com/compute/cuda/13.2.1/local_installers/cuda_13.2.1_595.58.03_linux.run"
        );
    }

    #[test]
    fn updates_zshrc_for_nvidia_linux() {
        let mut host = FakeHost::new();
        host.set_env("HOME", "/home/tester");
        host.add_file(
            "/home/tester/.zshrc",
            "export PATH=\"/usr/local/cuda-12.4/bin:$PATH\"\nexport EDITOR=\"vim\"\n",
        );
        let executor = FakeExecutor::new(false);
        let config = Config::default();
        let platform = PlatformInfo::nvidia_linux(
            Some("RTX Workstation".into()),
            None,
            Some("x86_64".into()),
            "test",
        );
        let reporter = FakeReporter::new();
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        configure_cuda_zshrc(&context, None).expect("configure zshrc");

        let zshrc = host
            .read_to_string(Path::new("/home/tester/.zshrc"))
            .expect("zshrc");
        assert!(zshrc.contains("export EDITOR=\"vim\""));
        assert!(!zshrc.contains("cuda-12.4"));
        assert!(zshrc.contains(CUDA_BLOCK_START));
    }

    #[test]
    fn skips_non_cuda_hosts() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let config = Config::default();
        let platform = PlatformInfo::generic_linux(Some("Generic VM".into()), None, None);
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = upgrade_toolkit_and_configure(&context).expect("cuda step");

        assert_eq!(outcome.status, StepStatus::Skipped);
        assert!(executor.commands().is_empty());
    }

    #[test]
    fn detects_wsl_cuda_signal() {
        let mut host = FakeHost::new();
        host.add_file(
            "/proc/sys/kernel/osrelease",
            "6.6.87.2-microsoft-standard-WSL2\n",
        );
        host.add_file("/usr/lib/wsl/lib/libcuda.so", "");

        assert!(is_wsl_cuda_host(&host));
    }

    #[test]
    fn dgx_spark_uses_apt_toolkit_package_instead_of_runfile() {
        let mut host = FakeHost::new();
        host.set_env("HOME", "/home/tester");
        let executor = FakeExecutor::new(false);
        let mut config = Config::default();
        config.dgx.cuda_major = "13-0".to_string();
        let platform = PlatformInfo::gb10(
            Some("NVIDIA DGX Spark GB10 Developer System".into()),
            None,
            Some("aarch64".into()),
            "test",
        );
        let reporter = FakeReporter::new();
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let outcome = upgrade_toolkit_and_configure(&context).expect("spark cuda apt");

        assert_eq!(outcome.status, StepStatus::Ok);
        let commands = executor.commands();
        assert!(commands.contains(&"sudo apt update".to_string()));
        assert!(commands.contains(&"sudo apt -y install cuda-toolkit-13-0".to_string()));
        assert!(!commands.iter().any(|command| command.contains("local_installers")));
    }

    #[test]
    fn resolves_configured_dgx_spark_cuda_package() {
        assert_eq!(
            configured_dgx_spark_cuda_toolkit_package("13-0"),
            Some("cuda-toolkit-13-0".to_string())
        );
        assert_eq!(
            configured_dgx_spark_cuda_toolkit_package("cuda-toolkit-13-0"),
            Some("cuda-toolkit-13-0".to_string())
        );
        assert_eq!(configured_dgx_spark_cuda_toolkit_package(""), None);
    }

    #[test]
    fn selects_latest_versioned_cuda_toolkit_from_apt_list() {
        let output = r#"
Listing... Done
cuda-toolkit-13-0/unknown 13.0.2-1 arm64
cuda-toolkit-13-2/unknown 13.2.0-1 arm64
cuda-toolkit-13-1/unknown 13.1.1-1 arm64
cuda-toolkit-13-0-config-common/unknown 13.0.2-1 all
"#;

        assert_eq!(
            latest_cuda_toolkit_package(output).as_deref(),
            Some("cuda-toolkit-13-2")
        );
    }

    #[test]
    fn missing_download_tool_is_validation_error() {
        let host = FakeHost::new();
        let executor = FakeExecutor::new(false);
        let reporter = FakeReporter::new();
        let config = Config::default();
        let platform = PlatformInfo::nvidia_linux(None, None, Some("x86_64".into()), "test");
        let context = MaintenanceContext {
            config: &config,
            platform: &platform,
            host: &host,
            executor: &executor,
            reporter: &reporter,
        };

        let err = fetch_text(&context, "https://example.invalid")
            .unwrap_err()
            .to_string();

        assert!(err.contains("DOMAIN_CUDA_DOWNLOAD_TOOL"));
    }
}
