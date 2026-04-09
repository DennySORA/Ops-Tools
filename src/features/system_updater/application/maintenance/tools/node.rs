use crate::features::system_updater::application::maintenance::tools::common::{
    filter_targets, parse_parseable_package_lines, run_npm_update,
};
use crate::features::system_updater::application::maintenance::{
    MaintenanceContext, WarningCollector,
};
use crate::features::system_updater::domain::command::CommandSpec;
use crate::features::system_updater::domain::error::{AppResult, DomainError};
use crate::features::system_updater::domain::report::{StepOutcome, StepStatus};
use crate::features::system_updater::ports::{CommandExecutor, HostServices, RunReporter};
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn update_nvm_node<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let home = context.host.var("HOME").unwrap_or_default();
    let mut warnings = WarningCollector::new();

    if let Some(nvm_dir) = find_nvm_dir(context, &home) {
        println!("  Detected nvm at {nvm_dir}");
        if context.config.tools.nvm.self_update {
            warnings.capture(
                "nvm self-update",
                secure_nvm_self_update(context, &context.config.tools.nvm.installer_version),
            );
        } else {
            println!("  nvm self-update disabled in config, skipping.");
        }

        let script = format!(
            r#"export NVM_DIR="{nvm_dir}" && \. "$NVM_DIR/nvm.sh" && \
               nvm install --lts --reinstall-packages-from=current --latest-npm && \
               nvm alias default 'lts/*' && \
               nvm use --lts"#
        );
        warnings.capture(
            "nvm install/use latest LTS",
            context
                .executor
                .run(&CommandSpec::new("bash", ["-c", script.as_str()]).with_timeout_secs(1800)),
        );
        warnings.capture("npm global update", npm_update_global(context));
    } else if context.host.command_path("npm").is_some() {
        println!("  No nvm detected, updating npm global packages only.");
        warnings.capture("npm global update", npm_update_global(context));
    } else {
        println!("  No nvm or npm detected, skipping.");
        return Ok(StepOutcome::skipped("neither nvm nor npm is installed"));
    }

    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("node toolchain maintenance previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

pub fn update_pnpm<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<StepOutcome>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("pnpm").is_none() {
        println!("  pnpm not found, skipping.");
        return Ok(StepOutcome::skipped("pnpm not installed"));
    }

    let mut warnings = WarningCollector::new();
    warnings.capture(
        "pnpm self-update",
        context
            .executor
            .run(&CommandSpec::new("pnpm", ["self-update"]).with_timeout_secs(900)),
    );

    let package_targets = if context.config.tools.node.pnpm_allow.is_empty()
        && context.config.tools.node.pnpm_deny.is_empty()
    {
        Vec::new()
    } else {
        let raw = context
            .executor
            .capture(&CommandSpec::new("pnpm", ["list", "-g", "--parseable"]))?;
        filter_targets(
            parse_parseable_package_lines(&raw),
            &context.config.tools.node.pnpm_allow,
            &context.config.tools.node.pnpm_deny,
        )
    };

    if context.config.tools.node.pnpm_allow.is_empty()
        && context.config.tools.node.pnpm_deny.is_empty()
    {
        warnings.capture(
            "pnpm update -g",
            context
                .executor
                .run(&CommandSpec::new("pnpm", ["update", "-g"]).with_timeout_secs(1800)),
        );
    } else if package_targets.is_empty() {
        println!("  No pnpm packages matched policy filters, skipping package upgrades.");
    } else {
        let mut args = vec!["update".to_string(), "-g".into()];
        args.extend(package_targets);
        warnings.capture(
            "pnpm update -g [filtered]",
            context
                .executor
                .run(&CommandSpec::new("pnpm", args).with_timeout_secs(1800)),
        );
    }

    warnings.capture(
        "pnpm store prune",
        context
            .executor
            .run(&CommandSpec::new("pnpm", ["store", "prune"]).with_timeout_secs(900)),
    );
    Ok(if context.executor.is_dry_run() {
        StepOutcome::dry_run("pnpm maintenance previewed")
    } else {
        warnings.finish_as(StepStatus::Partial)
    })
}

