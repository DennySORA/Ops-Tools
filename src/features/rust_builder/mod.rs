use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Clone, Copy)]
enum Builder {
    Cargo,
    Cross,
}

#[derive(Clone)]
struct Target {
    triple: &'static str,
    name_key: &'static str,
}

/// Entry point for Rust multi-platform builder
pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::RUST_BUILDER_HEADER));

    // Ensure Cargo project exists
    let project_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            console.error(&err.to_string());
            return;
        }
    };

    if !project_dir.join("Cargo.toml").exists() {
        console.error(i18n::t(keys::RUST_BUILDER_NO_CARGO_TOML));
        return;
    }

    if !command_available("cargo") {
        console.error(&crate::tr!(
            keys::ERROR_COMMAND_NOT_FOUND,
            command = "cargo"
        ));
        return;
    }

    if !command_available("rustup") {
        console.error(i18n::t(keys::RUST_BUILDER_RUSTUP_MISSING));
        return;
    }

    let builder = match select_builder(&prompts) {
        Some(b) => b,
        None => {
            console.warning(i18n::t(keys::RUST_BUILDER_CANCELLED));
            return;
        }
    };

    let release = match select_profile(&prompts) {
        Some(p) => p,
        None => {
            console.warning(i18n::t(keys::RUST_BUILDER_CANCELLED));
            return;
        }
    };

    let targets = match select_targets(&prompts) {
        Some(t) if !t.is_empty() => t,
        _ => {
            console.warning(i18n::t(keys::RUST_BUILDER_NO_TARGET_SELECTED));
            return;
        }
    };

    // Install missing targets
    let installed = match installed_targets() {
        Ok(list) => list,
        Err(err) => {
            console.error(&err);
            return;
        }
    };

    let missing: Vec<&Target> = targets
        .iter()
        .filter(|t| !installed.contains(t.triple))
        .collect();

    let mut install_failures = HashSet::new();
    if !missing.is_empty() {
        console.warning(&crate::tr!(
            keys::RUST_BUILDER_MISSING_TARGETS,
            count = missing.len()
        ));

        if prompts.confirm(i18n::t(keys::RUST_BUILDER_CONFIRM_INSTALL_TARGETS)) {
            for (idx, target) in missing.iter().enumerate() {
                console.show_progress(
                    idx + 1,
                    missing.len(),
                    &crate::tr!(keys::RUST_BUILDER_INSTALLING_TARGET, target = target.triple),
                );

                match install_target(target.triple) {
                    Ok(_) => console.success_item(&crate::tr!(
                        keys::RUST_BUILDER_INSTALL_SUCCESS,
                        target = target.triple
                    )),
                    Err(err) => {
                        console.error_item(
                            &crate::tr!(keys::RUST_BUILDER_INSTALL_FAILED, target = target.triple),
                            &err,
                        );
                        install_failures.insert(target.triple);
                    }
                }
            }
            console.separator();
        } else {
            console.warning(i18n::t(keys::RUST_BUILDER_SKIP_INSTALL));
            console.separator();
        }
    }

    // Build selected targets
    let mut success = 0;
    let mut failed = 0;

    for (idx, target) in targets.iter().enumerate() {
        if install_failures.contains(target.triple) {
            failed += 1;
            continue;
        }

        console.show_progress(
            idx + 1,
            targets.len(),
            &crate::tr!(keys::RUST_BUILDER_BUILDING, target = target.triple),
        );

        match build_target(&project_dir, target.triple, builder, release) {
            Ok(binary_dir) => {
                console.success_item(&crate::tr!(
                    keys::RUST_BUILDER_BUILD_SUCCESS,
                    target = target.triple
                ));
                console.list_item(" ", &binary_dir.display().to_string());
                success += 1;
            }
            Err(err) => {
                console.error_item(
                    &crate::tr!(keys::RUST_BUILDER_BUILD_FAILED, target = target.triple),
                    &err,
                );
                failed += 1;
            }
        }

        console.blank_line();
    }

    console.show_summary(i18n::t(keys::RUST_BUILDER_SUMMARY_TITLE), success, failed);
}

fn select_builder(prompts: &Prompts) -> Option<Builder> {
    let cross_available = command_available("cross");

    let mut options = vec![i18n::t(keys::RUST_BUILDER_BUILDER_CARGO).to_string()];
    if cross_available {
        options.push(i18n::t(keys::RUST_BUILDER_BUILDER_CROSS).to_string());
    }

    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
    let selection = prompts.select(i18n::t(keys::RUST_BUILDER_SELECT_BUILDER), &option_refs)?;

    if selection == 0 {
        Some(Builder::Cargo)
    } else {
        Some(Builder::Cross)
    }
}

