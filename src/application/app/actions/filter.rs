use crate::app::App;

impl App {
    pub(crate) fn toggle_show_added(&mut self) {
        self.file_status_filter.show_added = !self.file_status_filter.show_added;
        self.apply_filter_state_change();
    }

    pub(crate) fn toggle_show_modified(&mut self) {
        self.file_status_filter.show_modified = !self.file_status_filter.show_modified;
        self.apply_filter_state_change();
    }

    pub(crate) fn toggle_show_deleted(&mut self) {
        self.file_status_filter.show_deleted = !self.file_status_filter.show_deleted;
        self.apply_filter_state_change();
    }

    pub(crate) fn toggle_show_renamed(&mut self) {
        self.file_status_filter.show_renamed = !self.file_status_filter.show_renamed;
        self.apply_filter_state_change();
    }

    pub(crate) fn toggle_show_unchanged(&mut self) {
        self.file_status_filter.show_unchanged = !self.file_status_filter.show_unchanged;
        self.apply_filter_state_change();
    }

    pub(crate) fn reset_file_status_filter(&mut self) {
        self.file_status_filter = Default::default();
        self.apply_filter_state_change();
    }

    pub(crate) fn show_added(&self) -> bool {
        self.file_status_filter.show_added
    }

    pub(crate) fn show_modified(&self) -> bool {
        self.file_status_filter.show_modified
    }

    pub(crate) fn show_deleted(&self) -> bool {
        self.file_status_filter.show_deleted
    }

    pub(crate) fn show_renamed(&self) -> bool {
        self.file_status_filter.show_renamed
    }

    pub(crate) fn show_unchanged(&self) -> bool {
        self.file_status_filter.show_unchanged
    }

    pub(crate) fn visible_file_indices(&self) -> &[usize] {
        self.visible_file_indices.as_slice()
    }

    pub(crate) fn current_visible_file_row(&self) -> Option<usize> {
        self.visible_file_indices
            .iter()
            .position(|&idx| idx == self.current_file)
    }

    pub(crate) fn rebuild_visible_file_indices(&mut self) {
        self.visible_file_indices.clear();
        for (idx, file) in self.files.iter().enumerate() {
            if self.file_status_filter.includes(file.status) {
                self.visible_file_indices.push(idx);
            }
        }
    }

    pub(crate) fn sync_visible_files_after_file_update(&mut self) {
        let previous = self.current_file;
        self.rebuild_visible_file_indices();
        self.retarget_current_file_after_filter(previous);
    }

    fn apply_filter_state_change(&mut self) {
        self.sync_visible_files_after_file_update();
    }

    fn retarget_current_file_after_filter(&mut self, previous: usize) {
        if self.visible_file_indices.is_empty() || self.files.is_empty() {
            self.update_context_status();
            return;
        }

        if self.visible_file_indices.contains(&previous) {
            self.update_context_status();
            return;
        }

        let next_visible = self
            .visible_file_indices
            .iter()
            .copied()
            .find(|&idx| idx >= previous)
            .or_else(|| self.visible_file_indices.last().copied());
        let Some(next_visible) = next_visible else {
            return;
        };

        self.current_file = next_visible;
        self.current_hunk = 0;
        self.scroll.reset_all();
        self.mark_diff_view_dirty();
        self.sync_current_hunk_focus();
        self.prefetch_around();
        self.update_context_status();
    }
}
