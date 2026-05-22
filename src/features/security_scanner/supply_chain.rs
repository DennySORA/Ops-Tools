use crate::core::{OperationError, Result};
use crate::i18n;
use serde_json::Value as JsonValue;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;
use walkdir::{DirEntry, WalkDir};

const NPM_LOCKFILES: &[&str] = &[
    "package-lock.json",
    "npm-shrinkwrap.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lock",
    "bun.lockb",
];

const PYTHON_LOCKFILES: &[&str] = &[
    "poetry.lock",
    "uv.lock",
    "pdm.lock",
    "Pipfile.lock",
    "requirements.lock",
];

const SKIP_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    "vendor",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".cache",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    "__pycache__",
    "venv",
    ".venv",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Npm,
    Python,
    Rust,
}

impl Ecosystem {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Python => "Python",
            Self::Rust => "Rust",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    fn rank(self) -> u8 {
        match self {
            Self::Critical => 5,
            Self::High => 4,
            Self::Medium => 3,
            Self::Low => 2,
            Self::Info => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FindingKind {
    ManifestParseFailed,
    NpmLifecycleScript,
    NpmSuspiciousScript,
    NpmLockMissing,
    NpmRemoteDependency,
    NpmLocalDependency,
    NpmUnpinnedDependency,
    NpmLockInstallScript,
    NpmLockExternalSource,
    NpmLockMissingIntegrity,
    PythonDirectUrl,
    PythonUnpinnedRequirement,
    PythonExternalIndex,
    PythonTrustedHost,
    PythonLockMissing,
    PythonLocalPath,
    RustGitDependency,
    RustMutableGitDependency,
    RustPathDependency,
    RustWildcardDependency,
    RustPatchOverride,
    RustLockMissing,
    RustBuildScript,
    RustLockMissingChecksum,
    RustAlternateRegistry,
}

impl FindingKind {
    fn title_key(self) -> &'static str {
        match self {
            Self::ManifestParseFailed => "security_scanner.supply_chain.rule.parse_failed.title",
            Self::NpmLifecycleScript => {
                "security_scanner.supply_chain.rule.npm_lifecycle_script.title"
            }
            Self::NpmSuspiciousScript => {
                "security_scanner.supply_chain.rule.npm_suspicious_script.title"
            }
            Self::NpmLockMissing => "security_scanner.supply_chain.rule.npm_lock_missing.title",
            Self::NpmRemoteDependency => {
                "security_scanner.supply_chain.rule.npm_remote_dependency.title"
            }
            Self::NpmLocalDependency => {
                "security_scanner.supply_chain.rule.npm_local_dependency.title"
            }
            Self::NpmUnpinnedDependency => {
                "security_scanner.supply_chain.rule.npm_unpinned_dependency.title"
            }
            Self::NpmLockInstallScript => {
                "security_scanner.supply_chain.rule.npm_lock_install_script.title"
            }
            Self::NpmLockExternalSource => {
                "security_scanner.supply_chain.rule.npm_lock_external_source.title"
            }
            Self::NpmLockMissingIntegrity => {
                "security_scanner.supply_chain.rule.npm_lock_missing_integrity.title"
            }
            Self::PythonDirectUrl => "security_scanner.supply_chain.rule.python_direct_url.title",
            Self::PythonUnpinnedRequirement => {
                "security_scanner.supply_chain.rule.python_unpinned_requirement.title"
            }
            Self::PythonExternalIndex => {
                "security_scanner.supply_chain.rule.python_external_index.title"
            }
            Self::PythonTrustedHost => {
                "security_scanner.supply_chain.rule.python_trusted_host.title"
            }
            Self::PythonLockMissing => {
                "security_scanner.supply_chain.rule.python_lock_missing.title"
            }
            Self::PythonLocalPath => "security_scanner.supply_chain.rule.python_local_path.title",
            Self::RustGitDependency => {
                "security_scanner.supply_chain.rule.rust_git_dependency.title"
            }
            Self::RustMutableGitDependency => {
                "security_scanner.supply_chain.rule.rust_mutable_git_dependency.title"
            }
            Self::RustPathDependency => {
                "security_scanner.supply_chain.rule.rust_path_dependency.title"
            }
            Self::RustWildcardDependency => {
                "security_scanner.supply_chain.rule.rust_wildcard_dependency.title"
            }
            Self::RustPatchOverride => {
                "security_scanner.supply_chain.rule.rust_patch_override.title"
            }
            Self::RustLockMissing => "security_scanner.supply_chain.rule.rust_lock_missing.title",
            Self::RustBuildScript => "security_scanner.supply_chain.rule.rust_build_script.title",
            Self::RustLockMissingChecksum => {
                "security_scanner.supply_chain.rule.rust_lock_missing_checksum.title"
            }
            Self::RustAlternateRegistry => {
                "security_scanner.supply_chain.rule.rust_alternate_registry.title"
            }
        }
    }

    fn recommendation_key(self) -> &'static str {
        match self {
            Self::ManifestParseFailed => {
                "security_scanner.supply_chain.rule.parse_failed.recommendation"
            }
            Self::NpmLifecycleScript => {
                "security_scanner.supply_chain.rule.npm_lifecycle_script.recommendation"
            }
            Self::NpmSuspiciousScript => {
                "security_scanner.supply_chain.rule.npm_suspicious_script.recommendation"
            }
            Self::NpmLockMissing => {
                "security_scanner.supply_chain.rule.npm_lock_missing.recommendation"
            }
            Self::NpmRemoteDependency => {
                "security_scanner.supply_chain.rule.npm_remote_dependency.recommendation"
            }
            Self::NpmLocalDependency => {
                "security_scanner.supply_chain.rule.npm_local_dependency.recommendation"
            }
            Self::NpmUnpinnedDependency => {
                "security_scanner.supply_chain.rule.npm_unpinned_dependency.recommendation"
            }
            Self::NpmLockInstallScript => {
                "security_scanner.supply_chain.rule.npm_lock_install_script.recommendation"
            }
            Self::NpmLockExternalSource => {
                "security_scanner.supply_chain.rule.npm_lock_external_source.recommendation"
            }
            Self::NpmLockMissingIntegrity => {
                "security_scanner.supply_chain.rule.npm_lock_missing_integrity.recommendation"
            }
            Self::PythonDirectUrl => {
                "security_scanner.supply_chain.rule.python_direct_url.recommendation"
            }
            Self::PythonUnpinnedRequirement => {
                "security_scanner.supply_chain.rule.python_unpinned_requirement.recommendation"
            }
            Self::PythonExternalIndex => {
                "security_scanner.supply_chain.rule.python_external_index.recommendation"
            }
            Self::PythonTrustedHost => {
                "security_scanner.supply_chain.rule.python_trusted_host.recommendation"
            }
            Self::PythonLockMissing => {
                "security_scanner.supply_chain.rule.python_lock_missing.recommendation"
            }
            Self::PythonLocalPath => {
                "security_scanner.supply_chain.rule.python_local_path.recommendation"
            }
            Self::RustGitDependency => {
                "security_scanner.supply_chain.rule.rust_git_dependency.recommendation"
            }
            Self::RustMutableGitDependency => {
                "security_scanner.supply_chain.rule.rust_mutable_git_dependency.recommendation"
            }
            Self::RustPathDependency => {
                "security_scanner.supply_chain.rule.rust_path_dependency.recommendation"
            }
            Self::RustWildcardDependency => {
                "security_scanner.supply_chain.rule.rust_wildcard_dependency.recommendation"
            }
            Self::RustPatchOverride => {
                "security_scanner.supply_chain.rule.rust_patch_override.recommendation"
            }
            Self::RustLockMissing => {
                "security_scanner.supply_chain.rule.rust_lock_missing.recommendation"
            }
            Self::RustBuildScript => {
                "security_scanner.supply_chain.rule.rust_build_script.recommendation"
            }
            Self::RustLockMissingChecksum => {
                "security_scanner.supply_chain.rule.rust_lock_missing_checksum.recommendation"
            }
            Self::RustAlternateRegistry => {
                "security_scanner.supply_chain.rule.rust_alternate_registry.recommendation"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageFile {
    pub ecosystem: Ecosystem,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainFinding {
    pub ecosystem: Ecosystem,
    pub severity: Severity,
    pub kind: FindingKind,
    pub path: PathBuf,
    pub detail: String,
}

impl SupplyChainFinding {
    pub fn title(&self) -> &'static str {
        i18n::t(self.kind.title_key())
    }

    pub fn recommendation(&self) -> &'static str {
        i18n::t(self.kind.recommendation_key())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SupplyChainReport {
    pub package_files: Vec<PackageFile>,
    pub findings: Vec<SupplyChainFinding>,
}

impl SupplyChainReport {
    pub fn ecosystem_summary(&self) -> String {
        let ecosystems = self
            .package_files
            .iter()
            .map(|file| file.ecosystem.display_name())
            .collect::<BTreeSet<_>>();

        if ecosystems.is_empty() {
            i18n::t("security_scanner.supply_chain.none").to_string()
        } else {
            ecosystems.into_iter().collect::<Vec<_>>().join(", ")
        }
    }

    fn add_package_file(&mut self, ecosystem: Ecosystem, path: PathBuf) {
        self.package_files.push(PackageFile { ecosystem, path });
    }

    fn add_finding(
        &mut self,
        ecosystem: Ecosystem,
        severity: Severity,
        kind: FindingKind,
        path: &Path,
        detail: impl Into<String>,
    ) {
        self.findings.push(SupplyChainFinding {
            ecosystem,
            severity,
            kind,
            path: path.to_path_buf(),
            detail: detail.into(),
        });
    }

    fn sort(&mut self) {
        self.package_files
            .sort_by(|left, right| left.path.cmp(&right.path));
        self.findings.sort_by(|left, right| {
            right
                .severity
                .rank()
                .cmp(&left.severity.rank())
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.detail.cmp(&right.detail))
        });
    }
}

pub fn scan_supply_chain(root: &Path) -> Result<SupplyChainReport> {
    let mut report = SupplyChainReport::default();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_visit)
    {
        let entry = entry.map_err(|err| OperationError::Io {
            path: err
                .path()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| root.display().to_string()),
            source: err
                .into_io_error()
                .unwrap_or_else(|| std::io::Error::other("walkdir error")),
        })?;

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative_path = relative_path(root, path);
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        match file_name {
            "package.json" => scan_npm_package(root, path, &relative_path, &mut report)?,
            "package-lock.json" | "npm-shrinkwrap.json" => {
                scan_npm_lock(path, &relative_path, &mut report)?
            }
            "pyproject.toml" => scan_pyproject(root, path, &relative_path, &mut report)?,
            "Pipfile" => scan_pipfile(root, path, &relative_path, &mut report)?,
            "Cargo.toml" => scan_cargo_toml(root, path, &relative_path, &mut report)?,
            "Cargo.lock" => scan_cargo_lock(path, &relative_path, &mut report)?,
            name if is_python_requirements_file(name) => {
                scan_requirements_txt(path, &relative_path, &mut report)?
            }
            name if is_npm_text_lock_file(name) => {
                scan_npm_text_lock(path, &relative_path, &mut report)?
            }
            "bun.lockb" => report.add_package_file(Ecosystem::Npm, relative_path),
            name if PYTHON_LOCKFILES.contains(&name) => {
                scan_python_lock_file(path, &relative_path, &mut report)?
            }
            _ => {}
        }
    }

    report.sort();
    Ok(report)
}

fn should_visit(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }

    let name = entry.file_name().to_string_lossy();
    !SKIP_DIRS.contains(&name.as_ref())
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|err| OperationError::Io {
        path: path.display().to_string(),
        source: err,
    })
}

fn scan_npm_package(
    root: &Path,
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Npm, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let json = match serde_json::from_str::<JsonValue>(&content) {
        Ok(json) => json,
        Err(err) => {
            report.add_finding(
                Ecosystem::Npm,
                Severity::Medium,
                FindingKind::ManifestParseFailed,
                relative_path,
                format!("package.json parse error: {err}"),
            );
            return Ok(());
        }
    };

    let Some(object) = json.as_object() else {
        report.add_finding(
            Ecosystem::Npm,
            Severity::Medium,
            FindingKind::ManifestParseFailed,
            relative_path,
            "package.json root is not an object",
        );
        return Ok(());
    };

    let package_dir = path.parent().unwrap_or(root);
    if !has_any_file_in_ancestors(root, package_dir, NPM_LOCKFILES) {
        report.add_finding(
            Ecosystem::Npm,
            Severity::Medium,
            FindingKind::NpmLockMissing,
            relative_path,
            "no npm, pnpm, Yarn, or Bun lockfile found in this package path",
        );
    }

    scan_npm_scripts(object, relative_path, report);
    scan_npm_dependencies(object, relative_path, report);
    scan_npm_resolution_overrides(object, relative_path, report);

    Ok(())
}

fn scan_npm_scripts(
    object: &serde_json::Map<String, JsonValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let Some(scripts) = object.get("scripts").and_then(|value| value.as_object()) else {
        return;
    };

    for (name, value) in scripts {
        let Some(command) = value.as_str() else {
            continue;
        };

        if is_npm_lifecycle_script(name) {
            let severity = if matches!(name.as_str(), "preinstall" | "install" | "postinstall") {
                Severity::High
            } else {
                Severity::Medium
            };
            report.add_finding(
                Ecosystem::Npm,
                severity,
                FindingKind::NpmLifecycleScript,
                relative_path,
                format!("script `{name}` runs `{command}`"),
            );
        }

        if let Some(reason) = suspicious_script_reason(command) {
            report.add_finding(
                Ecosystem::Npm,
                Severity::Critical,
                FindingKind::NpmSuspiciousScript,
                relative_path,
                format!("script `{name}` {reason}: `{command}`"),
            );
        }
    }
}

fn scan_npm_dependencies(
    object: &serde_json::Map<String, JsonValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let sections = [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ];

    for section in sections {
        let Some(dependencies) = object.get(section).and_then(|value| value.as_object()) else {
            continue;
        };

        for (name, value) in dependencies {
            let Some(spec) = value.as_str() else {
                continue;
            };
            analyze_npm_dependency(section, name, spec, relative_path, report);
        }
    }
}

fn scan_npm_resolution_overrides(
    object: &serde_json::Map<String, JsonValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    for section in ["overrides", "resolutions"] {
        let Some(value) = object.get(section) else {
            continue;
        };
        scan_npm_override_value(section, value, relative_path, report);
    }

    if let Some(pnpm_overrides) = object.get("pnpm").and_then(|value| value.get("overrides")) {
        scan_npm_override_value("pnpm.overrides", pnpm_overrides, relative_path, report);
    }
}

fn scan_npm_override_value(
    path: &str,
    value: &JsonValue,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    match value {
        JsonValue::String(spec) => {
            let name = path.rsplit('.').next().unwrap_or(path);
            analyze_npm_dependency(path, name, spec, relative_path, report);
        }
        JsonValue::Object(map) => {
            for (name, value) in map {
                let child_path = if path.is_empty() {
                    name.to_string()
                } else {
                    format!("{path}.{name}")
                };
                scan_npm_override_value(&child_path, value, relative_path, report);
            }
        }
        _ => {}
    }
}

fn analyze_npm_dependency(
    section: &str,
    name: &str,
    spec: &str,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let normalized = spec.trim().to_ascii_lowercase();
    let subject = format!("{section}.{name} uses `{spec}`");

    if normalized.is_empty() || normalized == "*" || normalized == "latest" {
        report.add_finding(
            Ecosystem::Npm,
            Severity::Medium,
            FindingKind::NpmUnpinnedDependency,
            relative_path,
            subject,
        );
        return;
    }

    if is_remote_npm_spec(&normalized) {
        let severity = if normalized.starts_with("http://") || normalized.starts_with("git://") {
            Severity::High
        } else {
            Severity::Medium
        };
        report.add_finding(
            Ecosystem::Npm,
            severity,
            FindingKind::NpmRemoteDependency,
            relative_path,
            subject,
        );
    } else if normalized.starts_with("file:")
        || normalized.starts_with("link:")
        || normalized.starts_with("workspace:")
    {
        report.add_finding(
            Ecosystem::Npm,
            Severity::Info,
            FindingKind::NpmLocalDependency,
            relative_path,
            subject,
        );
    }
}

fn scan_npm_lock(path: &Path, relative_path: &Path, report: &mut SupplyChainReport) -> Result<()> {
    report.add_package_file(Ecosystem::Npm, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let json = match serde_json::from_str::<JsonValue>(&content) {
        Ok(json) => json,
        Err(err) => {
            report.add_finding(
                Ecosystem::Npm,
                Severity::Medium,
                FindingKind::ManifestParseFailed,
                relative_path,
                format!("npm lockfile parse error: {err}"),
            );
            return Ok(());
        }
    };

    let packages = json.get("packages").and_then(|value| value.as_object());
    if let Some(packages) = packages {
        for (package_path, package) in packages {
            if package_path.is_empty() {
                continue;
            }
            scan_npm_locked_package(package_path, package, relative_path, report);
        }
    } else if let Some(dependencies) = json.get("dependencies").and_then(|value| value.as_object())
    {
        for (name, dependency) in dependencies {
            scan_npm_legacy_lock_dependency(name, dependency, relative_path, report);
        }
    }

    Ok(())
}

fn scan_npm_text_lock(
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Npm, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let mut seen = BTreeSet::new();
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let normalized = trimmed.to_ascii_lowercase();
        if is_npm_lock_selector_only_line(&normalized) {
            continue;
        }
        if !is_suspicious_npm_lock_line(&normalized) || !seen.insert(normalized.clone()) {
            continue;
        }

        let severity = if normalized.contains("http://")
            || normalized.contains("git://")
            || normalized.contains("ssh://")
            || normalized.contains("git+http")
            || normalized.contains("git+ssh")
        {
            Severity::High
        } else {
            Severity::Medium
        };
        report.add_finding(
            Ecosystem::Npm,
            severity,
            FindingKind::NpmLockExternalSource,
            relative_path,
            format!("line {} references `{trimmed}`", index + 1),
        );
    }

    Ok(())
}

fn scan_npm_locked_package(
    package_path: &str,
    package: &JsonValue,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let label = npm_lock_package_label(package_path, package);

    if package
        .get("hasInstallScript")
        .and_then(|value| value.as_bool())
        == Some(true)
    {
        report.add_finding(
            Ecosystem::Npm,
            Severity::Medium,
            FindingKind::NpmLockInstallScript,
            relative_path,
            format!("locked package `{label}` declares install lifecycle scripts"),
        );
    }

    let resolved = package.get("resolved").and_then(|value| value.as_str());
    if let Some(resolved) = resolved {
        analyze_npm_lock_source(&label, resolved, package, relative_path, report);
    }
}

fn scan_npm_legacy_lock_dependency(
    name: &str,
    dependency: &JsonValue,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let version = dependency
        .get("version")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let label = format!("{name}@{version}");

    if let Some(resolved) = dependency.get("resolved").and_then(|value| value.as_str()) {
        analyze_npm_lock_source(&label, resolved, dependency, relative_path, report);
    }

    if let Some(children) = dependency
        .get("dependencies")
        .and_then(|value| value.as_object())
    {
        for (child_name, child_dependency) in children {
            scan_npm_legacy_lock_dependency(child_name, child_dependency, relative_path, report);
        }
    }
}

fn analyze_npm_lock_source(
    label: &str,
    resolved: &str,
    package: &JsonValue,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let normalized = resolved.to_ascii_lowercase();

    if normalized.starts_with("http://")
        || normalized.starts_with("git://")
        || normalized.starts_with("ssh://")
    {
        report.add_finding(
            Ecosystem::Npm,
            Severity::High,
            FindingKind::NpmLockExternalSource,
            relative_path,
            format!("`{label}` resolves from `{resolved}`"),
        );
    } else if !is_default_npm_registry(&normalized) && is_remote_npm_spec(&normalized) {
        report.add_finding(
            Ecosystem::Npm,
            Severity::Medium,
            FindingKind::NpmLockExternalSource,
            relative_path,
            format!("`{label}` resolves from non-default source `{resolved}`"),
        );
    }

    if is_default_npm_registry(&normalized)
        && package
            .get("integrity")
            .and_then(|value| value.as_str())
            .is_none()
    {
        report.add_finding(
            Ecosystem::Npm,
            Severity::Medium,
            FindingKind::NpmLockMissingIntegrity,
            relative_path,
            format!("`{label}` resolves from the npm registry without an integrity hash"),
        );
    }
}

fn scan_requirements_txt(
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Python, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    for (index, line) in content.lines().enumerate() {
        let Some(requirement) = normalize_requirement_line(line) else {
            continue;
        };
        analyze_python_requirement_line(index + 1, &requirement, relative_path, report);
    }

    Ok(())
}

fn analyze_python_requirement_line(
    line_number: usize,
    requirement: &str,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let normalized = requirement.to_ascii_lowercase();
    let detail = format!("line {line_number}: `{requirement}`");

    if normalized.starts_with("--trusted-host") {
        report.add_finding(
            Ecosystem::Python,
            Severity::High,
            FindingKind::PythonTrustedHost,
            relative_path,
            detail,
        );
        return;
    }

    if is_python_index_option(&normalized) {
        let severity = if normalized.contains("http://") {
            Severity::High
        } else {
            Severity::Medium
        };
        report.add_finding(
            Ecosystem::Python,
            severity,
            FindingKind::PythonExternalIndex,
            relative_path,
            detail,
        );
        return;
    }

    if is_python_direct_reference(&normalized) {
        let severity = if normalized.contains("http://")
            || normalized.starts_with("-e ")
            || normalized.starts_with("--editable")
        {
            Severity::High
        } else {
            Severity::Medium
        };
        report.add_finding(
            Ecosystem::Python,
            severity,
            FindingKind::PythonDirectUrl,
            relative_path,
            detail,
        );
        return;
    }

    if is_probably_python_package_spec(requirement) && !is_exact_python_pin(requirement) {
        report.add_finding(
            Ecosystem::Python,
            Severity::Low,
            FindingKind::PythonUnpinnedRequirement,
            relative_path,
            detail,
        );
    }
}

fn scan_python_lock_file(
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Python, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let mut seen = BTreeSet::new();
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let normalized = trimmed.to_ascii_lowercase();
        if !seen.insert(normalized.clone()) {
            continue;
        }

        if normalized.contains("trusted-host") {
            report.add_finding(
                Ecosystem::Python,
                Severity::High,
                FindingKind::PythonTrustedHost,
                relative_path,
                format!("line {}: `{trimmed}`", index + 1),
            );
        } else if is_python_lock_direct_source_line(&normalized) {
            let severity = if normalized.contains("http://")
                || normalized.contains("ssh://")
                || normalized.contains("git+ssh")
            {
                Severity::High
            } else {
                Severity::Medium
            };
            report.add_finding(
                Ecosystem::Python,
                severity,
                FindingKind::PythonDirectUrl,
                relative_path,
                format!("line {}: `{trimmed}`", index + 1),
            );
        } else if is_python_lock_external_index_line(&normalized) {
            let severity = if normalized.contains("http://") {
                Severity::High
            } else {
                Severity::Medium
            };
            report.add_finding(
                Ecosystem::Python,
                severity,
                FindingKind::PythonExternalIndex,
                relative_path,
                format!("line {}: `{trimmed}`", index + 1),
            );
        }
    }

