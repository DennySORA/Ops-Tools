use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::platform::PlatformInfo;
use crate::features::system_updater::ports::{CommandExecutor, HostServices};
use std::os::unix::fs::PermissionsExt;

struct ToolSpec {
    binary: &'static str,
    args: &'static [&'static str],
    label: &'static str,
}

const TOOLS: &[ToolSpec] = &[
    ToolSpec {
        binary: "brew",
        args: &["--version"],
        label: "Homebrew",
    },
    ToolSpec {
        binary: "nvidia-smi",
        args: &["--query-gpu=driver_version", "--format=csv,noheader"],
        label: "NVIDIA Driver",
    },
    ToolSpec {
        binary: "nvcc",
        args: &["--version"],
        label: "CUDA (nvcc)",
    },
    ToolSpec {
        binary: "docker",
        args: &["--version"],
        label: "Docker",
    },
    ToolSpec {
        binary: "node",
        args: &["--version"],
        label: "Node.js",
    },
    ToolSpec {
        binary: "npm",
        args: &["--version"],
        label: "npm",
    },
    ToolSpec {
        binary: "pnpm",
        args: &["--version"],
        label: "pnpm",
    },
    ToolSpec {
        binary: "bun",
        args: &["--version"],
        label: "Bun",
    },
    ToolSpec {
        binary: "deno",
        args: &["--version"],
        label: "Deno",
    },
    ToolSpec {
        binary: "rustup",
        args: &["--version"],
        label: "rustup",
    },
    ToolSpec {
        binary: "rustc",
        args: &["--version"],
        label: "Rust",
    },
    ToolSpec {
        binary: "cargo",
        args: &["--version"],
        label: "Cargo",
    },
    ToolSpec {
        binary: "uv",
        args: &["--version"],
        label: "uv",
    },
    ToolSpec {
        binary: "python3",
        args: &["--version"],
        label: "Python",
    },
    ToolSpec {
        binary: "pip3",
        args: &["--version"],
        label: "pip",
    },
    ToolSpec {
        binary: "pipx",
        args: &["--version"],
        label: "pipx",
    },
    ToolSpec {
        binary: "go",
        args: &["version"],
        label: "Go",
    },
    ToolSpec {
        binary: "java",
        args: &["-version"],
        label: "Java",
    },
    ToolSpec {
        binary: "ruby",
        args: &["--version"],
        label: "Ruby",
    },
    ToolSpec {
        binary: "cmake",
        args: &["--version"],
        label: "CMake",
    },
    ToolSpec {
        binary: "gcc",
        args: &["--version"],
        label: "GCC",
    },
    ToolSpec {
        binary: "make",
        args: &["--version"],
        label: "Make",
    },
    ToolSpec {
        binary: "git",
        args: &["--version"],
        label: "Git",
    },
    ToolSpec {
        binary: "curl",
        args: &["--version"],
        label: "curl",
    },
    ToolSpec {
        binary: "wget",
        args: &["--version"],
        label: "wget",
    },
    ToolSpec {
        binary: "zsh",
        args: &["--version"],
        label: "zsh",
    },
    ToolSpec {
        binary: "tmux",
        args: &["-V"],
        label: "tmux",
    },
    ToolSpec {
        binary: "snap",
        args: &["--version"],
        label: "Snap",
    },
    ToolSpec {
        binary: "flatpak",
        args: &["--version"],
        label: "Flatpak",
    },
    ToolSpec {
        binary: "conda",
        args: &["--version"],
        label: "Conda",
    },
    ToolSpec {
        binary: "claude",
        args: &["--version"],
        label: "Claude Code",
    },
];

pub fn run_scan(
    host: &impl HostServices,
    executor: &impl CommandExecutor,
    platform: &PlatformInfo,
) {
    println!();
    println!("-- System Info ----------------------------------------");
    println!();
    scan_system_info(host, executor, platform);

    println!();
    println!("-- Installed Tools ------------------------------------");
    println!();
    scan_tools(host, executor);

    println!();
    println!("-- Global Packages ------------------------------------");
    println!();
    scan_global_packages(host, executor, platform);

    println!();
    println!("-- Security Posture -----------------------------------");
    println!();
    scan_security(host, executor, platform);
}

fn scan_system_info(
    host: &impl HostServices,
    executor: &impl CommandExecutor,
    platform: &PlatformInfo,
) {
    if platform.is_macos() {
        let product = executor
            .capture(&CommandSpec::new("sw_vers", ["-productName"]))
            .ok()
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|| "macOS".to_string());
        let version = executor
            .capture(&CommandSpec::new("sw_vers", ["-productVersion"]))
            .ok()
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if version.is_empty() {
            println!("  OS:       {product}");
        } else {
            println!("  OS:       {product} {version}");
        }
    } else if let Ok(content) = host.read_to_string(std::path::Path::new("/etc/os-release")) {
        for line in content.lines() {
            if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                println!("  OS:       {}", name.trim_matches('"'));
                break;
            }
        }
    }

    println!("  Platform  {}", platform.summary());
    print_capture(
        "Kernel",
        executor.capture(&CommandSpec::new("uname", ["-r"])),
    );
    print_capture("Arch", executor.capture(&CommandSpec::new("uname", ["-m"])));
    print_capture(
        "Uptime",
        executor.capture(&CommandSpec::new("uptime", ["-p"])),
    );
}

