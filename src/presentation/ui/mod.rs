use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::syntax::SyntaxPainter;

mod diff_view;
mod file_list;
mod scroll;
mod style;

pub(crate) use diff_view::DiffViewRenderCache;

pub(crate) fn render(
    frame: &mut Frame<'_>,
    app: &mut App,
    painter: &SyntaxPainter,
    cache: &mut DiffViewRenderCache,
) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(root[1]);

    let left_title = app
        .roots()
        .left_label
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| app.roots().left.display().to_string());
    let right_title = app
        .roots()
        .right_label
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| app.roots().right.display().to_string());
    let title = format!("pontis : {} ↔ {}", left_title, right_title);
    let header = Paragraph::new(title).style(Style::default().fg(style::HEADER_FG));
    frame.render_widget(header, root[0]);

    file_list::render_file_list(frame, app, main[0]);
    diff_view::render_diff_view(frame, app, painter, cache, main[1]);

    let status = Paragraph::new(app.status_line()).style(Style::default().fg(style::STATUS_FG));
    frame.render_widget(status, root[2]);
}

pub(crate) fn render_loading(frame: &mut Frame<'_>, status: &str) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(root[1]);
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[1]);

    let header = Paragraph::new("pontis : loading...").style(Style::default().fg(style::HEADER_FG));
    frame.render_widget(header, root[0]);

    let loading =
        Paragraph::new("loading...").block(Block::default().borders(Borders::ALL).title("Files"));
    frame.render_widget(loading, main[0]);

    let left =
        Paragraph::new("loading...").block(Block::default().borders(Borders::ALL).title("Left"));
    let right =
        Paragraph::new("loading...").block(Block::default().borders(Borders::ALL).title("Right"));
    frame.render_widget(left, panes[0]);
    frame.render_widget(right, panes[1]);

    let footer =
        Paragraph::new(format!("{status} | q quit")).style(Style::default().fg(style::STATUS_FG));
    frame.render_widget(footer, root[2]);
}