    Ok(())
}

fn scan_pyproject(
    root: &Path,
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Python, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let toml = match toml::from_str::<TomlValue>(&content) {
        Ok(toml) => toml,
        Err(err) => {
            report.add_finding(
                Ecosystem::Python,
                Severity::Medium,
                FindingKind::ManifestParseFailed,
                relative_path,
                format!("pyproject.toml parse error: {err}"),
            );
            return Ok(());
        }
    };

    let mut has_dependencies = false;

    if let Some(project) = toml.get("project").and_then(|value| value.as_table()) {
        if let Some(dependencies) = project
            .get("dependencies")
            .and_then(|value| value.as_array())
        {
            has_dependencies |= scan_python_dependency_array(
                "project.dependencies",
                dependencies,
                relative_path,
                report,
            );
        }

        if let Some(optional) = project
            .get("optional-dependencies")
            .and_then(|value| value.as_table())
        {
            for (group, dependencies) in optional {
                let Some(dependencies) = dependencies.as_array() else {
                    continue;
                };
                has_dependencies |= scan_python_dependency_array(
                    &format!("project.optional-dependencies.{group}"),
                    dependencies,
                    relative_path,
                    report,
                );
            }
        }
    }

    if let Some(groups) = toml
        .get("dependency-groups")
        .and_then(|value| value.as_table())
    {
        for (group, dependencies) in groups {
            let Some(dependencies) = dependencies.as_array() else {
                continue;
            };
            has_dependencies |= scan_python_dependency_array(
                &format!("dependency-groups.{group}"),
                dependencies,
                relative_path,
                report,
            );
        }
    }

    if let Some(poetry) = toml
        .get("tool")
        .and_then(|value| value.get("poetry"))
        .and_then(|value| value.as_table())
    {
        has_dependencies |= scan_poetry_dependency_tables(poetry, relative_path, report);
    }

    if let Some(uv) = toml
        .get("tool")
        .and_then(|value| value.get("uv"))
        .and_then(|value| value.as_table())
    {
        has_dependencies |= scan_uv_tables(uv, relative_path, report);
    }

    if let Some(pdm) = toml
        .get("tool")
        .and_then(|value| value.get("pdm"))
        .and_then(|value| value.as_table())
    {
        has_dependencies |= scan_pdm_tables(pdm, relative_path, report);
    }

    let package_dir = path.parent().unwrap_or(root);
    if has_dependencies && !has_any_file_in_ancestors(root, package_dir, PYTHON_LOCKFILES) {
        report.add_finding(
            Ecosystem::Python,
            Severity::Low,
            FindingKind::PythonLockMissing,
            relative_path,
            "no poetry.lock, uv.lock, pdm.lock, Pipfile.lock, or requirements.lock found",
        );
    }

    Ok(())
}

