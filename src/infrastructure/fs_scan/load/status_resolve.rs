use std::path::PathBuf;

use anyhow::Result;

use crate::diff::texts_equal_fast;
use crate::model::EntryStatus;
use crate::settings::AppSettings;

use super::super::reader::{FileContent, read_file_content};

pub(crate) fn resolve_status_only(
    left_path: Option<PathBuf>,
    right_path: Option<PathBuf>,
    cfg: &AppSettings,
) -> Result<EntryStatus> {
    let left_content = match &left_path {
        Some(path) => read_file_content(path)?,
        None => FileContent::text(String::new()),
    };
    let right_content = match &right_path {
        Some(path) => read_file_content(path)?,
        None => FileContent::text(String::new()),
    };

    let is_binary = left_content.is_binary || right_content.is_binary;
    let bytes_equal = left_content.raw == right_content.raw;
    let text_equal = texts_equal_fast(
        &left_content.text,
        &right_content.text,
        cfg.whitespace(),
        cfg.line_endings(),
    );
    Ok(derive_status_only(
        left_path.is_some(),
        right_path.is_some(),
        is_binary,
        bytes_equal,
        text_equal,
    ))
}

fn derive_status_only(
    has_left: bool,
    has_right: bool,
    is_binary: bool,
    bytes_equal: bool,
    text_equal: bool,
) -> EntryStatus {
    match (has_left, has_right) {
        (true, true) => status_for_existing_pair(is_binary, bytes_equal, text_equal),
        (true, false) => EntryStatus::Deleted,
        (false, true) => EntryStatus::Added,
        (false, false) => EntryStatus::Unchanged,
    }
}

pub(crate) fn status_for_existing_pair(
    is_binary: bool,
    bytes_equal: bool,
    non_binary_equal: bool,
) -> EntryStatus {
    if is_binary {
        if bytes_equal {
            EntryStatus::Unchanged
        } else {
            EntryStatus::Modified
        }
    } else if non_binary_equal {
        EntryStatus::Unchanged
    } else {
        EntryStatus::Modified
    }
}

#[cfg(test)]
mod tests {
    use super::derive_status_only;
    use crate::diff::{LineEndingPolicy, WhitespacePolicy, texts_equal_fast};
    use crate::model::EntryStatus;

    #[test]
    fn derive_status_only_covers_presence_cases() {
        assert_eq!(
            derive_status_only(true, true, false, true, true),
            EntryStatus::Unchanged
        );
        assert_eq!(
            derive_status_only(true, true, false, false, false),
            EntryStatus::Modified
        );
        assert_eq!(
            derive_status_only(true, false, false, false, false),
            EntryStatus::Deleted
        );
        assert_eq!(
            derive_status_only(false, true, false, false, false),
            EntryStatus::Added
        );
        assert_eq!(
            derive_status_only(true, true, true, true, false),
            EntryStatus::Unchanged
        );
        assert_eq!(
            derive_status_only(true, true, true, false, true),
            EntryStatus::Modified
        );
    }

    #[test]
    fn texts_equal_fast_respects_whitespace_policy() {
        let left = "a  b\n\tc\n";
        let right = "ab\nc\n";
        assert!(!texts_equal_fast(
            left,
            right,
            WhitespacePolicy::Compare,
            LineEndingPolicy::Compare,
        ));
        assert!(texts_equal_fast(
            left,
            right,
            WhitespacePolicy::Ignore,
            LineEndingPolicy::Compare,
        ));
    }

    #[test]
    fn texts_equal_fast_can_ignore_line_endings() {
        let left = "a\r\nb\r\n";
        let right = "a\nb\n";
        assert!(!texts_equal_fast(
            left,
            right,
            WhitespacePolicy::Compare,
            LineEndingPolicy::Compare,
        ));
        assert!(texts_equal_fast(
            left,
            right,
            WhitespacePolicy::Compare,
            LineEndingPolicy::Ignore,
        ));
    }
}
