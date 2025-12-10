use std::path::{Path, PathBuf};

// 路徑工具模組 - 提供路徑相關的工具函數

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

    // 先排序，這樣父路徑會在子路徑前面
    let mut sorted_paths = paths;
    sorted_paths.sort();

    let mut filtered = Vec::new();

    for path in sorted_paths {
        // 檢查這個路徑是否是已經在 filtered 中的任何路徑的子路徑
        let is_child = filtered
            .iter()
            .any(|parent: &PathBuf| is_subpath(&path, parent));

        // 只添加不是任何已存在路徑的子路徑的項目
        if !is_child {
            filtered.push(path);
        }
    }

    filtered
}

/// 統計有多少子路徑被過濾掉
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
        assert!(!is_subpath(&same, &parent)); // 相同路徑不算子路徑
        assert!(!is_subpath(&parent, &child)); // 父不是子的子路徑
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
        assert_eq!(filtered.len(), 0);
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
        // 應該只保留 /project/.terragrunt-cache 和 /project/module/.terraform, /project/module/.terraform.lock.hcl
        // 因為前兩個是 .terragrunt-cache 的子項目
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
        assert_eq!(count, 2); // c 和 d 被過濾掉
    }
}