fn scan_pipfile(
    root: &Path,
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Python, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let toml = match toml::from_str::<TomlValue>(&content) {
        Ok(toml) => toml,
        Err(err) => {
            report.add_finding(
                Ecosystem::Python,
                Severity::Medium,
                FindingKind::ManifestParseFailed,
                relative_path,
                format!("Pipfile parse error: {err}"),
            );
            return Ok(());
        }
    };

    let mut has_dependencies = false;
    for section in ["packages", "dev-packages"] {
        let Some(packages) = toml.get(section).and_then(|value| value.as_table()) else {
            continue;
        };
        has_dependencies |= !packages.is_empty();
        scan_python_toml_dependencies(section, packages, relative_path, report);
    }

    let package_dir = path.parent().unwrap_or(root);
    if has_dependencies && !has_any_file_in_ancestors(root, package_dir, PYTHON_LOCKFILES) {
        report.add_finding(
            Ecosystem::Python,
            Severity::Low,
            FindingKind::PythonLockMissing,
            relative_path,
            "no Pipfile.lock or Python lockfile found",
        );
    }

    Ok(())
}

fn scan_poetry_dependency_tables(
    poetry: &toml::map::Map<String, TomlValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> bool {
    let mut has_dependencies = false;

    for section in ["dependencies", "dev-dependencies"] {
        let Some(dependencies) = poetry.get(section).and_then(|value| value.as_table()) else {
            continue;
        };
        has_dependencies |= dependencies.keys().any(|name| name != "python");
        scan_python_toml_dependencies(
            &format!("tool.poetry.{section}"),
            dependencies,
            relative_path,
            report,
        );
    }

    if let Some(groups) = poetry.get("group").and_then(|value| value.as_table()) {
        for (group, config) in groups {
            let Some(dependencies) = config
                .get("dependencies")
                .and_then(|value| value.as_table())
            else {
                continue;
            };
            has_dependencies |= dependencies.keys().any(|name| name != "python");
            scan_python_toml_dependencies(
                &format!("tool.poetry.group.{group}.dependencies"),
                dependencies,
                relative_path,
                report,
            );
        }
    }

    has_dependencies
}

fn scan_uv_tables(
    uv: &toml::map::Map<String, TomlValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> bool {
    let mut has_dependencies = false;

    if let Some(dependencies) = uv
        .get("dev-dependencies")
        .and_then(|value| value.as_array())
    {
        has_dependencies |= scan_python_dependency_array(
            "tool.uv.dev-dependencies",
            dependencies,
            relative_path,
            report,
        );
    }

    if let Some(sources) = uv.get("sources").and_then(|value| value.as_table()) {
        for (name, source) in sources {
            scan_python_source_value(
                &format!("tool.uv.sources.{name}"),
                source,
                relative_path,
                report,
            );
        }
    }

    if let Some(indexes) = uv.get("index").and_then(|value| value.as_array()) {
        for (index, source) in indexes.iter().enumerate() {
            scan_python_index_source(
                &format!("tool.uv.index[{index}]"),
                source,
                relative_path,
                report,
            );
        }
    }

    if let Some(find_links) = uv.get("find-links").and_then(|value| value.as_array()) {
        for (index, source) in find_links.iter().enumerate() {
            scan_python_index_source(
                &format!("tool.uv.find-links[{index}]"),
                source,
                relative_path,
                report,
            );
        }
    }

    has_dependencies
}

fn scan_pdm_tables(
    pdm: &toml::map::Map<String, TomlValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> bool {
    let mut has_dependencies = false;

    if let Some(groups) = pdm
        .get("dev-dependencies")
        .and_then(|value| value.as_table())
    {
        for (group, dependencies) in groups {
            let Some(dependencies) = dependencies.as_array() else {
                continue;
            };
            has_dependencies |= scan_python_dependency_array(
                &format!("tool.pdm.dev-dependencies.{group}"),
                dependencies,
                relative_path,
                report,
            );
        }
    }

    if let Some(sources) = pdm.get("source").and_then(|value| value.as_array()) {
        for (index, source) in sources.iter().enumerate() {
            scan_python_index_source(
                &format!("tool.pdm.source[{index}]"),
                source,
                relative_path,
                report,
            );
        }
    }

    has_dependencies
}

fn scan_python_dependency_array(
    section: &str,
    dependencies: &[TomlValue],
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> bool {
    let mut has_dependencies = false;
    for dependency in dependencies {
        match dependency {
            TomlValue::String(spec) => {
                has_dependencies = true;
                analyze_python_dependency_spec(section, spec, relative_path, report);
            }
            TomlValue::Table(config) => {
                if config.contains_key("include-group") {
                    continue;
                }
                has_dependencies = true;
                scan_python_source_table(section, config, relative_path, report);
            }
            _ => {}
        }
    }
    has_dependencies
}

fn scan_python_toml_dependencies(
    section: &str,
    dependencies: &toml::map::Map<String, TomlValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    for (name, dependency) in dependencies {
        if name == "python" {
            continue;
        }

        match dependency {
            TomlValue::String(spec) => analyze_python_dependency_spec(
                &format!("{section}.{name}"),
                spec,
                relative_path,
                report,
            ),
            TomlValue::Table(config) => {
                scan_python_source_table(
                    &format!("{section}.{name}"),
                    config,
                    relative_path,
                    report,
                );
                if let Some(version) = config.get("version").and_then(|value| value.as_str()) {
                    analyze_python_dependency_spec(
                        &format!("{section}.{name}"),
                        version,
                        relative_path,
                        report,
                    );
                }
            }
            _ => {}
        }
    }
}

fn scan_python_source_value(
    subject: &str,
    source: &TomlValue,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    match source {
        TomlValue::String(spec) => {
            analyze_python_dependency_spec(subject, spec, relative_path, report)
        }
        TomlValue::Table(config) => {
            scan_python_source_table(subject, config, relative_path, report)
        }
        _ => {}
    }
}

fn scan_python_source_table(
    subject: &str,
    config: &toml::map::Map<String, TomlValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    if let Some(git) = config.get("git").and_then(|value| value.as_str()) {
        report.add_finding(
            Ecosystem::Python,
            Severity::Medium,
            FindingKind::PythonDirectUrl,
            relative_path,
            format!("{subject} uses git dependency `{git}`"),
        );
    }
    if let Some(url) = config.get("url").and_then(|value| value.as_str()) {
        let normalized = url.to_ascii_lowercase();
        if normalized.starts_with("file:") {
            report.add_finding(
                Ecosystem::Python,
                Severity::Info,
                FindingKind::PythonLocalPath,
                relative_path,
                format!("{subject} uses local URL `{url}`"),
            );
            return;
        }

        let severity = if normalized.starts_with("http://") {
            Severity::High
        } else {
            Severity::Medium
        };
        report.add_finding(
            Ecosystem::Python,
            severity,
            FindingKind::PythonDirectUrl,
            relative_path,
            format!("{subject} uses URL dependency `{url}`"),
        );
    }
    if let Some(path) = config.get("path").and_then(|value| value.as_str()) {
        report.add_finding(
            Ecosystem::Python,
            Severity::Info,
            FindingKind::PythonLocalPath,
            relative_path,
            format!("{subject} uses local path `{path}`"),
        );
    }
}

fn scan_python_index_source(
    subject: &str,
    source: &TomlValue,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    match source {
        TomlValue::String(url) => add_python_index_finding(subject, url, relative_path, report),
        TomlValue::Table(config) => {
            if let Some(url) = config.get("url").and_then(|value| value.as_str()) {
                add_python_index_finding(subject, url, relative_path, report);
            }
        }
        _ => {}
    }
}

fn add_python_index_finding(
    subject: &str,
    url: &str,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let severity = if url.to_ascii_lowercase().starts_with("http://") {
        Severity::High
    } else {
        Severity::Medium
    };
    report.add_finding(
        Ecosystem::Python,
        severity,
        FindingKind::PythonExternalIndex,
        relative_path,
        format!("{subject} uses package source `{url}`"),
    );
}

fn analyze_python_dependency_spec(
    subject: &str,
    spec: &str,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let normalized = spec.to_ascii_lowercase();
    let detail = format!("{subject} uses `{spec}`");

    if is_python_local_reference(&normalized) {
        report.add_finding(
            Ecosystem::Python,
            Severity::Info,
            FindingKind::PythonLocalPath,
            relative_path,
            detail,
        );
        return;
    }

    if is_python_direct_reference(&normalized) {
        let severity = if normalized.contains("http://") {
            Severity::High
        } else {
            Severity::Medium
        };
        report.add_finding(
            Ecosystem::Python,
            severity,
            FindingKind::PythonDirectUrl,
            relative_path,
            detail,
        );
        return;
    }

    if !is_exact_python_pin(spec) && spec.trim() != "*" {
        report.add_finding(
            Ecosystem::Python,
            Severity::Low,
            FindingKind::PythonUnpinnedRequirement,
            relative_path,
            detail,
        );
    } else if spec.trim() == "*" {
        report.add_finding(
            Ecosystem::Python,
            Severity::Medium,
            FindingKind::PythonUnpinnedRequirement,
            relative_path,
            detail,
        );
    }
}

fn scan_cargo_toml(
    root: &Path,
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Rust, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let toml = match toml::from_str::<TomlValue>(&content) {
        Ok(toml) => toml,
        Err(err) => {
            report.add_finding(
                Ecosystem::Rust,
                Severity::Medium,
                FindingKind::ManifestParseFailed,
                relative_path,
                format!("Cargo.toml parse error: {err}"),
            );
            return Ok(());
        }
    };

    let mut has_dependencies = false;
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(dependencies) = toml.get(section).and_then(|value| value.as_table()) else {
            continue;
        };
        has_dependencies |= !dependencies.is_empty();
        scan_cargo_dependency_table(section, dependencies, relative_path, report);
    }

    if let Some(workspace) = toml.get("workspace").and_then(|value| value.as_table()) {
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let Some(dependencies) = workspace.get(section).and_then(|value| value.as_table())
            else {
                continue;
            };
            has_dependencies |= !dependencies.is_empty();
            scan_cargo_dependency_table(
                &format!("workspace.{section}"),
                dependencies,
                relative_path,
                report,
            );
        }
    }

    if let Some(targets) = toml.get("target").and_then(|value| value.as_table()) {
        for (target, config) in targets {
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let Some(dependencies) = config.get(section).and_then(|value| value.as_table())
                else {
                    continue;
                };
                has_dependencies |= !dependencies.is_empty();
                scan_cargo_dependency_table(
                    &format!("target.{target}.{section}"),
                    dependencies,
                    relative_path,
                    report,
                );
            }
        }
    }

    if toml.get("patch").is_some() || toml.get("replace").is_some() {
        report.add_finding(
            Ecosystem::Rust,
            Severity::Medium,
            FindingKind::RustPatchOverride,
            relative_path,
            "Cargo manifest contains [patch] or [replace] dependency overrides",
        );
    }

    let package_dir = path.parent().unwrap_or(root);
    if has_dependencies && !has_file_in_ancestors(root, package_dir, "Cargo.lock") {
        report.add_finding(
            Ecosystem::Rust,
            Severity::Low,
            FindingKind::RustLockMissing,
            relative_path,
            "no Cargo.lock found in this crate path or its ancestors",
        );
    }

    let package_build = toml
        .get("package")
        .and_then(|value| value.get("build"))
        .and_then(|value| value.as_str());
    if package_dir.join("build.rs").is_file() || package_build.is_some_and(|build| build != "false")
    {
        report.add_finding(
            Ecosystem::Rust,
            Severity::Info,
            FindingKind::RustBuildScript,
            relative_path,
            "crate has a build script that executes during cargo build",
        );
    }

    Ok(())
}

fn scan_cargo_dependency_table(
    section: &str,
    dependencies: &toml::map::Map<String, TomlValue>,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    for (name, dependency) in dependencies {
        match dependency {
            TomlValue::String(version) => {
                analyze_cargo_version(section, name, version, relative_path, report);
            }
            TomlValue::Table(config) => {
                if let Some(git) = config.get("git").and_then(|value| value.as_str()) {
                    report.add_finding(
                        Ecosystem::Rust,
                        Severity::Medium,
                        FindingKind::RustGitDependency,
                        relative_path,
                        format!("{section}.{name} uses git dependency `{git}`"),
                    );

                    if config.get("rev").is_none() {
                        let reference = config
                            .get("branch")
                            .and_then(|value| value.as_str())
                            .or_else(|| config.get("tag").and_then(|value| value.as_str()))
                            .unwrap_or("default branch");
                        report.add_finding(
                            Ecosystem::Rust,
                            Severity::High,
                            FindingKind::RustMutableGitDependency,
                            relative_path,
                            format!("{section}.{name} uses mutable git reference `{reference}`"),
                        );
                    }
                }

                if let Some(path) = config.get("path").and_then(|value| value.as_str()) {
                    report.add_finding(
                        Ecosystem::Rust,
                        Severity::Info,
                        FindingKind::RustPathDependency,
                        relative_path,
                        format!("{section}.{name} uses local path `{path}`"),
                    );
                }

                if let Some(registry) = config.get("registry").and_then(|value| value.as_str()) {
                    report.add_finding(
                        Ecosystem::Rust,
                        Severity::Medium,
                        FindingKind::RustAlternateRegistry,
                        relative_path,
                        format!("{section}.{name} uses alternate registry `{registry}`"),
                    );
                }

                if let Some(version) = config.get("version").and_then(|value| value.as_str()) {
                    analyze_cargo_version(section, name, version, relative_path, report);
                }
            }
            _ => {}
        }
    }
}

fn analyze_cargo_version(
    section: &str,
    name: &str,
    version: &str,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) {
    let normalized = version.trim();
    if normalized == "*" || normalized.contains(".*") {
        report.add_finding(
            Ecosystem::Rust,
            Severity::Medium,
            FindingKind::RustWildcardDependency,
            relative_path,
            format!("{section}.{name} uses version requirement `{version}`"),
        );
    }
}

fn scan_cargo_lock(
    path: &Path,
    relative_path: &Path,
    report: &mut SupplyChainReport,
) -> Result<()> {
    report.add_package_file(Ecosystem::Rust, relative_path.to_path_buf());

    let content = read_to_string(path)?;
    let toml = match toml::from_str::<TomlValue>(&content) {
        Ok(toml) => toml,
        Err(err) => {
            report.add_finding(
                Ecosystem::Rust,
                Severity::Medium,
                FindingKind::ManifestParseFailed,
                relative_path,
                format!("Cargo.lock parse error: {err}"),
            );
            return Ok(());
        }
    };

    let Some(packages) = toml.get("package").and_then(|value| value.as_array()) else {
        return Ok(());
    };

    for package in packages {
        let Some(package) = package.as_table() else {
            continue;
        };
        let name = package
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown");
        let version = package
            .get("version")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown");
        let label = format!("{name}@{version}");
        let Some(source) = package.get("source").and_then(|value| value.as_str()) else {
            continue;
        };
        let normalized = source.to_ascii_lowercase();

        if normalized.starts_with("git+") {
            let severity = if source.contains('#') {
                Severity::Medium
            } else {
                Severity::High
            };
            report.add_finding(
                Ecosystem::Rust,
                severity,
                FindingKind::RustGitDependency,
                relative_path,
                format!("locked package `{label}` uses `{source}`"),
            );
        }

        if normalized.starts_with("registry+http://") || normalized.starts_with("sparse+http://") {
            report.add_finding(
                Ecosystem::Rust,
                Severity::High,
                FindingKind::RustGitDependency,
                relative_path,
                format!("locked package `{label}` uses insecure registry `{source}`"),
            );
        }

        if (normalized.starts_with("registry+") || normalized.starts_with("sparse+"))
            && !is_default_cargo_registry(&normalized)
        {
            report.add_finding(
                Ecosystem::Rust,
                Severity::Medium,
                FindingKind::RustAlternateRegistry,
                relative_path,
                format!("locked package `{label}` uses alternate registry `{source}`"),
            );
        }

        if (normalized.starts_with("registry+") || normalized.starts_with("sparse+"))
            && package
                .get("checksum")
                .and_then(|value| value.as_str())
                .is_none()
        {
            report.add_finding(
                Ecosystem::Rust,
                Severity::Medium,
                FindingKind::RustLockMissingChecksum,
                relative_path,
                format!("locked package `{label}` has no checksum"),
            );
        }
    }

    Ok(())
}

fn has_any_file_in_ancestors(root: &Path, start: &Path, names: &[&str]) -> bool {
    let mut current = Some(start);
    while let Some(dir) = current {
        if names.iter().any(|name| dir.join(name).is_file()) {
            return true;
        }
        if dir == root {
            break;
        }
        current = dir.parent();
    }
    false
}

fn has_file_in_ancestors(root: &Path, start: &Path, name: &str) -> bool {
    has_any_file_in_ancestors(root, start, &[name])
}

fn is_npm_lifecycle_script(name: &str) -> bool {
    matches!(
        name,
        "preinstall" | "install" | "postinstall" | "prepublish" | "prepublishOnly" | "prepare"
    )
}

fn suspicious_script_reason(command: &str) -> Option<&'static str> {
    let normalized = command.to_ascii_lowercase();
    let downloads_remote_code = normalized.contains("curl ")
        || normalized.contains("curl\t")
        || normalized.contains("wget ")
        || normalized.contains("wget\t")
        || normalized.contains("invoke-webrequest")
        || normalized.contains(" iwr ");

    if downloads_remote_code
        && (normalized.contains("| sh")
            || normalized.contains("|sh")
            || normalized.contains("| bash")
            || normalized.contains("|bash")
            || normalized.contains("sh -c")
            || normalized.contains("bash -c"))
    {
        return Some("downloads remote content and pipes it into a shell");
    }

    if normalized.contains("powershell")
        || normalized.contains("certutil")
        || normalized.contains("frombase64string")
        || normalized.contains("base64 -d")
        || normalized.contains(" nc ")
        || normalized.contains("netcat")
    {
        return Some("contains command patterns commonly used for payload staging");
    }

    None
}

