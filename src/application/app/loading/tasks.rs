use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;

use crate::app::{PrefetchResult, StatusResolveResult};
use crate::ports::DiffLoader;
use crate::settings::AppSettings;

pub(super) fn spawn_status_resolver_jobs(
    jobs: Vec<(usize, Option<PathBuf>, Option<PathBuf>)>,
    settings: AppSettings,
    loader: Arc<dyn DiffLoader>,
    tx: Sender<StatusResolveResult>,
) {
    thread::spawn(move || {
        for (idx, left_path, right_path) in jobs {
            let result = loader
                .resolve_status(left_path, right_path, &settings)
                .map_err(|e| format!("{e:#}"));
            if tx.send(StatusResolveResult { idx, result }).is_err() {
                break;
            }
        }
    });
}

pub(super) fn prefetch_file_data(
    idx: usize,
    left_path: Option<PathBuf>,
    right_path: Option<PathBuf>,
    settings: AppSettings,
    loader: Arc<dyn DiffLoader>,
    tx: Sender<PrefetchResult>,
) {
    thread::spawn(move || {
        let result = loader
            .load_data(left_path, right_path, &settings)
            .map_err(|e| format!("{e:#}"));
        let _ = tx.send(PrefetchResult { idx, result });
    });
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::model::{EntryStatus, Hunk, LoadedDiffData};

    use super::*;

    #[derive(Debug)]
    struct StubLoader {
        load_data_result: Mutex<Option<anyhow::Result<LoadedDiffData>>>,
        resolve_status_results: Mutex<VecDeque<anyhow::Result<EntryStatus>>>,
    }

    impl StubLoader {
        fn with_load_data(result: anyhow::Result<LoadedDiffData>) -> Self {
            Self {
                load_data_result: Mutex::new(Some(result)),
                resolve_status_results: Mutex::new(VecDeque::new()),
            }
        }

        fn with_resolve_status(results: Vec<anyhow::Result<EntryStatus>>) -> Self {
            Self {
                load_data_result: Mutex::new(None),
                resolve_status_results: Mutex::new(results.into()),
            }
        }
    }

    impl DiffLoader for StubLoader {
        fn load_file_with_config(
            &self,
            _file: &mut crate::model::DiffFile,
            _cfg: &AppSettings,
        ) -> anyhow::Result<()> {
            anyhow::bail!("unused in loading task tests")
        }

        fn load_data(
            &self,
            _left_path: Option<PathBuf>,
            _right_path: Option<PathBuf>,
            _cfg: &AppSettings,
        ) -> anyhow::Result<LoadedDiffData> {
            self.load_data_result
                .lock()
                .expect("lock load_data_result")
                .take()
                .expect("configured load_data result")
        }

        fn resolve_status(
            &self,
            _left_path: Option<PathBuf>,
            _right_path: Option<PathBuf>,
            _cfg: &AppSettings,
        ) -> anyhow::Result<EntryStatus> {
            self.resolve_status_results
                .lock()
                .expect("lock resolve_status_results")
                .pop_front()
                .expect("configured resolve_status result")
        }
    }

    #[test]
    fn spawn_status_resolver_jobs_sends_successes_and_errors_in_order() {
        let (tx, rx) = channel();
        let loader: Arc<dyn DiffLoader> = Arc::new(StubLoader::with_resolve_status(vec![
            Ok(EntryStatus::Unchanged),
            Err(anyhow::anyhow!("resolve failed")),
        ]));

        spawn_status_resolver_jobs(
            vec![(2, None, None), (5, None, None)],
            AppSettings::default(),
            loader,
            tx,
        );

        let first = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("first result");
        assert_eq!(first.idx, 2);
        assert_eq!(first.result.expect("first status"), EntryStatus::Unchanged);

        let second = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("second result");
        assert_eq!(second.idx, 5);
        assert!(
            second
                .result
                .expect_err("second error")
                .contains("resolve failed")
        );
    }

    #[test]
    fn prefetch_file_data_sends_loaded_data_and_maps_errors() {
        let (tx, rx) = channel();
        let loader: Arc<dyn DiffLoader> = Arc::new(StubLoader::with_load_data(Ok(sample_loaded())));

        prefetch_file_data(3, None, None, AppSettings::default(), loader, tx);

        let success = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("success result");
        assert_eq!(success.idx, 3);
        let data = success.result.expect("loaded data");
        assert_eq!(data.left_text, "left\n");
        assert_eq!(data.right_text, "right\n");

        let (tx, rx) = channel();
        let loader: Arc<dyn DiffLoader> = Arc::new(StubLoader::with_load_data(Err(
            anyhow::anyhow!("load failed"),
        )));

        prefetch_file_data(4, None, None, AppSettings::default(), loader, tx);

        let failure = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("failure result");
        assert_eq!(failure.idx, 4);
        assert!(
            failure
                .result
                .expect_err("load error")
                .contains("load failed")
        );
    }

    fn sample_loaded() -> LoadedDiffData {
        LoadedDiffData {
            left_text: "left\n".to_string(),
            right_text: "right\n".to_string(),
            left_bytes: 5,
            right_bytes: 6,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
            hunks: vec![Hunk {
                old_start: 0,
                old_end: 1,
                new_start: 0,
                new_end: 1,
                old_lines: vec!["left\n".to_string()],
                new_lines: vec!["right\n".to_string()],
            }],
            status: EntryStatus::Modified,
        }
    }
}
