use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

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

    if app.help_open() {
        render_help_overlay(frame, app);
    }
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

fn render_help_overlay(frame: &mut Frame<'_>, _app: &App) {
    let lines = help_lines();
    let area = help_overlay_area(frame.area(), &lines);
    let help = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(Style::default().fg(style::FOCUS_BORDER_FG)),
        )
        .style(Style::default().fg(style::STATUS_FG));
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn help_overlay_area(area: Rect, lines: &[Line<'static>]) -> Rect {
    let vertical_margin = match area.height {
        0..=12 => 1,
        13..=24 => 2,
        _ => 3,
    };
    let horizontal_margin = match area.width {
        0..=50 => 1,
        51..=80 => 3,
        81..=120 => 6,
        _ => 10,
    };

    let max_width = area.width.saturating_sub(horizontal_margin * 2).max(1);
    let max_height = area.height.saturating_sub(vertical_margin * 2).max(1);

    let content_width = lines
        .iter()
        .map(help_line_width)
        .max()
        .unwrap_or(0)
        .saturating_add(4) as u16;
    let content_height = lines.len().saturating_add(2) as u16;

    let min_width = max_width.min(area.width.min(24)).max(1);
    let min_height = max_height.min(area.height.min(8)).max(1);
    let desired_width = content_width.clamp(min_width, max_width);
    let desired_height = content_height.clamp(min_height, max_height);

    let centered_width = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(desired_width)])
        .flex(Flex::Center)
        .split(area)[0];

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(desired_height)])
        .flex(Flex::Center)
        .split(Rect {
            x: centered_width.x,
            y: area.y,
            width: centered_width.width,
            height: area.height,
        })[0]
}

fn help_line_width(line: &Line<'_>) -> usize {
    line.spans.iter().map(|span| span.content.len()).sum()
}

fn help_lines() -> Vec<Line<'static>> {
    vec![
        help_section("Common"),
        help_item("q", "Quit"),
        help_item("? / esc", "Close help"),
        help_item("l", "Reload comparison"),
        help_item("s / S", "Save current / save all"),
        help_item("u / r", "Undo / redo"),
        help_item("e / E", "Open right / left in editor"),
        Line::from(""),
        help_section("File List"),
        help_item("enter", "Open diff"),
        help_item("↑ / ↓", "Move selection"),
        help_item("PageUp / PageDown", "Move by 10 entries"),
        help_item("← / →", "Horizontal scroll"),
        help_item("Home / End", "Jump to horizontal edge"),
        help_item("A / M / D / R / =", "Toggle status filter"),
        help_item("f", "Reset filters"),
        help_item("alt+↑ / ↓", "Previous / next change"),
        Line::from(""),
        help_section("Diff"),
        help_item("esc", "Back to file list"),
        help_item("↑ / ↓", "Vertical scroll"),
        help_item("← / →", "Horizontal scroll"),
        help_item("PageUp / PageDown", "Scroll by 10 lines"),
        help_item("Home / End", "Jump to horizontal edge"),
        help_item("alt+↑ / ↓", "Previous / next change"),
        help_item("alt+← / →", "Apply current hunk"),
    ]
}

fn help_section(title: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(style::HEADER_FG)
            .add_modifier(Modifier::BOLD),
    ))
}

fn help_item(keys: &'static str, description: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{keys:<22}"),
            Style::default()
                .fg(style::FOCUS_BORDER_FG)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(description),
    ])
}
