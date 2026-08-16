//! 工作区路径约束与同目录原子替换：教学级边界，不是 OS 沙箱。

use crate::tool_types::ToolError;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 固定 root 的教学级路径边界。
#[derive(Clone, Debug)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// 固定并 canonicalize 工作区根目录。
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

    /// 返回已经 canonicalize 的根目录。
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 解析路径；现有目标查真实路径，不存在目标查最近的现有父目录。
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

    /// 以 POSIX 风格返回工作区相对路径。
    pub fn relative_to_root(&self, path: &Path) -> Result<String, ToolError> {
        let abs = canonicalize_with_missing_tail(path)?;
        let rel = abs
            .strip_prefix(&self.root)
            .map_err(|_| ToolError::new("path escapes workspace root"))?;
        let text = rel.to_string_lossy().replace('\\', "/");
        Ok(if text.is_empty() { ".".into() } else { text })
    }

    /// 在目标同目录完整写临时文件，再 rename 替换目标。
    pub fn atomic_write(&self, path: &Path, bytes: &[u8]) -> Result<(), ToolError> {
        let parent = path
            .parent()
            .ok_or_else(|| ToolError::new("target has no parent directory"))?;
        let checked_parent = self.resolve(Some(&parent.to_string_lossy()), false)?;
        std::fs::create_dir_all(&checked_parent)
            .map_err(|error| ToolError::new(error.to_string()))?;
        let checked_path = self.resolve(Some(&path.to_string_lossy()), false)?;
        let file_name = checked_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ToolError::new("target file name is not valid UTF-8"))?;
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp =
            checked_parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));

        let result = (|| {
            let mut handle = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .map_err(|error| ToolError::new(error.to_string()))?;
            handle
                .write_all(bytes)
                .and_then(|_| handle.sync_all())
                .map_err(|error| ToolError::new(error.to_string()))?;
            std::fs::rename(&temp, &checked_path).map_err(|error| ToolError::new(error.to_string()))
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temp);
        }
        result
    }
}

/// Canonicalize 最近的现有祖先，再原样接回尚不存在的尾部。
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
