use std::path::Path;

use anyhow::Result;

use super::save_support::{FileSide, save_file_side_if_dirty};
use crate::app::App;
use crate::model::Mode;

impl App {
    pub(crate) fn save_current(&mut self) -> Result<()> {
        if self.files.is_empty() || self.current_file().is_none() {
            self.set_temporary_status("no visible file selected");
            return Ok(());
        }
        let roots_mode = self.roots.mode;
        let roots_left = self.roots.left.clone();
        let roots_right = self.roots.right.clone();
        let idx = self.current_file;
        let saved_count = self.save_file_sides(idx, roots_mode, &roots_left, &roots_right)?;

        if saved_count == 0 {
            self.set_temporary_status("no writable pending changes for current file");
        } else {
            self.set_temporary_status("saved current file");
        }
        Ok(())
    }

    pub(crate) fn save_all(&mut self) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }

        let roots_mode = self.roots.mode;
        let roots_left = self.roots.left.clone();
        let roots_right = self.roots.right.clone();
        let mut saved_count = 0usize;

        for idx in 0..self.files.len() {
            saved_count += self.save_file_sides(idx, roots_mode, &roots_left, &roots_right)?;
        }

        if saved_count == 0 {
            self.set_temporary_status("no pending changes to save");
        } else {
            self.set_temporary_status(&format!("saved {saved_count} side(s)"));
        }
        Ok(())
    }

    fn save_file_sides(
        &mut self,
        idx: usize,
        roots_mode: Mode,
        roots_left: &Path,
        roots_right: &Path,
    ) -> Result<usize> {
        let mut saved_count = 0;
        saved_count +=
            self.save_file_side_if_dirty(idx, FileSide::Left, roots_mode, roots_left, roots_right)?;
        saved_count += self.save_file_side_if_dirty(
            idx,
            FileSide::Right,
            roots_mode,
            roots_left,
            roots_right,
        )?;
        Ok(saved_count)
    }

    fn save_file_side_if_dirty(
        &mut self,
        idx: usize,
        side: FileSide,
        roots_mode: Mode,
        roots_left: &Path,
        roots_right: &Path,
    ) -> Result<usize> {
        let can_write = match side {
            FileSide::Left => self.allow_left_write(),
            FileSide::Right => self.allow_right_write(),
        };
        let create_backup = self.create_backup();
        save_file_side_if_dirty(
            &mut self.files[idx],
            side,
            can_write,
            roots_mode,
            roots_left,
            roots_right,
            create_backup,
        )
    }
}