fn is_remote_npm_spec(spec: &str) -> bool {
    spec.starts_with("git+")
        || spec.starts_with("git://")
        || spec.starts_with("github:")
        || spec.starts_with("gitlab:")
        || spec.starts_with("bitbucket:")
        || spec.starts_with("http://")
        || spec.starts_with("https://")
        || spec.starts_with("ssh://")
}

fn is_default_npm_registry(resolved: &str) -> bool {
    resolved.starts_with("https://registry.npmjs.org/")
        || resolved.starts_with("https://registry.yarnpkg.com/")
}

fn is_default_cargo_registry(source: &str) -> bool {
    source == "registry+https://github.com/rust-lang/crates.io-index"
        || source == "sparse+https://index.crates.io/"
}

fn is_suspicious_npm_lock_line(line: &str) -> bool {
    if line.contains("https://registry.npmjs.org/")
        || line.contains("https://registry.yarnpkg.com/")
    {
        return false;
    }

    line.contains("http://")
        || line.contains("git://")
        || line.contains("ssh://")
        || line.contains("git+http")
        || line.contains("git+ssh")
        || line.contains("github:")
        || line.contains("gitlab:")
        || line.contains("bitbucket:")
}

fn is_npm_lock_selector_only_line(line: &str) -> bool {
    line.ends_with("\":")
        && !line.contains("git+")
        && !line.contains("http://")
        && !line.contains("git://")
        && !line.contains("ssh://")
}

