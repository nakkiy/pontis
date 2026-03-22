use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::App;
use crate::syntax::SyntaxPainter;
use crate::ui;
use crate::ui::DiffViewRenderCache;

use super::input::{self, InputOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LoopExit {
    Quit,
    ReloadRequested,
}

pub(super) fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    painter: &SyntaxPainter,
    render_cache: &mut DiffViewRenderCache,
) -> Result<LoopExit> {
    let mut needs_draw = true;
    loop {
        if app.poll_prefetch() {
            needs_draw = true;
        }
        if app.tick_status() {
            needs_draw = true;
        }
        if needs_draw {
            terminal.draw(|f| ui::render(f, app, painter, render_cache))?;
            needs_draw = false;
        }

        if app.should_quit() {
            return Ok(LoopExit::Quit);
        }

        if !event::poll(Duration::from_millis(16))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => {
                if app.poll_prefetch() {
                    needs_draw = true;
                }
                if app.tick_status() {
                    needs_draw = true;
                }
                match input::handle_key_event(app, key) {
                    InputOutcome::None => {}
                    InputOutcome::Redraw => {
                        needs_draw = true;
                    }
                    InputOutcome::ReloadRequested => {
                        return Ok(LoopExit::ReloadRequested);
                    }
                }
            }
            Event::Resize(_, _) => {
                needs_draw = true;
            }
            _ => {}
        }
    }
}
