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
use crate::ports::DiffLoader;
use crate::settings::AppSettings;
use crate::syntax::SyntaxPainter;
use crate::ui;
use crate::ui::DiffViewRenderCache;

mod input;
mod loop_core;

type LoadedTargets = (Vec<crate::model::DiffFile>, crate::model::Roots, bool, bool);
type TargetLoadResult = Result<LoadedTargets, String>;

pub(crate) fn run_tui_with_loading(
    settings: AppSettings,
    painter: SyntaxPainter,
    loader: Arc<dyn DiffLoader>,
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
    target_rx: Receiver<TargetLoadResult>,
    render_cache: &mut DiffViewRenderCache,
) -> Result<()> {
    loop {
        match target_rx.try_recv() {
            Ok(Ok((files, roots, allow_left_write, allow_right_write))) => {
                let backup_on_save = settings.backup_on_save;
                let mut app = App::new(
                    files,
                    roots,
                    backup_on_save,
                    settings,
                    loader,
                    allow_left_write,
                    allow_right_write,
                );
                return loop_core::run_event_loop(terminal, &mut app, &painter, render_cache);
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
