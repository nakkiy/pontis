use std::path::Path;

use anyhow::Result;

use super::io::{resolve_left_path, resolve_right_path, write_with_optional_backup};
use crate::model::{DiffFile, Mode};

#[derive(Clone, Copy)]
pub(super) enum FileSide {
    Left,
    Right,
}

pub(super) fn save_file_side_if_dirty(
    file: &mut DiffFile,
    side: FileSide,
    can_write: bool,
    roots_mode: Mode,
    roots_left: &Path,
    roots_right: &Path,
    create_backup: bool,
) -> Result<usize> {
    let is_dirty = match side {
        FileSide::Left => file.left_dirty,
        FileSide::Right => file.right_dirty,
    };
    if !is_dirty || !can_write {
        return Ok(0);
    }

    let path = match side {
        FileSide::Left => resolve_left_path(roots_mode, roots_left, file),
        FileSide::Right => resolve_right_path(roots_mode, roots_right, file),
    };
    let content = match side {
        FileSide::Left => file.left_text.clone(),
        FileSide::Right => file.right_text.clone(),
    };
    let with_utf8_bom = match side {
        FileSide::Left => file.left_has_utf8_bom,
        FileSide::Right => file.right_has_utf8_bom,
    };

    write_with_optional_backup(&path, &content, with_utf8_bom, create_backup)?;

    match side {
        FileSide::Left => {
            file.left_path = Some(path);
            file.left_dirty = false;
        }
        FileSide::Right => {
            file.right_path = Some(path);
            file.right_dirty = false;
        }
    }

    Ok(1)
}
