pub(super) mod store;

use self::store::MergeSnapshot;
use super::App;

impl App {
    pub(super) fn clear_merge_history(&mut self) {
        self.history.clear();
    }

    pub(super) fn clear_redo_history(&mut self) {
        self.history.clear_redo();
    }

    pub(crate) fn undo_merge(&mut self) {
        self.ensure_current_loaded();
        let Some(snapshot) = self.history.pop_undo() else {
            self.set_temporary_status("undo stack is empty");
            return;
        };
        if snapshot.file_idx >= self.files.len() {
            self.set_temporary_status("undo target is not available");
            return;
        }

        if let Some(current) = self.capture_snapshot(snapshot.file_idx) {
            self.history.push_redo(current);
        }
        self.apply_snapshot(snapshot);
        self.set_temporary_status("undid last merge");
    }

    pub(crate) fn redo_merge(&mut self) {
        self.ensure_current_loaded();
        let Some(snapshot) = self.history.pop_redo() else {
            self.set_temporary_status("redo stack is empty");
            return;
        };
        if snapshot.file_idx >= self.files.len() {
            self.set_temporary_status("redo target is not available");
            return;
        }

        if let Some(current) = self.capture_snapshot(snapshot.file_idx) {
            self.history.push_undo(current);
        }
        self.apply_snapshot(snapshot);
        self.set_temporary_status("redid merge");
    }

    fn capture_snapshot(&self, file_idx: usize) -> Option<MergeSnapshot> {
        let file = self.files.get(file_idx)?;
        Some(MergeSnapshot {
            file_idx,
            left_text: file.left_text.clone(),
            right_text: file.right_text.clone(),
            left_dirty: file.left_dirty,
            right_dirty: file.right_dirty,
            hunks: file.hunks.clone(),
            status: file.status,
            current_hunk: self.current_hunk,
            scroll: self.scroll,
        })
    }

    pub(super) fn push_undo_snapshot(&mut self, file_idx: usize) {
        if let Some(snapshot) = self.capture_snapshot(file_idx) {
            self.history.push_undo(snapshot);
        }
    }

    fn apply_snapshot(&mut self, snapshot: MergeSnapshot) {
        let idx = snapshot.file_idx;
        self.current_file = idx;
        if let Some(file) = self.files.get_mut(idx) {
            file.left_text = snapshot.left_text;
            file.right_text = snapshot.right_text;
            file.left_dirty = snapshot.left_dirty;
            file.right_dirty = snapshot.right_dirty;
            file.hunks = snapshot.hunks;
            file.status = snapshot.status;
            file.loaded = true;
            if file.hunks.is_empty() {
                self.current_hunk = 0;
            } else {
                self.current_hunk = snapshot.current_hunk.min(file.hunks.len() - 1);
            }
            self.scroll = snapshot.scroll;
        }
        self.mark_diff_view_dirty();
        self.sync_visible_files_after_file_update();
        self.prefetch_around();
    }
}