fn scan_tools(host: &impl HostServices, executor: &impl CommandExecutor) {
    for tool in TOOLS {
        if host.command_path(tool.binary).is_some() {
            let version = command_version(executor, tool.binary, tool.args);
            println!("  + {:<16} {version}", tool.label);
        } else {
            println!("  - {:<16} not installed", tool.label);
        }
    }

    scan_nvm(host, executor);
}

fn scan_nvm(host: &impl HostServices, executor: &impl CommandExecutor) {
    let home = host.var("HOME").unwrap_or_default();
    let xdg = host
        .var("XDG_CONFIG_HOME")
        .unwrap_or_else(|| format!("{home}/.config"));
    let candidates = [format!("{xdg}/nvm"), format!("{home}/.nvm")];

    for directory in &candidates {
        let nvm_sh = std::path::Path::new(directory).join("nvm.sh");
        if host.exists(&nvm_sh) {
            let script =
                format!(r#"export NVM_DIR="{directory}" && \. "$NVM_DIR/nvm.sh" && nvm --version"#);
            match executor.capture(&CommandSpec::new("bash", ["-c", &script])) {
                Ok(output) if !output.trim().is_empty() => {
                    println!("  + {:<16} {}", "nvm", output.trim());
                }
                _ => println!("  + {:<16} installed (version unknown)", "nvm"),
            }
            return;
        }
    }
    println!("  - {:<16} not installed", "nvm");
}

fn scan_global_packages(
    host: &impl HostServices,
    executor: &impl CommandExecutor,
    platform: &PlatformInfo,
) {
    if executor.is_dry_run() {
        println!("  [dry-run] skipping package count");
        return;
    }

    let mut found = false;

    if host.command_path("npm").is_some() {
        report_count_or_error(
            "npm global",
            "packages",
            executor.capture(&CommandSpec::new(
                "npm",
                ["list", "-g", "--depth=0", "--parseable"],
            )),
            |output| output.lines().count().saturating_sub(1),
            &mut found,
        );
    }

    if host.command_path("pnpm").is_some() {
        report_count_or_error(
            "pnpm global",
            "packages",
            executor.capture(&CommandSpec::new("pnpm", ["list", "-g", "--parseable"])),
            |output| {
                output
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .count()
                    .saturating_sub(1)
            },
            &mut found,
        );
    }

    if host.command_path("cargo").is_some() {
        report_count_or_error(
            "cargo install",
            "crates",
            executor.capture(&CommandSpec::new("cargo", ["install", "--list"])),
            |output| {
                output
                    .lines()
                    .filter(|line| !line.starts_with(' ') && !line.trim().is_empty())
                    .count()
            },
            &mut found,
        );
    }

    if host.command_path("uv").is_some() {
        report_count_or_error(
            "uv tools",
            "tools",
            executor.capture(&CommandSpec::new("uv", ["tool", "list"])),
            |output| {
                output
                    .lines()
                    .filter(|line| !line.starts_with(' ') && !line.trim().is_empty())
                    .count()
            },
            &mut found,
        );
    }

    if host.command_path("snap").is_some() {
        report_count_or_error(
            "snap",
            "packages",
            executor.capture(&CommandSpec::new("snap", ["list"])),
            |output| output.lines().count().saturating_sub(1),
            &mut found,
        );
    }

    if host.command_path("flatpak").is_some() {
        report_count_or_error(
            "flatpak",
            "packages",
            executor.capture(&CommandSpec::new("flatpak", ["list"])),
            |output| output.lines().count().saturating_sub(1),
            &mut found,
        );
    }

    if host.command_path("pipx").is_some() {
        report_count_or_error(
            "pipx apps",
            "apps",
            executor.capture(&CommandSpec::new("pipx", ["list", "--short"])),
            |output| {
                output
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .count()
            },
            &mut found,
        );
    }

    if host.command_path("conda").is_some() {
        report_count_or_error(
            "conda envs",
            "envs",
            executor.capture(&CommandSpec::new("conda", ["info", "--envs"])),
            |output| {
                output
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .filter(|line| !line.starts_with('#'))
                    .count()
            },
            &mut found,
        );
    }

    if platform.is_macos() {
        if host.command_path("brew").is_some() {
            report_count_or_error(
                "brew outdated",
                "packages",
                executor.capture(&CommandSpec::new("brew", ["outdated", "--quiet"])),
                |output| {
                    output
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .count()
                },
                &mut found,
            );
        }
        if host.command_path("softwareupdate").is_some() {
            report_count_or_error(
                "softwareupdate",
                "updates",
                executor.capture(&CommandSpec::new("softwareupdate", ["--list"])),
                available_software_update_count,
                &mut found,
            );
        }
    } else {
        report_count_or_error(
            "apt upgradable",
            "packages",
            executor.capture(&CommandSpec::new("apt", ["list", "--upgradable"])),
            |output| {
                output
                    .lines()
                    .filter(|line| line.contains("upgradable"))
                    .count()
            },
            &mut found,
        );
    }

    if !found {
        println!("  (no global packages detected)");
    }
}

fn report_count_or_error<F>(
    label: &str,
    unit: &str,
    result: Result<String, crate::features::system_updater::domain::error::InfrastructureError>,
    count_fn: F,
    found: &mut bool,
) where
    F: FnOnce(&str) -> usize,
{
    match result {
        Ok(output) => {
            let count = count_fn(&output);
            if count > 0 {
                println!("  {label:<15} {count} {unit}");
                *found = true;
            }
        }
        Err(err) => eprintln!("  !! failed to inspect {label}: {err}"),
    }
}

fn scan_security(
    host: &impl HostServices,
    executor: &impl CommandExecutor,
    platform: &PlatformInfo,
) {
    let path_var = host.var("PATH").unwrap_or_default();
    let mut issues = Vec::new();
    for directory in path_var.split(':') {
        if directory.is_empty() {
            issues.push("empty entry (implies CWD)".to_string());
            continue;
        }
        if let Ok(metadata) = std::fs::metadata(directory)
            && metadata.permissions().mode() & 0o002 != 0
        {
            issues.push(directory.to_string());
        }
    }

    if issues.is_empty() {
        println!("  OK  No world-writable directories in PATH");
    } else {
        for issue in &issues {
            println!("  !!  World-writable in PATH: {issue}");
        }
    }

    let home = host.var("HOME").unwrap_or_default();
    let ssh_dir = std::path::Path::new(&home).join(".ssh");
    if let Ok(metadata) = std::fs::metadata(&ssh_dir) {
        let mode = metadata.permissions().mode() & 0o777;
        if mode == 0o700 {
            println!("  OK  SSH directory permissions (700)");
        } else {
            println!("  !!  SSH directory permissions: {mode:o} (should be 700)");
        }
    }

    if platform.is_linux()
        && host.command_path("ufw").is_some()
        && host.command_path("systemctl").is_some()
    {
        match executor.capture(&CommandSpec::new(
            "systemctl",
            ["show", "-p", "ActiveState", "--value", "ufw"],
        )) {
            Ok(output) => {
                let status = output.trim();
                if status == "active" {
                    println!("  OK  UFW firewall service active");
                } else {
                    println!("  ??  UFW firewall service: {status}");
                }
            }
            Err(err) => eprintln!("  !!  Failed to inspect UFW service: {err}"),
        }
    }

    if platform.is_linux() {
        if host.exists(std::path::Path::new("/etc/apt/apt.conf.d/20auto-upgrades")) {
            println!("  OK  Unattended upgrades configured");
        } else {
            println!("  ??  Unattended upgrades not configured");
        }
    } else if host.command_path("softwareupdate").is_some() {
        match executor.capture(&CommandSpec::new("softwareupdate", ["--schedule"])) {
            Ok(output) => println!("  info softwareupdate schedule: {}", output.trim()),
            Err(err) => eprintln!("  !!  Failed to inspect softwareupdate schedule: {err}"),
        }
    }
}

fn available_software_update_count(output: &str) -> usize {
    output
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with('*') || line.starts_with('-'))
        .count()
}

fn command_version(executor: &impl CommandExecutor, program: &str, args: &[&str]) -> String {
    match executor.capture(&CommandSpec::new(program, args.iter().copied())) {
        Ok(output) => {
            let lines: Vec<&str> = output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect();
            let first = lines.first().copied().unwrap_or("");
            let best = if first.chars().any(|character| character.is_ascii_digit()) {
                Some(first)
            } else {
                lines.iter().copied().find(|line| {
                    let lower = line.to_ascii_lowercase();
                    (lower.contains("release") || lower.contains("version"))
                        && line.chars().any(|character| character.is_ascii_digit())
                })
            }
            .or(lines.first().copied());

            best.unwrap_or("").trim().chars().take(72).collect()
        }
        Err(err) => format!("error: {err}"),
    }
}

fn print_capture(
    label: &str,
    result: Result<String, crate::features::system_updater::domain::error::InfrastructureError>,
) {
    if let Ok(value) = result {
        println!("  {label:<8} {}", value.trim());
    }
}
