use std::sync::mpsc::{self, Sender};

use super::tasks::{prefetch_file_data, spawn_status_resolver_jobs};
use crate::app::App;
use crate::model::EntryStatus;

impl App {
    pub(crate) fn spawn_status_resolver(&mut self, tx: Sender<super::super::StatusResolveResult>) {
        let jobs = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, file)| file.status == EntryStatus::Pending && !file.loaded)
            .map(|(idx, file)| (idx, file.left_path.clone(), file.right_path.clone()))
            .collect::<Vec<_>>();
        if jobs.is_empty() {
            return;
        }

        spawn_status_resolver_jobs(jobs, self.settings().clone(), self.loader.clone(), tx);
    }

    pub(crate) fn ensure_current_loaded(&mut self) {
        if self.current_visible_file_row().is_none() {
            return;
        }
        let idx = self.current_file;
        if self.files[idx].loaded {
            return;
        }
        let cfg = self.settings().clone();
        if let Err(err) = self
            .loader
            .load_file_with_config(&mut self.files[idx], &cfg)
        {
            self.set_error_status(&format!(
                "failed to load {}: {err:#}",
                self.files[idx].rel_path.display()
            ));
        } else {
            self.mark_diff_view_dirty();
            self.sync_visible_files_after_file_update();
        }
        self.prefetch_in_flight.remove(&idx);
        self.prefetch_neighbors();
        self.update_context_status();
    }

    pub(crate) fn poll_prefetch(&mut self) -> bool {
        let mut changed = false;

        if self.poll_status_resolve() {
            changed = true;
        }

        while let Ok(msg) = self.prefetch_rx.try_recv() {
            self.prefetch_in_flight.remove(&msg.idx);
            match msg.result {
                Ok(data) => {
                    if let Some(file) = self.files.get_mut(msg.idx)
                        && !file.loaded
                    {
                        file.apply_loaded_data(data);
                        changed = true;
                        if msg.idx == self.current_file {
                            self.sync_current_hunk_focus();
                            self.update_context_status();
                        }
                    }
                }
                Err(err) => {
                    if msg.idx == self.current_file {
                        self.set_error_status(&format!("prefetch failed: {err}"));
                        changed = true;
                    }
                }
            }
        }
        if changed {
            self.sync_visible_files_after_file_update();
        }
        changed
    }

    pub(crate) fn prefetch_neighbors(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.prefetch_index(self.current_file.saturating_sub(1));
        if self.current_file + 1 < self.files.len() {
            self.prefetch_index(self.current_file + 1);
        }
    }

    pub(crate) fn prefetch_around(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.prefetch_index(self.current_file);
        self.prefetch_neighbors();
    }

    fn prefetch_index(&mut self, idx: usize) {
        if idx >= self.files.len()
            || self.files[idx].loaded
            || self.prefetch_in_flight.contains(&idx)
        {
            return;
        }
        self.prefetch_in_flight.insert(idx);
        prefetch_file_data(
            idx,
            self.files[idx].left_path.clone(),
            self.files[idx].right_path.clone(),
            self.settings().clone(),
            self.loader.clone(),
            self.prefetch_tx.clone(),
        );
    }

    fn poll_status_resolve(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.status_resolve_rx.try_recv() {
                Ok(msg) => {
                    if let Some(file) = self.files.get_mut(msg.idx)
                        && !file.loaded
                        && file.status == EntryStatus::Pending
                    {
                        match msg.result {
                            Ok(status) => {
                                file.status = status;
                                changed = true;
                            }
                            Err(err) => {
                                if msg.idx == self.current_file {
                                    self.set_error_status(&format!("status resolve failed: {err}"));
                                    changed = true;
                                }
                            }
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
        changed
    }
}
