use std::fs;
use std::path::Path;

use anyhow::Result;

use super::GitCompareMode;
use crate::diff::texts_equal;
use crate::model::{DiffContent, DiffFile, EntryStatus};
use crate::settings::AppSettings;
use crate::text::{DecodedKind, decode_bytes_for_diff};

use super::super::command::{
    GitEntryChange, GitStatusEntry, git_show_index_if_exists, git_show_rev_if_exists,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct EntryPresence {
    pub(super) left_exists: bool,
    pub(super) right_exists: bool,
}

struct EntryBytes {
    presence: EntryPresence,
    left: Vec<u8>,
    right: Vec<u8>,
}

struct DecodedEntryContent {
    content: DiffContent,
    raw_bytes_equal: bool,
    utf8_text_equal: bool,
}

pub(super) fn build_files_from_entries(
    repo_root: &Path,
    cfg: &AppSettings,
    mode: &GitCompareMode,
    left_rev: &str,
    right_rev: Option<&str>,
    entries: Vec<GitStatusEntry>,
) -> Result<Vec<DiffFile>> {
    let mut files = Vec::with_capacity(entries.len());
    for entry in entries {
        files.push(build_entry_diff(
            repo_root, &entry, cfg, mode, left_rev, right_rev,
        )?);
    }
    Ok(files)
}

fn build_entry_diff(
    repo_root: &Path,
    e: &GitStatusEntry,
    cfg: &AppSettings,
    mode: &GitCompareMode,
    left_rev: &str,
    right_rev: Option<&str>,
) -> Result<DiffFile> {
    let rel = e.path.clone();
    let work_path = repo_root.join(&rel);
    let left_rel = e.original_path.as_ref().unwrap_or(&rel);
    let left_path = repo_root.join(left_rel);
    let entry_bytes = load_entry_bytes(
        repo_root, left_rev, right_rev, left_rel, &rel, &work_path, mode,
    )?;
    let decoded = decode_entry_content(entry_bytes.left, entry_bytes.right, cfg);
    let status = derive_status(
        entry_bytes.presence,
        decoded.raw_bytes_equal,
        decoded.utf8_text_equal,
        e.change == GitEntryChange::Renamed,
    );

    let mut file = DiffFile::new_with_policies(
        rel,
        if entry_bytes.presence.left_exists {
            Some(left_path)
        } else {
            None
        },
        if entry_bytes.presence.right_exists {
            Some(work_path)
        } else {
            None
        },
        decoded.content,
        status,
        cfg.compare_policies,
    );
    if e.change == GitEntryChange::Renamed
        && let Some(original_rel_path) = e.original_path.clone()
    {
        file.set_original_rel_path(original_rel_path);
    }
    Ok(file)
}

fn load_entry_bytes(
    repo_root: &Path,
    left_rev: &str,
    right_rev: Option<&str>,
    left_rel: &Path,
    rel: &Path,
    work_path: &Path,
    mode: &GitCompareMode,
) -> Result<EntryBytes> {
    let left_opt = git_show_rev_if_exists(repo_root, left_rev, left_rel)?;
    let right_opt = match mode {
        GitCompareMode::WorkingTree => fs::read(work_path).ok(),
        GitCompareMode::Staged => git_show_index_if_exists(repo_root, rel)?,
        GitCompareMode::RevisionPair { .. } => {
            git_show_rev_if_exists(repo_root, right_rev.expect("revision pair right rev"), rel)?
        }
    };
    let presence = EntryPresence {
        left_exists: left_opt.is_some(),
        right_exists: right_opt.is_some(),
    };
    Ok(EntryBytes {
        presence,
        left: left_opt.unwrap_or_default(),
        right: right_opt.unwrap_or_default(),
    })
}

fn decode_entry_content(
    left_bytes: Vec<u8>,
    right_bytes: Vec<u8>,
    cfg: &AppSettings,
) -> DecodedEntryContent {
    let raw_bytes_equal = left_bytes == right_bytes;
    let left_decoded = decode_bytes_for_diff(left_bytes);
    let right_decoded = decode_bytes_for_diff(right_bytes);
    let left_is_utf8 = left_decoded.kind == DecodedKind::TextUtf8;
    let right_is_utf8 = right_decoded.kind == DecodedKind::TextUtf8;
    let left_is_binary = !left_is_utf8;
    let right_is_binary = !right_is_utf8;
    let left_unsupported = left_decoded.kind == DecodedKind::UnsupportedEncoding;
    let right_unsupported = right_decoded.kind == DecodedKind::UnsupportedEncoding;

    let utf8_text_equal = left_is_utf8
        && right_is_utf8
        && texts_equal(
            &left_decoded.text,
            &right_decoded.text,
            cfg.whitespace_policy(),
            cfg.line_ending_policy(),
        );
    let highlight_limited = left_decoded.bytes.max(right_decoded.bytes) > cfg.highlight_max_bytes
        || left_decoded
            .text
            .lines()
            .count()
            .max(right_decoded.text.lines().count())
            > cfg.highlight_max_lines;

    DecodedEntryContent {
        raw_bytes_equal,
        utf8_text_equal,
        content: DiffContent {
            left_text: left_decoded.text,
            right_text: right_decoded.text,
            left_bytes: left_decoded.bytes,
            right_bytes: right_decoded.bytes,
            left_is_binary,
            right_is_binary,
            is_binary: left_is_binary || right_is_binary,
            has_unsupported_encoding: left_unsupported || right_unsupported,
            left_has_unsupported_encoding: left_unsupported,
            right_has_unsupported_encoding: right_unsupported,
            left_has_utf8_bom: left_decoded.has_utf8_bom,
            right_has_utf8_bom: right_decoded.has_utf8_bom,
            highlight_limited,
        },
    }
}

pub(super) fn derive_status(
    presence: EntryPresence,
    raw_bytes_equal: bool,
    utf8_text_equal: bool,
    renamed: bool,
) -> EntryStatus {
    match (presence.left_exists, presence.right_exists) {
        (true, true) => {
            if renamed {
                EntryStatus::Renamed
            } else if utf8_text_equal || raw_bytes_equal {
                EntryStatus::Unchanged
            } else {
                EntryStatus::Modified
            }
        }
        (true, false) => EntryStatus::Deleted,
        (false, true) => EntryStatus::Added,
        (false, false) => EntryStatus::Unchanged,
    }
}
