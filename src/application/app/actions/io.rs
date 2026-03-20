use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::model::{DiffFile, Mode};
use crate::text::encode_utf8_for_write;

pub(super) fn resolve_left_path(mode: Mode, root_left: &Path, file: &DiffFile) -> PathBuf {
    if let Some(path) = &file.left_path {
        return path.clone();
    }
    match mode {
        Mode::File => root_left.to_path_buf(),
        Mode::Directory => root_left.join(&file.rel_path),
    }
}

pub(super) fn resolve_right_path(mode: Mode, root_right: &Path, file: &DiffFile) -> PathBuf {
    if let Some(path) = &file.right_path {
        return path.clone();
    }
    match mode {
        Mode::File => root_right.to_path_buf(),
        Mode::Directory => root_right.join(&file.rel_path),
    }
}

pub(super) fn write_with_optional_backup(
    path: &Path,
    content: &str,
    with_utf8_bom: bool,
    backup_on_save: bool,
) -> Result<()> {
    ensure_parent(path)?;

    if backup_on_save && path.exists() {
        let backup_path = backup_path_for(path);
        fs::copy(path, &backup_path).with_context(|| {
            format!(
                "failed to create backup {} from {}",
                backup_path.display(),
                path.display()
            )
        })?;
    }

    let bytes = encode_utf8_for_write(content, with_utf8_bom);
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent dir {}", parent.display()))?;
    }
    Ok(())
}

fn backup_path_for(path: &Path) -> PathBuf {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        return path.with_file_name(format!("{name}.bak"));
    }
    path.with_extension("bak")
}
