use crate::app::App;

impl App {
    pub(crate) fn select_next_file(&mut self) {
        let Some(current_row) = self.current_visible_file_row().or(Some(0)) else {
            return;
        };
        self.select_visible_row(current_row.saturating_add(1));
    }

    pub(crate) fn select_next_file_page(&mut self) {
        let Some(current_row) = self.current_visible_file_row().or(Some(0)) else {
            return;
        };
        self.select_visible_row(current_row.saturating_add(10));
    }

    pub(crate) fn select_prev_file(&mut self) {
        let Some(current_row) = self
            .current_visible_file_row()
            .or_else(|| self.visible_file_indices().len().checked_sub(1))
        else {
            return;
        };
        self.select_visible_row(current_row.saturating_sub(1));
    }

    pub(crate) fn select_prev_file_page(&mut self) {
        let Some(current_row) = self
            .current_visible_file_row()
            .or_else(|| self.visible_file_indices().len().checked_sub(1))
        else {
            return;
        };
        self.select_visible_row(current_row.saturating_sub(10));
    }

    pub(super) fn next_file_with_hunks(&mut self) {
        let Some(current_row) = self.current_visible_file_row().or(Some(0)) else {
            return;
        };
        for &idx in self.visible_file_indices().iter().skip(current_row + 1) {
            if self.files[idx].loaded && self.files[idx].hunks.is_empty() {
                continue;
            }
            if !self.files[idx].loaded || !self.files[idx].hunks.is_empty() {
                self.activate_file(idx, 0);
                return;
            }
        }
    }

    pub(super) fn prev_file_with_hunks(&mut self) {
        let Some(current_row) = self
            .current_visible_file_row()
            .or_else(|| self.visible_file_indices().len().checked_sub(1))
        else {
            return;
        };
        for &idx in self.visible_file_indices()[..current_row].iter().rev() {
            if self.files[idx].loaded && self.files[idx].hunks.is_empty() {
                continue;
            }
            if !self.files[idx].loaded || !self.files[idx].hunks.is_empty() {
                let target_hunk = if self.files[idx].loaded {
                    self.files[idx].hunks.len().saturating_sub(1)
                } else {
                    0
                };
                self.activate_file(idx, target_hunk);
                return;
            }
        }
    }

    fn select_visible_row(&mut self, row: usize) {
        if self.visible_file_indices().is_empty() {
            return;
        }
        let last_row = self.visible_file_indices().len().saturating_sub(1);
        let row = row.min(last_row);
        self.select_file(self.visible_file_indices()[row]);
    }

    pub(crate) fn activate_file(&mut self, idx: usize, hunk_idx: usize) {
        if idx >= self.files.len() {
            return;
        }
        self.current_file = idx;
        self.current_hunk = hunk_idx;
        self.scroll.reset_all();
        self.mark_diff_view_dirty();
        self.sync_current_hunk_focus();
        self.prefetch_around();
        self.update_context_status();
    }
}