fn npm_lock_package_label(package_path: &str, package: &JsonValue) -> String {
    let name = package_path
        .rsplit_once("node_modules/")
        .map(|(_, name)| name)
        .unwrap_or(package_path);
    let version = package
        .get("version")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    format!("{name}@{version}")
}

fn is_npm_text_lock_file(name: &str) -> bool {
    matches!(name, "pnpm-lock.yaml" | "yarn.lock" | "bun.lock")
}

fn is_python_requirements_file(name: &str) -> bool {
    name == "requirements.txt" || (name.starts_with("requirements-") && name.ends_with(".txt"))
}

fn normalize_requirement_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let without_inline_comment = trimmed.split(" #").next().unwrap_or(trimmed).trim();
    if without_inline_comment.is_empty()
        || without_inline_comment.starts_with("-r ")
        || without_inline_comment.starts_with("--requirement ")
        || without_inline_comment.starts_with("-c ")
        || without_inline_comment.starts_with("--constraint ")
    {
        None
    } else {
        Some(without_inline_comment.to_string())
    }
}

fn is_python_index_option(requirement: &str) -> bool {
    requirement.starts_with("--index-url")
        || requirement.starts_with("-i ")
        || requirement.starts_with("--extra-index-url")
        || requirement.starts_with("--find-links")
        || requirement.starts_with("-f ")
}

