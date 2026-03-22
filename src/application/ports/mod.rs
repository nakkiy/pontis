use std::path::PathBuf;

use anyhow::Result;

use crate::model::{DiffFile, LoadedDiffData, Roots};
use crate::settings::AppSettings;

pub(crate) type ReloadedTargets = (Vec<DiffFile>, Roots, bool, bool);

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

pub(crate) trait ComparisonReloader: Send + Sync {
    fn reload_targets(&self, cfg: &AppSettings) -> Result<ReloadedTargets>;
}
