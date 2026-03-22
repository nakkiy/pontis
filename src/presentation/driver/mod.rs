use std::io;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use anyhow::{Result, anyhow};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::App;
use crate::ports::{ComparisonReloader, DiffLoader};
use crate::settings::AppSettings;
use crate::syntax::SyntaxPainter;
use crate::ui;
use crate::ui::DiffViewRenderCache;

mod input;
mod loop_core;

use loop_core::LoopExit;

type LoadedTargets = (Vec<crate::model::DiffFile>, crate::model::Roots, bool, bool);
type TargetLoadResult = Result<LoadedTargets, String>;

pub(crate) fn run_tui_with_loading(
    settings: AppSettings,
    painter: SyntaxPainter,
    loader: Arc<dyn DiffLoader>,
    reloader: Option<Arc<dyn ComparisonReloader>>,
    target_rx: Receiver<TargetLoadResult>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut render_cache = DiffViewRenderCache::default();

    let run_result = run_loading_loop(
        &mut terminal,
        settings,
        painter,
        loader,
        reloader,
        target_rx,
        &mut render_cache,
    );

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    run_result
}

fn run_loading_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    settings: AppSettings,
    painter: SyntaxPainter,
    loader: Arc<dyn DiffLoader>,
    reloader: Option<Arc<dyn ComparisonReloader>>,
    target_rx: Receiver<TargetLoadResult>,
    render_cache: &mut DiffViewRenderCache,
) -> Result<()> {
    loop {
        match target_rx.try_recv() {
            Ok(Ok((files, roots, allow_left_write, allow_right_write))) => {
                let mut app = App::new(
                    files,
                    roots,
                    settings.clone(),
                    loader.clone(),
                    allow_left_write,
                    reloader.is_some(),
                    allow_right_write,
                );
                loop {
                    match loop_core::run_event_loop(terminal, &mut app, &painter, render_cache)? {
                        LoopExit::Quit => return Ok(()),
                        LoopExit::ReloadRequested => {
                            let Some(reloader) = &reloader else {
                                return Err(anyhow!("reload requested without reloader"));
                            };
                            try_reload_app(&mut app, &settings, loader.clone(), reloader.as_ref());
                        }
                    }
                }
            }
            Ok(Err(err)) => return Err(anyhow!(err)),
            Err(TryRecvError::Disconnected) => {
                return Err(anyhow!("background target build disconnected"));
            }
            Err(TryRecvError::Empty) => {}
        }

        terminal.draw(|f| ui::render_loading(f, "loading comparison targets..."))?;

        if !event::poll(Duration::from_millis(16))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.code == KeyCode::Char('q') => return Ok(()),
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}

fn try_reload_app(
    app: &mut App,
    settings: &AppSettings,
    loader: Arc<dyn DiffLoader>,
    reloader: &dyn ComparisonReloader,
) {
    match reloader.reload_targets(settings) {
        Ok((files, roots, allow_left_write, allow_right_write)) => {
            *app = App::new(
                files,
                roots,
                settings.clone(),
                loader,
                allow_left_write,
                true,
                allow_right_write,
            );
            app.set_temporary_status("reloaded comparison targets");
        }
        Err(err) => {
            app.set_error_status(&format!("reload failed: {err:#}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use anyhow::Result;

    use super::try_reload_app;
    use crate::app::App;
    use crate::model::{DiffContent, DiffFile, EntryStatus, LoadedDiffData, Mode, Roots};
    use crate::ports::{ComparisonReloader, DiffLoader, ReloadedTargets};
    use crate::settings::AppSettings;

    #[test]
    fn reload_failure_keeps_existing_app_and_sets_error_status() {
        let original = sample_file("current.txt");
        let roots = sample_roots();
        let mut app = App::new(
            vec![original],
            roots,
            AppSettings::default(),
            Arc::new(NoopLoader),
            true,
            true,
            true,
        );

        try_reload_app(
            &mut app,
            &AppSettings::default(),
            Arc::new(NoopLoader),
            &FailingReloader,
        );

        assert_eq!(app.files()[0].rel_path, PathBuf::from("current.txt"));
        assert!(app.status_line().contains("reload failed:"));
        assert!(app.status_line().contains("reload exploded"));
    }

    #[derive(Debug)]
    struct NoopLoader;

    impl DiffLoader for NoopLoader {
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
            anyhow::bail!("unused resolve_status");
        }
    }

    #[derive(Debug)]
    struct FailingReloader;

    impl ComparisonReloader for FailingReloader {
        fn reload_targets(&self, _cfg: &AppSettings) -> Result<ReloadedTargets> {
            anyhow::bail!("reload exploded");
        }
    }

    fn sample_roots() -> Roots {
        Roots {
            left: PathBuf::from("/tmp/l"),
            right: PathBuf::from("/tmp/r"),
            mode: Mode::Directory,
            left_label: None,
            right_label: None,
        }
    }

    fn sample_file(name: &str) -> DiffFile {
        DiffFile::new(
            PathBuf::from(name),
            Some(PathBuf::from(format!("/tmp/l/{name}"))),
            Some(PathBuf::from(format!("/tmp/r/{name}"))),
            DiffContent {
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
            },
            EntryStatus::Modified,
        )
    }
}