fn is_python_direct_reference(requirement: &str) -> bool {
    requirement.starts_with("-e ")
        || requirement.starts_with("--editable")
        || requirement.starts_with("git+")
        || requirement.starts_with("svn+")
        || requirement.starts_with("hg+")
        || requirement.starts_with("bzr+")
        || requirement.starts_with("http://")
        || requirement.starts_with("https://")
        || requirement.contains(" @ git+")
        || requirement.contains(" @ http://")
        || requirement.contains(" @ https://")
}

fn is_python_local_reference(requirement: &str) -> bool {
    requirement.starts_with("file:")
        || requirement.contains(" @ file:")
        || requirement.starts_with("./")
        || requirement.starts_with("../")
}

fn is_default_python_package_host(line: &str) -> bool {
    line.contains("https://pypi.org/")
        || line.contains("https://files.pythonhosted.org/")
        || line.contains("https://pypi.python.org/")
}

fn is_python_lock_external_index_line(line: &str) -> bool {
    (line.contains("index")
        || line.contains("source")
        || line.contains("\"url\"")
        || line.contains("url ="))
        && (line.contains("http://")
            || (line.contains("https://") && !is_default_python_package_host(line)))
}

fn is_python_lock_direct_source_line(line: &str) -> bool {
    line.contains("git+")
        || line.contains("ssh://")
        || line.contains("github.com")
        || line.contains("gitlab.com")
        || line.contains("bitbucket.org")
        || line.contains(" @ file:")
        || line.contains("file://")
}

