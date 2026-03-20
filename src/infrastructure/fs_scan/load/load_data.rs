use std::path::PathBuf;

use anyhow::Result;

use crate::diff::{Hunk, compute_hunks};
use crate::model::{DiffFile, EntryStatus, LoadedDiffData};
use crate::settings::AppSettings;

use super::super::reader::{FileContent, read_file_content};
use super::status_resolve::status_for_existing_pair;

pub(crate) fn load_diff_file_with_config(file: &mut DiffFile, cfg: &AppSettings) -> Result<()> {
    if file.loaded {
        return Ok(());
    }

    let data = load_diff_data(file.left_path.clone(), file.right_path.clone(), cfg)?;
    file.apply_loaded_data(data);

    Ok(())
}

pub(crate) fn load_diff_data(
    left_path: Option<PathBuf>,
    right_path: Option<PathBuf>,
    cfg: &AppSettings,
) -> Result<LoadedDiffData> {
    let left_content = match &left_path {
        Some(path) => read_file_content(path)?,
        None => FileContent::text(String::new()),
    };
    let right_content = match &right_path {
        Some(path) => read_file_content(path)?,
        None => FileContent::text(String::new()),
    };

    let left_is_binary = left_content.is_binary;
    let right_is_binary = right_content.is_binary;
    let is_binary = left_is_binary || right_is_binary;
    let left_has_unsupported_encoding = left_content.has_unsupported_encoding;
    let right_has_unsupported_encoding = right_content.has_unsupported_encoding;
    let has_unsupported_encoding = left_has_unsupported_encoding || right_has_unsupported_encoding;
    let bytes_equal = left_content.raw == right_content.raw;
    let highlight_limited = should_limit_highlight(
        &left_content.text,
        &right_content.text,
        left_content.bytes,
        right_content.bytes,
        cfg,
    );
    let hunks = if is_binary {
        Vec::new()
    } else {
        compute_hunks(
            &left_content.text,
            &right_content.text,
            cfg.whitespace_policy(),
            cfg.line_ending_policy(),
        )
    };
    let status = derive_loaded_status(
        left_path.is_some(),
        right_path.is_some(),
        is_binary,
        bytes_equal,
        &hunks,
    );

    Ok(LoadedDiffData {
        left_text: left_content.text,
        right_text: right_content.text,
        left_bytes: left_content.bytes,
        right_bytes: right_content.bytes,
        left_is_binary,
        right_is_binary,
        is_binary,
        has_unsupported_encoding,
        left_has_unsupported_encoding,
        right_has_unsupported_encoding,
        left_has_utf8_bom: left_content.has_utf8_bom,
        right_has_utf8_bom: right_content.has_utf8_bom,
        highlight_limited,
        hunks,
        status,
    })
}

fn derive_loaded_status(
    has_left: bool,
    has_right: bool,
    is_binary: bool,
    bytes_equal: bool,
    hunks: &[Hunk],
) -> EntryStatus {
    match (has_left, has_right) {
        (true, true) => status_for_existing_pair(is_binary, bytes_equal, hunks.is_empty()),
        (true, false) => EntryStatus::Deleted,
        (false, true) => EntryStatus::Added,
        (false, false) => EntryStatus::Unchanged,
    }
}

fn should_limit_highlight(
    left_text: &str,
    right_text: &str,
    left_bytes: usize,
    right_bytes: usize,
    cfg: &AppSettings,
) -> bool {
    left_bytes.max(right_bytes) > cfg.highlight_max_bytes
        || left_text.lines().count().max(right_text.lines().count()) > cfg.highlight_max_lines
}

#[cfg(test)]
mod tests {
    use super::{derive_loaded_status, should_limit_highlight};
    use crate::model::{EntryStatus, Hunk};
    use crate::settings::AppSettings;

    #[test]
    fn derive_loaded_status_covers_presence_cases() {
        assert_eq!(
            derive_loaded_status(true, true, false, true, &[]),
            EntryStatus::Unchanged
        );
        assert_eq!(
            derive_loaded_status(true, true, false, false, &[empty_hunk()]),
            EntryStatus::Modified
        );
        assert_eq!(
            derive_loaded_status(true, false, false, false, &[]),
            EntryStatus::Deleted
        );
        assert_eq!(
            derive_loaded_status(false, true, false, false, &[]),
            EntryStatus::Added
        );
        assert_eq!(
            derive_loaded_status(true, true, true, true, &[]),
            EntryStatus::Unchanged
        );
        assert_eq!(
            derive_loaded_status(true, true, true, false, &[]),
            EntryStatus::Modified
        );
    }

    #[test]
    fn highlight_limit_uses_bytes_or_lines() {
        let cfg = AppSettings {
            highlight_max_bytes: 4,
            highlight_max_lines: 2,
            ..AppSettings::default()
        };
        assert!(should_limit_highlight("abc", "12345", 3, 5, &cfg));
        assert!(should_limit_highlight("a\nb\nc\n", "x\n", 3, 2, &cfg));
        assert!(!should_limit_highlight("a\nb\n", "x\ny\n", 4, 4, &cfg));
    }

    fn empty_hunk() -> Hunk {
        Hunk {
            old_start: 0,
            old_end: 0,
            new_start: 0,
            new_end: 0,
            old_lines: Vec::new(),
            new_lines: Vec::new(),
        }
    }
}
