use std::path::PathBuf;

use anyhow::Result;

use crate::model::{DiffFile, LoadedDiffData};
use crate::settings::AppSettings;

pub(crate) trait DiffLoader: Send + Sync {
    fn load_file_with_config(&self, file: &mut DiffFile, cfg: &AppSettings) -> Result<()>;
    fn load_data(
        &self,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        cfg: &AppSettings,
    ) -> Result<LoadedDiffData>;
    fn resolve_status(
        &self,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        cfg: &AppSettings,
    ) -> Result<crate::model::EntryStatus>;
}
