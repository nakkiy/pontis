use crate::app::{App, Focus};
use crate::diff::DiffComparePolicies;
use crate::model::{DiffFile, Roots};
use crate::settings::AppSettings;

impl App {
    pub(crate) fn current_file(&self) -> Option<&DiffFile> {
        self.current_visible_file_row()?;
        self.files.get(self.current_file)
    }

    pub(crate) fn current_file_mut(&mut self) -> Option<&mut DiffFile> {
        self.current_visible_file_row()?;
        self.files.get_mut(self.current_file)
    }

    pub(crate) fn files(&self) -> &[DiffFile] {
        self.files.as_slice()
    }

    pub(crate) fn roots(&self) -> &Roots {
        &self.roots
    }

    pub(crate) fn current_file_index(&self) -> usize {
        self.current_file
    }

    pub(crate) fn focus(&self) -> Focus {
        self.focus
    }

    pub(crate) fn current_hunk(&self) -> usize {
        self.current_hunk
    }

    pub(crate) fn scroll_y(&self) -> u16 {
        self.scroll.y()
    }

    pub(crate) fn scroll_x(&self) -> u16 {
        self.scroll.x()
    }

    pub(crate) fn file_list_scroll_y(&self) -> u16 {
        self.file_list_scroll.y()
    }

    pub(crate) fn file_list_scroll_x(&self) -> u16 {
        self.file_list_scroll.x()
    }

    pub(crate) fn focus_diff(&mut self) {
        self.focus = Focus::Diff;
        self.update_context_status();
    }

    pub(crate) fn focus_file_list(&mut self) {
        self.focus = Focus::FileList;
        self.update_context_status();
    }

    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub(crate) fn status_line(&self) -> &str {
        self.status_line.as_str()
    }

    pub(crate) fn allow_left_write(&self) -> bool {
        self.allow_left_write
    }

    pub(crate) fn allow_right_write(&self) -> bool {
        self.allow_right_write
    }

    pub(crate) fn backup_on_save(&self) -> bool {
        self.backup_on_save
    }

    pub(crate) fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub(crate) fn request_quit(&mut self) {
        self.should_quit = true;
    }

    pub(crate) fn update_diff_scroll_limits(&mut self, max_scroll_y: u16, max_scroll_x: u16) {
        self.scroll.update_limits(max_scroll_y, max_scroll_x);
    }

    pub(crate) fn update_file_list_scroll_limits(
        &mut self,
        max_scroll_y: u16,
        max_scroll_x: u16,
        viewport_height: u16,
    ) {
        self.file_list_scroll
            .update_limits(max_scroll_y, max_scroll_x);
        self.ensure_current_file_visible_in_list(viewport_height);
    }

    pub(crate) fn scroll_file_list_left(&mut self, cols: u16) {
        self.file_list_scroll.move_left(cols);
    }

    pub(crate) fn scroll_file_list_right(&mut self, cols: u16) {
        self.file_list_scroll.move_right(cols);
    }

    pub(crate) fn scroll_file_list_left_edge(&mut self) {
        self.file_list_scroll.move_left_edge();
    }

    pub(crate) fn scroll_file_list_right_edge(&mut self) {
        self.file_list_scroll.move_right_edge();
    }

    pub(crate) fn diff_view_epoch(&self) -> u64 {
        self.diff_view_epoch
    }

    pub(crate) fn mark_diff_view_dirty(&mut self) {
        self.diff_view_epoch = self.diff_view_epoch.wrapping_add(1);
    }

    pub(crate) fn refresh_current_file_after_text_change(&mut self) {
        let compare_policies = self.settings.compare_policies;
        let Some(file) = self.current_file_mut() else {
            return;
        };
        file.recompute_hunks_with_policies(DiffComparePolicies::new(
            compare_policies.whitespace_policy,
            compare_policies.line_ending_policy,
        ));
        self.sync_current_hunk_focus();
        self.sync_visible_files_after_file_update();
    }

    fn ensure_current_file_visible_in_list(&mut self, viewport_height: u16) {
        if viewport_height == 0 {
            return;
        }
        let Some(selected_row) = self.current_visible_file_row() else {
            self.file_list_scroll.set_y(0);
            return;
        };
        let selected = selected_row.min(u16::MAX as usize) as u16;
        let top = self.file_list_scroll.y();
        let bottom = top.saturating_add(viewport_height.saturating_sub(1));
        if selected < top {
            self.file_list_scroll.set_y(selected);
            return;
        }
        if selected > bottom {
            self.file_list_scroll
                .set_y(selected.saturating_sub(viewport_height.saturating_sub(1)));
        }
    }
}
