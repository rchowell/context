use crate::error::{ContextError, Result};
use std::path::{Path, PathBuf};

pub const CONTEXT_DIR_NAME: &str = ".context";

/// Find .context by searching upward from the given path
pub fn discover_root(from: &Path) -> Result<PathBuf> {
    let mut current = from.canonicalize().ok();

    while let Some(dir) = current {
        let candidate = dir.join(CONTEXT_DIR_NAME);
        if candidate.is_dir() {
            return Ok(candidate);
        }
        current = dir.parent().map(Path::to_path_buf);
    }

    Err(ContextError::NotARepository)
}

/// Convenience wrapper using CWD
pub fn discover_root_from_cwd() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    discover_root(&cwd)
}
