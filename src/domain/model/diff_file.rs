use std::path::PathBuf;

use super::{EntryStatus, Hunk, LoadedDiffData};

#[derive(Debug, Clone)]
pub struct DiffFile {
    pub rel_path: PathBuf,
    pub original_rel_path: Option<PathBuf>,
    pub left_path: Option<PathBuf>,
    pub right_path: Option<PathBuf>,
    pub left_text: String,
    pub right_text: String,
    pub hunks: Vec<Hunk>,
    pub status: EntryStatus,
    pub left_is_binary: bool,
    pub right_is_binary: bool,
    pub is_binary: bool,
    pub has_unsupported_encoding: bool,
    pub left_has_unsupported_encoding: bool,
    pub right_has_unsupported_encoding: bool,
    pub left_has_utf8_bom: bool,
    pub right_has_utf8_bom: bool,
    pub left_bytes: usize,
    pub right_bytes: usize,
    pub highlight_limited: bool,
    pub loaded: bool,
    pub left_dirty: bool,
    pub right_dirty: bool,
}

impl DiffFile {
    pub fn new_unloaded(
        rel_path: PathBuf,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        left_bytes: usize,
        right_bytes: usize,
        highlight_limited: bool,
        status: EntryStatus,
    ) -> Self {
        Self {
            rel_path,
            original_rel_path: None,
            left_path,
            right_path,
            left_text: String::new(),
            right_text: String::new(),
            hunks: Vec::new(),
            status,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            left_bytes,
            right_bytes,
            highlight_limited,
            loaded: false,
            left_dirty: false,
            right_dirty: false,
        }
    }

    pub fn should_use_plain_render(&self) -> bool {
        self.is_binary || self.highlight_limited
    }

    pub fn set_original_rel_path(&mut self, original_rel_path: PathBuf) {
        self.original_rel_path = Some(original_rel_path);
        if self.left_path.is_some() && self.right_path.is_some() {
            self.status = EntryStatus::Renamed;
        }
    }

    pub fn is_renamed(&self) -> bool {
        self.original_rel_path.is_some() && self.left_path.is_some() && self.right_path.is_some()
    }

    pub(crate) fn apply_loaded_data(&mut self, data: LoadedDiffData) {
        self.left_text = data.left_text;
        self.right_text = data.right_text;
        self.left_bytes = data.left_bytes;
        self.right_bytes = data.right_bytes;
        self.left_is_binary = data.left_is_binary;
        self.right_is_binary = data.right_is_binary;
        self.is_binary = data.is_binary;
        self.has_unsupported_encoding = data.has_unsupported_encoding;
        self.left_has_unsupported_encoding = data.left_has_unsupported_encoding;
        self.right_has_unsupported_encoding = data.right_has_unsupported_encoding;
        self.left_has_utf8_bom = data.left_has_utf8_bom;
        self.right_has_utf8_bom = data.right_has_utf8_bom;
        self.highlight_limited = data.highlight_limited;
        self.hunks = data.hunks;
        self.status = data.status;
        if self.is_renamed() {
            self.status = EntryStatus::Renamed;
        }
        self.loaded = true;
    }
}