fn is_probably_python_package_spec(requirement: &str) -> bool {
    requirement.chars().next().is_some_and(|character| {
        character.is_ascii_alphanumeric() || character == '_' || character == '-'
    })
}

fn is_exact_python_pin(spec: &str) -> bool {
    spec.contains("==") || spec.contains("===")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn finding_kinds(report: &SupplyChainReport) -> Vec<FindingKind> {
        report.findings.iter().map(|finding| finding.kind).collect()
    }

    #[test]
    fn detects_nested_npm_python_and_rust_manifests() {
        let temp_dir = tempfile::tempdir().unwrap();

        let web_dir = temp_dir.path().join("apps/web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("package.json"),
            r#"{
                "scripts": {
                    "postinstall": "node setup.js",
                    "verify": "curl https://example.com/install.sh | sh"
                },
                "dependencies": {
                    "left-pad": "latest",
                    "remote": "git+https://github.com/example/pkg.git",
                    "local": "file:../local"
                }
            }"#,
        )
        .unwrap();

        let python_dir = temp_dir.path().join("services/api");
        fs::create_dir_all(&python_dir).unwrap();
        fs::write(
            python_dir.join("requirements.txt"),
            "flask>=3\n--extra-index-url http://packages.example/simple\n-e git+https://github.com/example/app.git#egg=app\n",
        )
        .unwrap();

        let rust_dir = temp_dir.path().join("crates/worker");
        fs::create_dir_all(&rust_dir).unwrap();
        fs::write(
            rust_dir.join("Cargo.toml"),
            r#"[package]
name = "worker"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = "*"
helper = { git = "https://github.com/example/helper", branch = "main" }
"#,
        )
        .unwrap();

        let report = scan_supply_chain(temp_dir.path()).unwrap();
        let kinds = finding_kinds(&report);

        assert_eq!(report.package_files.len(), 3);
        assert!(kinds.contains(&FindingKind::NpmLifecycleScript));
        assert!(kinds.contains(&FindingKind::NpmSuspiciousScript));
        assert!(kinds.contains(&FindingKind::NpmLockMissing));
        assert!(kinds.contains(&FindingKind::NpmRemoteDependency));
        assert!(kinds.contains(&FindingKind::PythonExternalIndex));
        assert!(kinds.contains(&FindingKind::PythonDirectUrl));
        assert!(kinds.contains(&FindingKind::PythonUnpinnedRequirement));
        assert!(
            kinds.contains(&FindingKind::RustWildcardDependency),
            "{report:#?}"
        );
        assert!(kinds.contains(&FindingKind::RustMutableGitDependency));
        assert!(kinds.contains(&FindingKind::RustLockMissing));
    }

    #[test]
    fn scans_npm_lockfile_for_transitive_risk() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("package-lock.json"),
            r#"{
                "lockfileVersion": 3,
                "packages": {
                    "": { "name": "root" },
                    "node_modules/evil": {
                        "version": "1.0.0",
                        "resolved": "http://registry.npmjs.org/evil/-/evil-1.0.0.tgz",
                        "hasInstallScript": true
                    },
                    "node_modules/plain": {
                        "version": "1.0.0",
                        "resolved": "https://registry.npmjs.org/plain/-/plain-1.0.0.tgz"
                    }
                }
            }"#,
        )
        .unwrap();

        let report = scan_supply_chain(temp_dir.path()).unwrap();
        let kinds = finding_kinds(&report);

        assert!(kinds.contains(&FindingKind::NpmLockInstallScript));
        assert!(kinds.contains(&FindingKind::NpmLockExternalSource));
        assert!(kinds.contains(&FindingKind::NpmLockMissingIntegrity));
    }

    #[test]
    fn scans_pyproject_and_cargo_lock() {
        let temp_dir = tempfile::tempdir().unwrap();

        fs::write(
            temp_dir.path().join("pyproject.toml"),
            r#"[project]
dependencies = [
  "requests>=2",
  "tool @ git+https://github.com/example/tool.git"
]

[tool.uv.sources]
local-tool = { url = "file:///tmp/local-tool" }
"#,
        )
        .unwrap();

        fs::write(
            temp_dir.path().join("Cargo.lock"),
            r#"version = 4

[[package]]
name = "git-lib"
version = "0.1.0"
source = "git+https://github.com/example/git-lib"

[[package]]
name = "crate-lib"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "private-lib"
version = "0.1.0"
source = "registry+https://packages.example/crates-index"
"#,
        )
        .unwrap();

        let report = scan_supply_chain(temp_dir.path()).unwrap();
        let kinds = finding_kinds(&report);

        assert!(kinds.contains(&FindingKind::PythonDirectUrl), "{report:#?}");
        assert!(kinds.contains(&FindingKind::PythonUnpinnedRequirement));
        assert!(kinds.contains(&FindingKind::PythonLocalPath));
        assert!(kinds.contains(&FindingKind::PythonLockMissing));
        assert!(kinds.contains(&FindingKind::RustGitDependency));
        assert!(kinds.contains(&FindingKind::RustLockMissingChecksum));
        assert!(kinds.contains(&FindingKind::RustAlternateRegistry));
    }

    #[test]
    fn scans_text_lockfiles_for_external_npm_sources() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("pnpm-lock.yaml"),
            r#"lockfileVersion: '9.0'
