use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{DiffContent, DiffFile, EntryStatus, LoadedDiffData, Mode, Roots};

pub(super) fn sample_diff_file() -> DiffFile {
    DiffFile::new(
        PathBuf::from("a.txt"),
        Some(PathBuf::from("/tmp/l/a.txt")),
        Some(PathBuf::from("/tmp/r/a.txt")),
        DiffContent {
            left_text: "left\n".to_string(),
            right_text: "right\n".to_string(),
            left_bytes: 5,
            right_bytes: 6,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Modified,
    )
}

pub(super) fn diff_file_with_leading_context(prefix_lines: usize) -> DiffFile {
    let left_text = format!("{}before\nold\nafter\n", "same\n".repeat(prefix_lines));
    let right_text = format!("{}before\nnew\nafter\n", "same\n".repeat(prefix_lines));

    DiffFile::new(
        PathBuf::from("late-diff.txt"),
        Some(PathBuf::from("/tmp/l/late-diff.txt")),
        Some(PathBuf::from("/tmp/r/late-diff.txt")),
        DiffContent {
            left_bytes: left_text.len(),
            right_bytes: right_text.len(),
            left_text,
            right_text,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Modified,
    )
}

pub(super) fn multi_hunk_file_with_expanding_first_hunk() -> DiffFile {
    let left_text = "a\nb\nc\nd\ne\n".to_string();
    let right_text = "a\nx\ny\nc\nz\ne\n".to_string();

    DiffFile::new(
        PathBuf::from("multi-hunk.txt"),
        Some(PathBuf::from("/tmp/l/multi-hunk.txt")),
        Some(PathBuf::from("/tmp/r/multi-hunk.txt")),
        DiffContent {
            left_bytes: left_text.len(),
            right_bytes: right_text.len(),
            left_text,
            right_text,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Modified,
    )
}

pub(super) fn unchanged_loaded_file(name: &str) -> DiffFile {
    let text = "same\n".to_string();

    DiffFile::new(
        PathBuf::from(name),
        Some(PathBuf::from(format!("/tmp/l/{name}"))),
        Some(PathBuf::from(format!("/tmp/r/{name}"))),
        DiffContent {
            left_bytes: text.len(),
            right_bytes: text.len(),
            left_text: text.clone(),
            right_text: text,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Unchanged,
    )
}

pub(super) fn added_loaded_file(name: &str) -> DiffFile {
    let text = "new\n".to_string();

    DiffFile::new(
        PathBuf::from(name),
        None,
        Some(PathBuf::from(format!("/tmp/r/{name}"))),
        DiffContent {
            left_bytes: 0,
            right_bytes: text.len(),
            left_text: String::new(),
            right_text: text,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Added,
    )
}

pub(super) fn deleted_loaded_file(name: &str) -> DiffFile {
    let text = "old\n".to_string();

    DiffFile::new(
        PathBuf::from(name),
        Some(PathBuf::from(format!("/tmp/l/{name}"))),
        None,
        DiffContent {
            left_bytes: text.len(),
            right_bytes: 0,
            left_text: text,
            right_text: String::new(),
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Deleted,
    )
}

pub(super) fn unloaded_diff_file_with_leading_context(
    name: &str,
    prefix_lines: usize,
) -> (DiffFile, LoadedDiffData) {
    let left_text = format!("{}before\nold\nafter\n", "same\n".repeat(prefix_lines));
    let right_text = format!("{}before\nnew\nafter\n", "same\n".repeat(prefix_lines));

    let file = DiffFile::new_unloaded(
        PathBuf::from(name),
        Some(PathBuf::from(format!("/tmp/l/{name}"))),
        Some(PathBuf::from(format!("/tmp/r/{name}"))),
        left_text.len(),
        right_text.len(),
        false,
        EntryStatus::Modified,
    );

    let loaded = LoadedDiffData {
        left_text: left_text.clone(),
        right_text: right_text.clone(),
        left_bytes: left_text.len(),
        right_bytes: right_text.len(),
        left_is_binary: false,
        right_is_binary: false,
        is_binary: false,
        has_unsupported_encoding: false,
        left_has_unsupported_encoding: false,
        right_has_unsupported_encoding: false,
        left_has_utf8_bom: false,
        right_has_utf8_bom: false,
        highlight_limited: false,
        hunks: crate::diff::compute_hunks(
            &left_text,
            &right_text,
            crate::diff::WhitespacePolicy::Compare,
            crate::diff::LineEndingPolicy::Compare,
        ),
        status: EntryStatus::Modified,
    };

    (file, loaded)
}

pub(super) fn sample_roots() -> Roots {
    Roots {
        left: PathBuf::from("/tmp/l"),
        right: PathBuf::from("/tmp/r"),
        mode: Mode::Directory,
        left_label: None,
        right_label: None,
    }
}

pub(super) fn unique_temp_root(prefix: &str) -> PathBuf {
    let uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{uniq}"))
}
