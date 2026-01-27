//! Path extraction and validation from markdown content

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

/// Error types for path validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// Path is absolute (starts with /)
    Absolute,
    /// Path contains parent traversal (..)
    ParentTraversal,
    /// Path does not exist
    NotFound,
    /// Path is a directory, not a file
    IsDirectory,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absolute => write!(f, "absolute path not allowed"),
            Self::ParentTraversal => write!(f, "parent traversal (..) not allowed"),
            Self::NotFound => write!(f, "file not found"),
            Self::IsDirectory => write!(f, "path is a directory, not a file"),
        }
    }
}

/// Extract file path references from markdown content.
///
/// Finds single-backtick strings that look like file paths:
/// - Contains `/` OR starts with `./`
///
/// Excludes:
/// - Content inside fenced code blocks (``` ... ```)
/// - Strings without `/` that don't start with `./`
///
/// Returns deduplicated paths with leading `./` stripped.
pub fn extract_paths(content: &str) -> Vec<String> {
    let mut paths = HashSet::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        // Toggle code block state on fence markers
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip lines inside code blocks
        if in_code_block {
            continue;
        }

        // Extract backtick-enclosed strings from this line
        extract_backtick_paths(line, &mut paths);
    }

    let mut result: Vec<String> = paths.into_iter().collect();
    result.sort();
    result
}

/// Extract paths from backtick-enclosed strings in a single line
fn extract_backtick_paths(line: &str, paths: &mut HashSet<String>) {
    let mut chars = line.char_indices().peekable();

    while let Some((start_idx, ch)) = chars.next() {
        if ch == '`' {
            // Check for double/triple backtick (inline code spans with multiple backticks)
            if chars.peek().is_some_and(|(_, c)| *c == '`') {
                // Skip until we find matching closing backticks
                let mut backtick_count = 1;
                while chars.peek().is_some_and(|(_, c)| *c == '`') {
                    chars.next();
                    backtick_count += 1;
                }
                // Find closing backticks of same count
                let mut closing_count = 0;
                for (_, c) in chars.by_ref() {
                    if c == '`' {
                        closing_count += 1;
                        if closing_count == backtick_count {
                            break;
                        }
                    } else {
                        closing_count = 0;
                    }
                }
                continue;
            }

            // Single backtick - find the closing one
            let content_start = start_idx + 1;
            let mut end_idx = None;

            for (idx, c) in chars.by_ref() {
                if c == '`' {
                    end_idx = Some(idx);
                    break;
                }
            }

            if let Some(end) = end_idx {
                let content = &line[content_start..end];
                if is_path_like(content) {
                    let normalized = normalize_path(content);
                    paths.insert(normalized);
                }
            }
        }
    }
}

/// Check if a string looks like a file path
fn is_path_like(s: &str) -> bool {
    // Must contain `/` or start with `./`
    s.contains('/') || s.starts_with("./")
}

/// Normalize a path by stripping leading `./`
fn normalize_path(path: &str) -> String {
    path.strip_prefix("./").unwrap_or(path).to_string()
}

/// Validate and normalize a path reference.
///
/// Returns the normalized path or an error explaining why it's invalid.
///
/// Validation rules:
/// - Reject absolute paths (starting with `/`)
/// - Reject paths containing `..` (parent traversal)
/// - Reject paths that don't exist
/// - Reject paths that are directories
pub fn validate_path(path: &str, project_root: &Path) -> Result<String, PathError> {
    // Check for absolute path
    if path.starts_with('/') {
        return Err(PathError::Absolute);
    }

    // Check for parent traversal
    if path.contains("..") {
        return Err(PathError::ParentTraversal);
    }

    // Normalize the path
    let normalized = normalize_path(path);

    // Resolve against project root and check existence
    let full_path = project_root.join(&normalized);

    if !full_path.exists() {
        return Err(PathError::NotFound);
    }

    if full_path.is_dir() {
        return Err(PathError::IsDirectory);
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Path extraction tests

    #[test]
    fn test_extract_simple_path() {
        let content = "text with `src/foo.rs`";
        assert_eq!(extract_paths(content), vec!["src/foo.rs"]);
    }

    #[test]
    fn test_extract_dot_slash_path() {
        let content = "with `./src/bar.rs`";
        assert_eq!(extract_paths(content), vec!["src/bar.rs"]);
    }

    #[test]
    fn test_skip_code_blocks() {
        let content = "```rust\n`ignored.rs`\n```";
        assert!(extract_paths(content).is_empty());
    }

    #[test]
    fn test_skip_non_paths() {
        let content = "`grep` and `--help`";
        assert!(extract_paths(content).is_empty());
    }

    #[test]
    fn test_multiple_paths() {
        let content = "multiple `a/b.rs` and `c/d.rs`";
        let paths = extract_paths(content);
        assert_eq!(paths, vec!["a/b.rs", "c/d.rs"]);
    }

    #[test]
    fn test_deduplicate_paths() {
        let content = "`a/b.rs` and `a/b.rs`";
        assert_eq!(extract_paths(content), vec!["a/b.rs"]);
    }

    #[test]
    fn test_mixed_content() {
        let content = r"
# Document

Use `src/main.rs` for the entry point.

```rust
// This `src/lib.rs` should be ignored
fn main() {}
```

Also see `src/config.rs` and `grep` command.
";
        let paths = extract_paths(content);
        assert_eq!(paths, vec!["src/config.rs", "src/main.rs"]);
    }

    #[test]
    fn test_double_backticks_ignored() {
        let content = "use ``src/path.rs`` for something";
        assert!(extract_paths(content).is_empty());
    }

    #[test]
    fn test_path_with_extension() {
        let content = "See `docs/guide.md` and `src/lib.rs`";
        let paths = extract_paths(content);
        assert_eq!(paths, vec!["docs/guide.md", "src/lib.rs"]);
    }

    // Path validation tests

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/exists.rs"), "// content").unwrap();
        fs::create_dir_all(dir.path().join("src/subdir")).unwrap();
        dir
    }

    #[test]
    fn test_validate_absolute_path() {
        let dir = setup_test_dir();
        assert_eq!(
            validate_path("/etc/passwd", dir.path()),
            Err(PathError::Absolute)
        );
    }

    #[test]
    fn test_validate_parent_traversal() {
        let dir = setup_test_dir();
        assert_eq!(
            validate_path("../escape.rs", dir.path()),
            Err(PathError::ParentTraversal)
        );
    }

    #[test]
    fn test_validate_not_found() {
        let dir = setup_test_dir();
        assert_eq!(
            validate_path("src/missing.rs", dir.path()),
            Err(PathError::NotFound)
        );
    }

    #[test]
    fn test_validate_is_directory() {
        let dir = setup_test_dir();
        assert_eq!(
            validate_path("src/subdir", dir.path()),
            Err(PathError::IsDirectory)
        );
    }

    #[test]
    fn test_validate_existing_file() {
        let dir = setup_test_dir();
        assert_eq!(
            validate_path("src/exists.rs", dir.path()),
            Ok("src/exists.rs".to_string())
        );
    }

    #[test]
    fn test_validate_normalizes_dot_slash() {
        let dir = setup_test_dir();
        assert_eq!(
            validate_path("./src/exists.rs", dir.path()),
            Ok("src/exists.rs".to_string())
        );
    }
}
