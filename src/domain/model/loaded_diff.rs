use super::{EntryStatus, Hunk};

#[derive(Debug, Clone)]
pub(crate) struct LoadedDiffData {
    pub(crate) left_text: String,
    pub(crate) right_text: String,
    pub(crate) left_bytes: usize,
    pub(crate) right_bytes: usize,
    pub(crate) left_is_binary: bool,
    pub(crate) right_is_binary: bool,
    pub(crate) is_binary: bool,
    pub(crate) has_unsupported_encoding: bool,
    pub(crate) left_has_unsupported_encoding: bool,
    pub(crate) right_has_unsupported_encoding: bool,
    pub(crate) left_has_utf8_bom: bool,
    pub(crate) right_has_utf8_bom: bool,
    pub(crate) highlight_limited: bool,
    pub(crate) hunks: Vec<Hunk>,
    pub(crate) status: EntryStatus,
}
