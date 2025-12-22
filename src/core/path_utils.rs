use std::path::{Path, PathBuf};

/// 檢查 child 是否是 parent 的子路徑
pub fn is_subpath(child: &Path, parent: &Path) -> bool {
    child.starts_with(parent) && child != parent
}

/// 過濾掉被其他路徑包含的子路徑
///
/// 例如：
/// - 如果列表中有 `/a/b` 和 `/a/b/c`，則只保留 `/a/b`
/// - 如果列表中有 `/a/b/c` 和 `/a/d`，則兩者都保留
pub fn filter_subpaths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    if paths.is_empty() {
        return paths;
    }

    let mut sorted_paths = paths;
    sorted_paths.sort();

    let mut filtered = Vec::new();

    for path in sorted_paths {
        let is_child = filtered
            .iter()
            .any(|parent: &PathBuf| is_subpath(&path, parent));

        if !is_child {
            filtered.push(path);
        }
    }

    filtered
}

/// 統計有多少子路徑被過濾掉
#[allow(dead_code)]
pub fn count_filtered_subpaths(original: &[PathBuf], filtered: &[PathBuf]) -> usize {
    original.len().saturating_sub(filtered.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_subpath() {
        let parent = PathBuf::from("/a/b");
        let child = PathBuf::from("/a/b/c");
        let sibling = PathBuf::from("/a/d");
        let same = PathBuf::from("/a/b");

        assert!(is_subpath(&child, &parent));
        assert!(!is_subpath(&sibling, &parent));
        assert!(!is_subpath(&same, &parent));
        assert!(!is_subpath(&parent, &child));
    }

    #[test]
    fn test_filter_subpaths_basic() {
        let paths = vec![
            PathBuf::from("/a/b"),
            PathBuf::from("/a/b/c"),
            PathBuf::from("/a/b/c/d"),
        ];

        let filtered = filter_subpaths(paths);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], PathBuf::from("/a/b"));
    }

    #[test]
    fn test_filter_subpaths_multiple_trees() {
        let paths = vec![
            PathBuf::from("/a/b"),
            PathBuf::from("/a/b/c"),
            PathBuf::from("/x/y"),
            PathBuf::from("/x/y/z"),
        ];

        let filtered = filter_subpaths(paths);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&PathBuf::from("/a/b")));
        assert!(filtered.contains(&PathBuf::from("/x/y")));
    }

    #[test]
    fn test_filter_subpaths_no_overlap() {
        let paths = vec![
            PathBuf::from("/a/b"),
            PathBuf::from("/c/d"),
            PathBuf::from("/e/f"),
        ];

        let filtered = filter_subpaths(paths.clone());
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered, paths);
    }

    #[test]
    fn test_filter_subpaths_empty() {
        let paths: Vec<PathBuf> = vec![];
        let filtered = filter_subpaths(paths);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_subpaths_complex() {
        let paths = vec![
            PathBuf::from("/project/.terragrunt-cache"),
            PathBuf::from("/project/.terragrunt-cache/sub1/.terraform"),
            PathBuf::from("/project/.terragrunt-cache/sub1/.terraform.lock.hcl"),
            PathBuf::from("/project/module/.terraform"),
            PathBuf::from("/project/module/.terraform.lock.hcl"),
        ];

        let filtered = filter_subpaths(paths);
        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&PathBuf::from("/project/.terragrunt-cache")));
        assert!(filtered.contains(&PathBuf::from("/project/module/.terraform")));
        assert!(filtered.contains(&PathBuf::from("/project/module/.terraform.lock.hcl")));
    }

    #[test]
    fn test_count_filtered_subpaths() {
        let original = vec![
            PathBuf::from("/a/b"),
            PathBuf::from("/a/b/c"),
            PathBuf::from("/a/b/d"),
        ];

        let filtered = filter_subpaths(original.clone());
        let count = count_filtered_subpaths(&original, &filtered);
        assert_eq!(count, 2);
    }
}
