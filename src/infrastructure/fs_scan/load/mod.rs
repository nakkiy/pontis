use std::path::PathBuf;

use anyhow::Result;

use crate::model::{DiffFile, EntryStatus, LoadedDiffData};
use crate::ports::DiffLoader;
use crate::settings::AppSettings;

mod load_data;
mod status_resolve;

pub(crate) use load_data::{load_diff_data, load_diff_file_with_config};
pub(crate) use status_resolve::resolve_status_only;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FsDiffLoader;

impl DiffLoader for FsDiffLoader {
    fn load_file_with_config(&self, file: &mut DiffFile, cfg: &AppSettings) -> Result<()> {
        load_diff_file_with_config(file, cfg)
    }

    fn load_data(
        &self,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        cfg: &AppSettings,
    ) -> Result<LoadedDiffData> {
        load_diff_data(left_path, right_path, cfg)
    }

    fn resolve_status(
        &self,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        cfg: &AppSettings,
    ) -> Result<EntryStatus> {
        resolve_status_only(left_path, right_path, cfg)
    }
}
