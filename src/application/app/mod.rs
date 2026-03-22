use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use anyhow::Result;

use history::store::MergeHistory;

use crate::model::{DiffFile, EntryStatus, LoadedDiffData, Roots};
use crate::ports::DiffLoader;
use crate::settings::AppSettings;

mod access;
mod actions;
mod history;
mod init;
mod loading;
mod navigation;
#[cfg(test)]
mod tests;
mod view_state;

pub(crate) use actions::ReloadDecision;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    FileList,
    Diff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeDirection {
    LeftToRight,
    RightToLeft,
}

pub(crate) struct App {
    files: Vec<DiffFile>,
    visible_file_indices: Vec<usize>,
    roots: Roots,
    current_file: usize,
    current_hunk: usize,
    needs_hunk_focus_sync: bool,
    focus: Focus,
    help_open: bool,
    should_quit: bool,
    scroll: view_state::ScrollState,
    file_list_scroll: view_state::ScrollState,
    status_line: String,
    allow_left_write: bool,
    reload_supported: bool,
    allow_right_write: bool,
    settings: AppSettings,
    loader: Arc<dyn DiffLoader>,
    prefetch_tx: Sender<PrefetchResult>,
    prefetch_rx: Receiver<PrefetchResult>,
    prefetch_in_flight: HashSet<usize>,
    status_resolve_rx: Receiver<StatusResolveResult>,
    history: MergeHistory,
    status_until: Option<Instant>,
    diff_view_epoch: u64,
    file_status_filter: FileStatusFilter,
}

#[derive(Debug)]
pub(crate) struct PrefetchResult {
    idx: usize,
    result: Result<LoadedDiffData, String>,
}

#[derive(Debug)]
pub(crate) struct StatusResolveResult {
    idx: usize,
    result: Result<EntryStatus, String>,
}

#[derive(Debug, Clone, Copy)]
struct FileStatusFilter {
    show_added: bool,
    show_modified: bool,
    show_deleted: bool,
    show_renamed: bool,
    show_unchanged: bool,
}

impl Default for FileStatusFilter {
    fn default() -> Self {
        Self {
            show_added: true,
            show_modified: true,
            show_deleted: true,
            show_renamed: true,
            show_unchanged: true,
        }
    }
}

impl FileStatusFilter {
    fn includes(self, status: EntryStatus) -> bool {
        match status {
            EntryStatus::Pending => self.show_modified,
            EntryStatus::Added => self.show_added,
            EntryStatus::Modified => self.show_modified,
            EntryStatus::Deleted => self.show_deleted,
            EntryStatus::Renamed => self.show_renamed,
            EntryStatus::Unchanged => self.show_unchanged,
        }
    }
}

impl App {
    pub(super) const FILE_LIST_HINT: &'static str = "enter: diff | ↑/↓: move | A/M/D/R/=: filter | alt+↑/↓: change | l: reload | ?: help | q: quit";
    pub(super) const DIFF_HINT: &'static str = "esc: files | ↑/↓: scroll | alt+↑/↓: change | alt+←/→: merge | s: save | l: reload | ?: help | q: quit";
}
