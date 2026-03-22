use crate::app::App;
use crate::diff::hunk_display_row;

impl App {
    pub(crate) fn next_hunk_or_file(&mut self) {
        self.ensure_current_loaded();
        let Some(file) = self.current_file() else {
            return;
        };

        if file.hunks.is_empty() {
            self.next_file_with_hunks();
            return;
        }

        if self.current_hunk + 1 >= file.hunks.len() {
            self.next_file_with_hunks();
        } else {
            self.current_hunk += 1;
            self.focus_current_hunk();
        }
    }

    pub(crate) fn prev_hunk_or_file(&mut self) {
        self.ensure_current_loaded();
        let Some(file) = self.current_file() else {
            return;
        };

        if file.hunks.is_empty() {
            self.prev_file_with_hunks();
            return;
        }

        if self.current_hunk == 0 {
            self.prev_file_with_hunks();
        } else {
            self.current_hunk -= 1;
            self.focus_current_hunk();
        }
    }

    pub(crate) fn focus_current_hunk(&mut self) {
        if self.current_file().is_none() {
            return;
        }

        self.needs_hunk_focus_sync = true;
        self.apply_current_hunk_focus();
        self.mark_diff_view_dirty();
    }

    pub(crate) fn sync_current_hunk_focus(&mut self) {
        let Some(file) = self.current_file() else {
            return;
        };
        if !file.loaded {
            return;
        }

        if file.hunks.is_empty() {
            self.current_hunk = 0;
        } else {
            self.current_hunk = self.current_hunk.min(file.hunks.len() - 1);
        }
        self.focus_current_hunk();
    }

    pub(crate) fn sync_pending_hunk_focus(&mut self) {
        if !self.needs_hunk_focus_sync {
            return;
        }

        self.needs_hunk_focus_sync = false;
        if self.current_file().is_none() {
            return;
        }

        self.apply_current_hunk_focus();
    }

    pub(crate) fn scroll_up(&mut self, lines: u16) {
        self.scroll.move_up(lines);
    }

    pub(crate) fn scroll_down(&mut self, lines: u16) {
        self.scroll.move_down(lines);
    }

    pub(crate) fn scroll_left(&mut self, cols: u16) {
        self.scroll.move_left(cols);
    }

    pub(crate) fn scroll_right(&mut self, cols: u16) {
        self.scroll.move_right(cols);
    }

    pub(crate) fn scroll_left_edge(&mut self) {
        self.scroll.move_left_edge();
    }

    pub(crate) fn scroll_right_edge(&mut self) {
        self.scroll.move_right_edge();
    }

    fn apply_current_hunk_focus(&mut self) {
        let anchor = self.current_hunk_scroll_anchor();
        if let Some(anchor) = anchor {
            self.scroll.set_y(anchor.saturating_sub(3));
        } else {
            self.scroll.set_y(0);
        }
    }

    fn current_hunk_scroll_anchor(&self) -> Option<u16> {
        self.current_file()
            .and_then(|file| {
                hunk_display_row(
                    &file.left_text,
                    &file.right_text,
                    self.current_hunk,
                    self.settings.whitespace(),
                    self.settings.line_endings(),
                )
            })
            .map(|row| row as u16)
    }
}