fn find_nvm_dir<H, E, R>(context: &MaintenanceContext<'_, H, E, R>, home: &str) -> Option<String>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let xdg = context
        .host
        .var("XDG_CONFIG_HOME")
        .unwrap_or_else(|| format!("{home}/.config"));
    let candidates = [format!("{xdg}/nvm"), format!("{home}/.nvm")];

    for directory in &candidates {
        let path = Path::new(directory).join("nvm.sh");
        if context.host.exists(&path) {
            return Some(directory.clone());
        }
    }

    None
}

fn secure_nvm_self_update<H, E, R>(
    context: &MaintenanceContext<'_, H, E, R>,
    version: &str,
) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    let has_curl = context.host.command_path("curl").is_some();
    let has_wget = context.host.command_path("wget").is_some();
    if !has_curl && !has_wget {
        println!("  No curl/wget found, skipping nvm self-update.");
        return Ok(());
    }

    println!("  Updating nvm ({version}) -- secure download...");
    let url = format!("https://raw.githubusercontent.com/nvm-sh/nvm/{version}/install.sh");
    let temp = Path::new("/tmp/nvm-install.sh");

    if has_curl {
        context.executor.run(
            &CommandSpec::new(
                "curl",
                ["-fsSL", "-o", temp.to_string_lossy().as_ref(), url.as_str()],
            )
            .with_timeout_secs(300),
        )?;
    } else {
        context.executor.run(
            &CommandSpec::new(
                "wget",
                ["-qO", temp.to_string_lossy().as_ref(), url.as_str()],
            )
            .with_timeout_secs(300),
        )?;
    }

    if context.executor.is_dry_run() {
        return Ok(());
    }

    let content = context.host.read_to_string(temp)?;
    if content.is_empty() || content.len() > 1_048_576 {
        return Err(DomainError::safety(
            "DOMAIN_NVM_INSTALLER_SIZE",
            format!(
                "nvm installer has suspicious size ({} bytes)",
                content.len()
            ),
        )
        .into());
    }
    if !content.starts_with("#!/") {
        return Err(DomainError::safety(
            "DOMAIN_NVM_INSTALLER_SHEBANG",
            "nvm installer missing shebang -- aborting for safety",
        )
        .into());
    }
    if let Some(expected_hash) = &context.config.tools.nvm.installer_sha256 {
        let actual_hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        if &actual_hash != expected_hash {
            return Err(DomainError::safety(
                "DOMAIN_NVM_INSTALLER_HASH",
                format!("nvm installer hash mismatch: expected {expected_hash}, got {actual_hash}"),
            )
            .into());
        }
    }

    context
        .executor
        .run(&CommandSpec::new("bash", [temp.to_string_lossy().as_ref()]).with_timeout_secs(600))?;
    Ok(())
}

fn npm_update_global<H, E, R>(context: &MaintenanceContext<'_, H, E, R>) -> AppResult<()>
where
    H: HostServices,
    E: CommandExecutor,
    R: RunReporter,
{
    if context.host.command_path("npm").is_none() {
        return Ok(());
    }

    let targets = if context.config.tools.node.npm_allow.is_empty()
        && context.config.tools.node.npm_deny.is_empty()
    {
        Vec::new()
    } else {
        let raw = context.executor.capture(&CommandSpec::new(
            "npm",
            ["list", "-g", "--depth=0", "--parseable"],
        ))?;
        filter_targets(
            parse_parseable_package_lines(&raw),
            &context.config.tools.node.npm_allow,
            &context.config.tools.node.npm_deny,
        )
    };

    if !context.config.tools.node.npm_allow.is_empty()
        || !context.config.tools.node.npm_deny.is_empty()
    {
        if targets.is_empty() {
            println!("  No npm packages matched policy filters, skipping package upgrades.");
            return Ok(());
        }
        println!("  Updating filtered npm global packages...");
        run_npm_update(context, &targets)?;
        return Ok(());
    }

    println!("  Updating npm global packages...");
    run_npm_update(context, &[])?;
    Ok(())
}
