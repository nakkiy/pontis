mod edit;
mod helpers;
mod merge_history;
mod navigation;
mod prefetch;
mod save;

use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn test_loader() -> Arc<dyn crate::ports::DiffLoader> {
    Arc::new(TestDiffLoader)
}

#[derive(Debug, Default)]
struct TestDiffLoader;

impl crate::ports::DiffLoader for TestDiffLoader {
    fn load_file_with_config(
        &self,
        _file: &mut crate::model::DiffFile,
        _cfg: &crate::settings::AppSettings,
    ) -> anyhow::Result<()> {
        anyhow::bail!("test loader does not load files")
    }

    fn load_data(
        &self,
        _left_path: Option<PathBuf>,
        _right_path: Option<PathBuf>,
        _cfg: &crate::settings::AppSettings,
    ) -> anyhow::Result<crate::model::LoadedDiffData> {
        anyhow::bail!("test loader does not load data")
    }

    fn resolve_status(
        &self,
        _left_path: Option<PathBuf>,
        _right_path: Option<PathBuf>,
        _cfg: &crate::settings::AppSettings,
    ) -> anyhow::Result<crate::model::EntryStatus> {
        anyhow::bail!("test loader does not resolve status")
    }
}
