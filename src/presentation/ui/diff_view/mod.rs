use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, Focus};
use crate::syntax::SyntaxPainter;

use super::{scroll, style};

mod align;
mod lines;

use self::align::build_aligned_rows;
use self::lines::{DiffLineRenderOpts, build_side_lines, pane_title};

const VERTICAL_SCROLL_MARGIN: usize = 1;

#[derive(Default)]
pub(crate) struct DiffViewRenderCache {
    entry: Option<DiffViewCacheEntry>,
}

struct DiffViewCacheEntry {
    file_idx: usize,
    epoch: u64,
    row_count: usize,
    max_render_width: usize,
    left_title: String,
    right_title: String,
    left_lines: Vec<Line<'static>>,
    right_lines: Vec<Line<'static>>,
}

pub(super) fn render_diff_view(
    frame: &mut Frame<'_>,
    app: &mut App,
    painter: &SyntaxPainter,
    cache: &mut DiffViewRenderCache,
    area: ratatui::layout::Rect,
) {
    let Some(file) = app.current_file().cloned() else {
        cache.entry = None;
        app.update_diff_scroll_limits(0, 0);
        let empty = Paragraph::new("no files").block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };
    if !file.loaded {
        cache.entry = None;
        app.update_diff_scroll_limits(0, 0);
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let left = Paragraph::new("loading...")
            .block(Block::default().borders(Borders::ALL).title("Left"));
        let right = Paragraph::new("loading...")
            .block(Block::default().borders(Borders::ALL).title("Right"));
        frame.render_widget(left, panes[0]);
        frame.render_widget(right, panes[1]);
        return;
    }

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let viewport_height = panes[0].height.saturating_sub(2);
    let viewport_width = panes[0].width.saturating_sub(2);
    let file_idx = app.current_file_index();
    let epoch = app.diff_view_epoch();

    let needs_rebuild = !matches!(
        cache.entry,
        Some(DiffViewCacheEntry {
            file_idx: cached_idx,
            epoch: cached_epoch,
            ..
        }) if cached_idx == file_idx && cached_epoch == epoch
    );

    if needs_rebuild {
        let rows = build_aligned_rows(
            &file,
            app.current_hunk(),
            app.settings().whitespace(),
            app.settings().line_endings(),
        );
        let render_opts = DiffLineRenderOpts {
            inline_diff: app.settings().compare.inline_diff,
            whitespace_policy: app.settings().whitespace(),
            show_line_numbers: app.settings().view.line_numbers,
            line_ending_visibility: app.settings().view.line_ending_visibility,
        };
        let left_lines = build_side_lines(&rows, &file, true, render_opts, painter);
        let right_lines = build_side_lines(&rows, &file, false, render_opts, painter);
        let left_title = pane_title(&file, true);
        let right_title = pane_title(&file, false);
        let max_render_width = max_render_width_cached(
            &rows,
            app.settings().view.line_numbers,
            app.settings().view.line_ending_visibility,
        );

        cache.entry = Some(DiffViewCacheEntry {
            file_idx,
            epoch,
            row_count: rows.len(),
            max_render_width,
            left_title,
            right_title,
            left_lines,
            right_lines,
        });
    }

    let Some(cached) = cache.entry.as_ref() else {
        return;
    };

    let max_scroll_y = max_vertical_scroll(cached.row_count, viewport_height);
    let max_scroll_x = max_horizontal_scroll_from_width(cached.max_render_width, viewport_width);
    app.update_diff_scroll_limits(max_scroll_y, max_scroll_x);
    app.sync_pending_hunk_focus();
    let left_window = visible_window(&cached.left_lines, app.scroll_y(), viewport_height);
    let right_window = visible_window(&cached.right_lines, app.scroll_y(), viewport_height);

    let mut left_block = Block::default()
        .borders(Borders::ALL)
        .title(cached.left_title.clone());
    let mut right_block = Block::default()
        .borders(Borders::ALL)
        .title(cached.right_title.clone());

    if app.focus() == Focus::Diff {
        left_block = left_block.border_style(Style::default().fg(style::FOCUS_BORDER_FG));
        right_block = right_block.border_style(Style::default().fg(style::FOCUS_BORDER_FG));
    }

    let left = Paragraph::new(left_window)
        .block(left_block)
        .scroll((0, app.scroll_x()));
    let right = Paragraph::new(right_window)
        .block(right_block)
        .scroll((0, app.scroll_x()));

    frame.render_widget(left, panes[0]);
    frame.render_widget(right, panes[1]);
}

fn max_vertical_scroll(row_count: usize, viewport_height: u16) -> u16 {
    if viewport_height == 0 {
        return 0;
    }
    scroll::scroll_limit_with_margin(row_count, viewport_height as usize, VERTICAL_SCROLL_MARGIN)
}

fn max_render_width_cached(
    rows: &[align::AlignedRow],
    show_line_numbers: bool,
    line_ending_visibility: crate::settings::LineEndingVisibility,
) -> usize {
    lines::max_render_width(rows, show_line_numbers, line_ending_visibility)
}

fn max_horizontal_scroll_from_width(max_width: usize, viewport_width: u16) -> u16 {
    if viewport_width == 0 {
        return 0;
    }
    scroll::scroll_limit_with_margin(
        max_width,
        viewport_width as usize,
        scroll::HORIZONTAL_SCROLL_MARGIN,
    )
}

fn visible_window(
    lines: &[Line<'static>],
    scroll_y: u16,
    viewport_height: u16,
) -> Vec<Line<'static>> {
    if viewport_height == 0 || lines.is_empty() {
        return Vec::new();
    }
    let start = (scroll_y as usize).min(lines.len());
    let end = (start + viewport_height as usize).min(lines.len());
    lines[start..end].to_vec()
}
