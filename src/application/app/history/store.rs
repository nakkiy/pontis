use crate::app::view_state::ScrollState;
use crate::model::{EntryStatus, Hunk};

#[derive(Debug, Clone, Default)]
pub(crate) struct MergeHistory {
    undo_stack: Vec<MergeSnapshot>,
    redo_stack: Vec<MergeSnapshot>,
}

impl MergeHistory {
    pub(super) fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub(super) fn clear_redo(&mut self) {
        self.redo_stack.clear();
    }

    pub(super) fn pop_undo(&mut self) -> Option<MergeSnapshot> {
        self.undo_stack.pop()
    }

    pub(super) fn pop_redo(&mut self) -> Option<MergeSnapshot> {
        self.redo_stack.pop()
    }

    pub(super) fn push_undo(&mut self, snapshot: MergeSnapshot) {
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > 200 {
            let _ = self.undo_stack.remove(0);
        }
    }

    pub(super) fn push_redo(&mut self, snapshot: MergeSnapshot) {
        self.redo_stack.push(snapshot);
    }
}

#[derive(Debug, Clone)]
pub(super) struct MergeSnapshot {
    pub(super) file_idx: usize,
    pub(super) left_text: String,
    pub(super) right_text: String,
    pub(super) left_dirty: bool,
    pub(super) right_dirty: bool,
    pub(super) hunks: Vec<Hunk>,
    pub(super) status: EntryStatus,
    pub(super) current_hunk: usize,
    pub(super) scroll: ScrollState,
}
