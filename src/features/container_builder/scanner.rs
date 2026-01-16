use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scan for Dockerfiles in the given directory
pub fn scan_dockerfiles(root: &Path) -> Vec<PathBuf> {
    let mut dockerfiles = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip hidden directories and common ignored directories
        if should_skip_path(path) {
            continue;
        }

        if is_dockerfile(path) {
            dockerfiles.push(path.to_path_buf());
        }
    }

    // Sort by path for consistent ordering
    dockerfiles.sort();

    dockerfiles
}

/// Check if a path should be skipped during scanning
fn should_skip_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return true;
    };

    // Skip hidden files/directories
    if file_name.starts_with('.') {
        return true;
    }

    // Skip common build/dependency directories
    let skip_dirs = [
        "node_modules",
        "target",
        "vendor",
        "dist",
        "build",
        ".git",
        ".terraform",
        "__pycache__",
        "venv",
        ".venv",
    ];

    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
            if skip_dirs.contains(&name) {
                return true;
            }
        }
    }

    false
}

/// Check if a file is a Dockerfile
fn is_dockerfile(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    // Match common Dockerfile patterns
    let dockerfile_patterns = ["Dockerfile", "dockerfile", "Containerfile", "containerfile"];

    // Exact match
    if dockerfile_patterns.contains(&file_name) {
        return true;
    }

    // Match Dockerfile.* patterns (e.g., Dockerfile.dev, Dockerfile.prod)
    if file_name.starts_with("Dockerfile.") || file_name.starts_with("dockerfile.") {
        return true;
    }

    // Match Containerfile.* patterns
    if file_name.starts_with("Containerfile.") || file_name.starts_with("containerfile.") {
        return true;
    }

    // Match *.dockerfile patterns
    if file_name.ends_with(".dockerfile") || file_name.ends_with(".Dockerfile") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_is_dockerfile() {
        assert!(is_dockerfile_name("Dockerfile"));
        assert!(is_dockerfile_name("dockerfile"));
        assert!(is_dockerfile_name("Containerfile"));
        assert!(is_dockerfile_name("Dockerfile.dev"));
        assert!(is_dockerfile_name("Dockerfile.prod"));
        assert!(is_dockerfile_name("app.dockerfile"));
        assert!(!is_dockerfile_name("README.md"));
        assert!(!is_dockerfile_name("docker-compose.yml"));
    }

    fn is_dockerfile_name(name: &str) -> bool {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join(name);
        File::create(&path).unwrap();
        is_dockerfile(&path)
    }

    #[test]
    fn test_scan_dockerfiles() {
        let temp_dir = tempdir().unwrap();

        // Create test files
        File::create(temp_dir.path().join("Dockerfile")).unwrap();
        File::create(temp_dir.path().join("Dockerfile.dev")).unwrap();
        File::create(temp_dir.path().join("README.md")).unwrap();

        // Create subdirectory with Dockerfile
        let sub_dir = temp_dir.path().join("services").join("api");
        fs::create_dir_all(&sub_dir).unwrap();
        File::create(sub_dir.join("Dockerfile")).unwrap();

        let dockerfiles = scan_dockerfiles(temp_dir.path());

        assert_eq!(dockerfiles.len(), 3);
    }

    #[test]
    fn test_skip_node_modules() {
        let temp_dir = tempdir().unwrap();

        // Create Dockerfile in node_modules (should be skipped)
        let node_modules = temp_dir.path().join("node_modules").join("some-package");
        fs::create_dir_all(&node_modules).unwrap();
        File::create(node_modules.join("Dockerfile")).unwrap();

        // Create regular Dockerfile
        File::create(temp_dir.path().join("Dockerfile")).unwrap();

        let dockerfiles = scan_dockerfiles(temp_dir.path());

        assert_eq!(dockerfiles.len(), 1);
    }
}
