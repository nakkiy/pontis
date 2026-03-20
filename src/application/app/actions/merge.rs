use crate::app::{App, MergeDirection};
use crate::diff::{join_lines, split_lines_keep_newline};

impl App {
    pub(crate) fn merge_current_hunk(&mut self, direction: MergeDirection) {
        match direction {
            MergeDirection::LeftToRight if !self.allow_right_write() => {
                self.set_temporary_status("target side is read-only: cannot merge left -> right");
                return;
            }
            MergeDirection::RightToLeft if !self.allow_left_write() => {
                self.set_temporary_status("target side is read-only: cannot merge right -> left");
                return;
            }
            _ => {}
        }
        if self.files.is_empty() || self.current_file().is_none() {
            self.set_temporary_status("no visible file selected");
            return;
        }
        self.ensure_current_loaded();
        if self.files.is_empty() || self.current_file().is_none() {
            self.set_temporary_status("no visible file selected");
            return;
        }
        let file_idx = self.current_file;
        let mut hunk_idx = self.current_hunk;

        let file_hunk_len = self.files[file_idx].hunks.len();
        if self.files[file_idx].is_binary {
            self.set_temporary_status("binary file merge is not supported");
            return;
        }
        if file_hunk_len == 0 {
            self.set_temporary_status("no hunk to merge");
            return;
        }
        if hunk_idx >= file_hunk_len {
            hunk_idx = file_hunk_len.saturating_sub(1);
            self.current_hunk = hunk_idx;
        }

        let hunk = match self.files[file_idx].hunks.get(hunk_idx).cloned() {
            Some(h) => h,
            None => return,
        };

        self.push_undo_snapshot(file_idx);
        self.clear_redo_history();

        let file = &mut self.files[file_idx];
        match direction {
            MergeDirection::LeftToRight => {
                let mut right_lines = split_lines_keep_newline(&file.right_text);
                right_lines.splice(hunk.new_start..hunk.new_end, hunk.old_lines.clone());
                file.right_text = join_lines(&right_lines);
                file.right_dirty = true;
            }
            MergeDirection::RightToLeft => {
                let mut left_lines = split_lines_keep_newline(&file.left_text);
                left_lines.splice(hunk.old_start..hunk.old_end, hunk.new_lines.clone());
                file.left_text = join_lines(&left_lines);
                file.left_dirty = true;
            }
        }
        self.refresh_current_file_after_text_change();
        self.set_temporary_status("merged current hunk");
    }
}
