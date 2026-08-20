//! Path boundary for one fixed root: a teaching edge, not an OS sandbox.

use crate::tool_types::ToolError;
use std::path::{Component, Path, PathBuf};

/// A fixed-root, teaching-level path boundary.
#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Fixes and canonicalizes the workspace root.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, ToolError> {
        let root = root
            .as_ref()
            .canonicalize()
            .map_err(|error| ToolError::new(format!("workspace root invalid: {error}")))?;
        if !root.is_dir() {
            return Err(ToolError::new(format!(
                "workspace root is not a directory: {}",
                root.display()
            )));
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a path; an existing target is canonicalized directly, a missing
    /// one is resolved from its nearest existing ancestor so an external symlink
    /// parent cannot be used to escape the root before the target is created.
    pub fn resolve(&self, user_path: Option<&str>, must_exist: bool) -> Result<PathBuf, ToolError> {
        let raw = user_path.unwrap_or(".");
        let path = Path::new(if raw.trim().is_empty() { "." } else { raw });
        if path.components().any(|part| part == Component::ParentDir) {
            return Err(ToolError::new(format!(
                "path escapes workspace root: {user_path:?}"
            )));
        }
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let resolved = canonicalize_with_missing_tail(&candidate)?;
        if !resolved.starts_with(&self.root) {
            return Err(ToolError::new(format!(
                "path escapes workspace root: {user_path:?}"
            )));
        }
        if must_exist && !resolved.exists() {
            return Err(ToolError::new(format!(
                "path does not exist: {user_path:?}"
            )));
        }
        Ok(resolved)
    }

    /// Formats an absolute path as a POSIX-style path relative to the root.
    pub fn relative_to_root(&self, path: &Path) -> Result<String, ToolError> {
        let abs = canonicalize_with_missing_tail(path)?;
        let rel = abs
            .strip_prefix(&self.root)
            .map_err(|_| ToolError::new("path escapes workspace root"))?;
        let text = rel.to_string_lossy().replace('\\', "/");
        Ok(if text.is_empty() { ".".into() } else { text })
    }
}

/// Canonicalizes the nearest existing ancestor, then re-appends the still-missing tail.
fn canonicalize_with_missing_tail(path: &Path) -> Result<PathBuf, ToolError> {
    let mut ancestor = path.to_path_buf();
    let mut tail = Vec::new();
    while !ancestor.exists() {
        let name = ancestor
            .file_name()
            .ok_or_else(|| ToolError::new(format!("cannot resolve path: {}", path.display())))?;
        tail.push(name.to_os_string());
        if !ancestor.pop() {
            return Err(ToolError::new(format!(
                "cannot resolve path: {}",
                path.display()
            )));
        }
    }
    let mut resolved = ancestor
        .canonicalize()
        .map_err(|error| ToolError::new(format!("cannot resolve path: {error}")))?;
    for part in tail.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}