packages:
  evil@git+ssh://git@github.com/example/evil.git:
    resolution: {commit: abc123}
  plain@1.0.0:
    resolution: {integrity: sha512-test}
"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("yarn.lock"),
            r#""evil@github:example/evil":
  version "1.0.0"
  resolved "github:example/evil#abc123"
"#,
        )
        .unwrap();

        let report = scan_supply_chain(temp_dir.path()).unwrap();
        let external_sources = report
            .findings
            .iter()
            .filter(|finding| finding.kind == FindingKind::NpmLockExternalSource)
            .count();

        assert_eq!(external_sources, 2, "{report:#?}");
    }

    #[test]
    fn scans_pyproject_dependency_groups_uv_and_pdm_sources() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("pyproject.toml"),
            r#"[dependency-groups]
dev = [
  "pytest>=8",
  { include-group = "lint" }
]

[tool.uv.sources]
internal = { git = "https://github.com/example/internal.git" }
local-lib = { path = "../local-lib" }

[[tool.uv.index]]
url = "http://packages.example/simple"

[tool.pdm.dev-dependencies]
docs = ["mkdocs>=1"]

[[tool.pdm.source]]
url = "https://packages.example/simple"
"#,
        )
        .unwrap();

        let report = scan_supply_chain(temp_dir.path()).unwrap();
        let kinds = finding_kinds(&report);

        assert!(kinds.contains(&FindingKind::PythonUnpinnedRequirement));
        assert!(kinds.contains(&FindingKind::PythonDirectUrl));
        assert!(kinds.contains(&FindingKind::PythonExternalIndex));
        assert!(kinds.contains(&FindingKind::PythonLocalPath));
        assert!(kinds.contains(&FindingKind::PythonLockMissing));
    }

    #[test]
    fn scans_python_lockfiles_for_direct_sources_and_external_indexes() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("poetry.lock"),
            r#"[[package]]
name = "internal"
version = "0.1.0"
source = { type = "git", url = "git+https://github.com/example/internal.git", reference = "main" }
"#,
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("Pipfile.lock"),
            r#"{
  "_meta": {
    "sources": [
      { "name": "private", "url": "http://packages.example/simple", "verify_ssl": false }
    ]
  }
}"#,
        )
        .unwrap();

        let report = scan_supply_chain(temp_dir.path()).unwrap();
        let kinds = finding_kinds(&report);

        assert!(kinds.contains(&FindingKind::PythonDirectUrl));
        assert!(kinds.contains(&FindingKind::PythonExternalIndex));
    }

    #[test]
    fn scans_rust_workspace_dependencies() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[workspace]
members = ["crates/app"]

[workspace.dependencies]
serde = "*"
helper = { git = "https://github.com/example/helper", branch = "main" }
private = { version = "1.0.0", registry = "internal" }
"#,
        )
        .unwrap();

        let report = scan_supply_chain(temp_dir.path()).unwrap();
        let kinds = finding_kinds(&report);

        assert!(kinds.contains(&FindingKind::RustWildcardDependency));
        assert!(kinds.contains(&FindingKind::RustGitDependency));
        assert!(kinds.contains(&FindingKind::RustMutableGitDependency));
        assert!(kinds.contains(&FindingKind::RustAlternateRegistry));
        assert!(kinds.contains(&FindingKind::RustLockMissing));
    }
}
