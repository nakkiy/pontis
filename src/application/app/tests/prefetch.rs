use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use super::helpers::{sample_roots, unloaded_diff_file_with_leading_context};
use super::test_loader;
use crate::app::{App, PrefetchResult};
use crate::model::{DiffFile, EntryStatus, LoadedDiffData};
use crate::ports::DiffLoader;
use crate::settings::AppSettings;

#[test]
fn poll_prefetch_loads_current_file_and_focuses_first_hunk() {
    let (file, loaded) = unloaded_diff_file_with_leading_context("late-diff.txt", 12);
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        false,
        true,
    );

    app.prefetch_tx
        .send(PrefetchResult {
            idx: 0,
            result: Ok(loaded),
        })
        .expect("send prefetch result");

    assert!(app.poll_prefetch());
    assert!(app.files[0].loaded);

    app.update_diff_scroll_limits(20, 0);
    app.sync_pending_hunk_focus();

    assert_eq!(app.scroll_y(), 10);
}

#[test]
fn ensure_current_loaded_sets_error_status_when_loader_fails() {
    let (file, _) = unloaded_diff_file_with_leading_context("late-diff.txt", 2);
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        Arc::new(FailingLoader),
        true,
        false,
        true,
    );

    app.ensure_current_loaded();

    assert!(!app.files()[0].loaded);
    assert!(app.status_line().contains("failed to load"));
    assert!(app.status_line().contains("loader exploded"));
}

#[test]
fn poll_prefetch_sets_error_status_for_current_file_failure() {
    let (file, _) = unloaded_diff_file_with_leading_context("late-diff.txt", 2);
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        false,
        true,
    );

    app.prefetch_tx
        .send(PrefetchResult {
            idx: 0,
            result: Err("prefetch boom".to_string()),
        })
        .expect("send prefetch failure");

    assert!(app.poll_prefetch());
    assert_eq!(app.status_line(), "prefetch failed: prefetch boom");
}

#[test]
fn app_new_resolves_pending_status_in_background() {
    let file = DiffFile::new_unloaded(
        PathBuf::from("pending.txt"),
        Some(PathBuf::from("/tmp/l/pending.txt")),
        Some(PathBuf::from("/tmp/r/pending.txt")),
        5,
        5,
        false,
        EntryStatus::Pending,
    );
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        Arc::new(StatusOnlyLoader),
        true,
        false,
        true,
    );

    let updated = wait_for_prefetch_poll(&mut app);

    assert!(updated, "expected status resolver to send an update");
    assert_eq!(app.files()[0].status, EntryStatus::Unchanged);
}

#[derive(Debug)]
struct FailingLoader;

impl DiffLoader for FailingLoader {
    fn load_file_with_config(&self, _file: &mut DiffFile, _cfg: &AppSettings) -> Result<()> {
        anyhow::bail!("loader exploded");
    }

    fn load_data(
        &self,
        _left_path: Option<PathBuf>,
        _right_path: Option<PathBuf>,
        _cfg: &AppSettings,
    ) -> Result<LoadedDiffData> {
        anyhow::bail!("background prefetch exploded");
    }

    fn resolve_status(
        &self,
        _left_path: Option<PathBuf>,
        _right_path: Option<PathBuf>,
        _cfg: &AppSettings,
    ) -> Result<EntryStatus> {
        anyhow::bail!("status resolution exploded");
    }
}

#[derive(Debug)]
struct StatusOnlyLoader;

impl DiffLoader for StatusOnlyLoader {
    fn load_file_with_config(&self, _file: &mut DiffFile, _cfg: &AppSettings) -> Result<()> {
        anyhow::bail!("unused load_file_with_config");
    }

    fn load_data(
        &self,
        _left_path: Option<PathBuf>,
        _right_path: Option<PathBuf>,
        _cfg: &AppSettings,
    ) -> Result<LoadedDiffData> {
        anyhow::bail!("unused load_data");
    }

    fn resolve_status(
        &self,
        _left_path: Option<PathBuf>,
        _right_path: Option<PathBuf>,
        _cfg: &AppSettings,
    ) -> Result<EntryStatus> {
        Ok(EntryStatus::Unchanged)
    }
}

fn wait_for_prefetch_poll(app: &mut App) -> bool {
    for _ in 0..20 {
        if app.poll_prefetch() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    false
}
