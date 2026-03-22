use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc;

use crate::app::history::store::MergeHistory;
use crate::app::{App, Focus};
use crate::model::{DiffFile, Roots};
use crate::ports::DiffLoader;
use crate::settings::AppSettings;

impl App {
    pub(crate) fn new(
        files: Vec<DiffFile>,
        roots: Roots,
        settings: AppSettings,
        loader: Arc<dyn DiffLoader>,
        allow_left_write: bool,
        reload_supported: bool,
        allow_right_write: bool,
    ) -> Self {
        let (prefetch_tx, prefetch_rx) = mpsc::channel();
        let (status_resolve_tx, status_resolve_rx) = mpsc::channel();
        let mut app = Self {
            files,
            visible_file_indices: Vec::new(),
            roots,
            current_file: 0,
            current_hunk: 0,
            needs_hunk_focus_sync: false,
            focus: Focus::FileList,
            help_open: false,
            should_quit: false,
            scroll: Default::default(),
            file_list_scroll: Default::default(),
            allow_left_write,
            reload_supported,
            allow_right_write,
            settings,
            loader,
            prefetch_tx,
            prefetch_rx,
            prefetch_in_flight: HashSet::new(),
            status_resolve_rx,
            history: MergeHistory::default(),
            status_line: Self::FILE_LIST_HINT.to_string(),
            status_until: None,
            diff_view_epoch: 0,
            file_status_filter: Default::default(),
        };
        app.rebuild_visible_file_indices();
        app.sync_current_hunk_focus();
        app.spawn_status_resolver(status_resolve_tx);
        app.prefetch_around();
        app.update_context_status();
        app
    }

    pub(crate) fn select_file(&mut self, idx: usize) {
        self.activate_file(idx, 0);
    }
}