fn select_profile(prompts: &Prompts) -> Option<bool> {
    let options = vec![
        i18n::t(keys::RUST_BUILDER_PROFILE_RELEASE).to_string(),
        i18n::t(keys::RUST_BUILDER_PROFILE_DEBUG).to_string(),
    ];
    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

    prompts
        .select_with_default(i18n::t(keys::RUST_BUILDER_SELECT_PROFILE), &option_refs, 0)
        .map(|idx| idx == 0)
}

fn select_targets(prompts: &Prompts) -> Option<Vec<Target>> {
    let targets = available_targets();
    let host = host_triple();

    let items: Vec<String> = targets
        .iter()
        .map(|t| format!("{} — {}", i18n::t(t.name_key), t.triple))
        .collect();

    let defaults: Vec<bool> = targets
        .iter()
        .map(|t| host.as_deref() == Some(t.triple))
        .collect();

    let selections = prompts.multi_select(
        i18n::t(keys::RUST_BUILDER_SELECT_TARGETS),
        &items,
        &defaults,
    );

    if selections.is_empty() {
        return None;
    }

    let chosen = selections
        .into_iter()
        .map(|idx| targets[idx].clone())
        .collect();
    Some(chosen)
}

fn available_targets() -> Vec<Target> {
    vec![
        Target {
            triple: "x86_64-unknown-linux-gnu",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_X86_64_GNU,
        },
        Target {
            triple: "aarch64-unknown-linux-gnu",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_ARM64_GNU,
        },
        Target {
            triple: "i686-unknown-linux-gnu",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_I686_GNU,
        },
        Target {
            triple: "armv7-unknown-linux-gnueabihf",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_ARMV7_GNU,
        },
        Target {
            triple: "riscv64gc-unknown-linux-gnu",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_RISCV64_GNU,
        },
        Target {
            triple: "powerpc64le-unknown-linux-gnu",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_PPC64LE_GNU,
        },
        Target {
            triple: "x86_64-unknown-linux-musl",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_X86_64_MUSL,
        },
        Target {
            triple: "aarch64-unknown-linux-musl",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_ARM64_MUSL,
        },
        Target {
            triple: "i686-unknown-linux-musl",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_I686_MUSL,
        },
        Target {
            triple: "armv7-unknown-linux-musleabihf",
            name_key: keys::RUST_BUILDER_TARGET_LINUX_ARMV7_MUSL,
        },
        Target {
            triple: "x86_64-apple-darwin",
            name_key: keys::RUST_BUILDER_TARGET_MACOS_X86_64,
        },
        Target {
            triple: "aarch64-apple-darwin",
            name_key: keys::RUST_BUILDER_TARGET_MACOS_ARM64,
        },
        Target {
            triple: "x86_64-pc-windows-gnu",
            name_key: keys::RUST_BUILDER_TARGET_WINDOWS_X86_64,
        },
        Target {
            triple: "aarch64-pc-windows-msvc",
            name_key: keys::RUST_BUILDER_TARGET_WINDOWS_ARM64,
        },
        Target {
            triple: "wasm32-unknown-unknown",
            name_key: keys::RUST_BUILDER_TARGET_WASM32_UNKNOWN,
        },
    ]
}

fn installed_targets() -> Result<HashSet<String>, String> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let set = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(set)
}

fn install_target(target: &str) -> Result<(), String> {
    let status = Command::new("rustup")
        .args(["target", "add", target])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("rustup target add {} failed", target))
    }
}

fn build_target(
    project_dir: &PathBuf,
    target: &str,
    builder: Builder,
    release: bool,
) -> Result<PathBuf, String> {
    let mut args = vec!["build", "--target", target];
    if release {
        args.push("--release");
    }

    let program = match builder {
        Builder::Cargo => "cargo",
        Builder::Cross => "cross",
    };

    let status = Command::new(program)
        .args(&args)
        .current_dir(project_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        let profile_dir = if release { "release" } else { "debug" };
        Ok(project_dir.join("target").join(target).join(profile_dir))
    } else {
        Err(format!("{} build failed", program))
    }
}

fn command_available(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn host_triple() -> Option<String> {
    let output = Command::new("rustc").args(["-Vv"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(rest) = line.strip_prefix("host: ") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_exist() {
        let list = available_targets();
        assert!(!list.is_empty());
    }
}
