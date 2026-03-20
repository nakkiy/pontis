use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::app::{App, Focus};
use crate::model::EntryStatus;

use super::{scroll, style};

pub(super) fn render_file_list(frame: &mut Frame<'_>, app: &mut App, area: ratatui::layout::Rect) {
    let mut block = Block::default().borders(Borders::ALL).title("Files");
    if app.focus() == Focus::FileList {
        block = block.border_style(Style::default().fg(style::FOCUS_BORDER_FG));
    }
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let filter_single_line = format!(
        "A:{}   M:{}   D:{}   R:{}   =:{}",
        visibility_word(app.show_added()),
        visibility_word(app.show_modified()),
        visibility_word(app.show_deleted()),
        visibility_word(app.show_renamed()),
        visibility_word(app.show_unchanged())
    );
    let filter_inner_width = area.width.saturating_sub(4) as usize;
    let needs_two_line_filter = filter_single_line.chars().count() > filter_inner_width;
    let filter_height = if needs_two_line_filter { 4 } else { 3 };
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(filter_height)])
        .split(inner);
    let list_area = sections[0];
    let filter_area = sections[1];

    let viewport_height = list_area.height;
    let viewport_width = list_area.width;
    let max_scroll_y = max_vertical_scroll(app.visible_file_indices().len(), viewport_height);
    let max_scroll_x = max_horizontal_scroll(app, viewport_width);
    app.update_file_list_scroll_limits(max_scroll_y, max_scroll_x, viewport_height);

    let list_scroll_x = app.file_list_scroll_x() as usize;
    let selected_row = app.current_visible_file_row();
    let items = app
        .visible_file_indices()
        .iter()
        .enumerate()
        .map(|(row, &idx)| {
            let file = &app.files()[idx];
            let selected = Some(row) == selected_row;
            let mut style = Style::default();
            if selected {
                style = style
                    .bg(style::SELECTED_BG)
                    .fg(style::SELECTED_FG)
                    .add_modifier(Modifier::BOLD);
            }
            let marker = status_marker(file.status);
            let binary = if file.is_binary { "B " } else { "" };
            let dirty = if file.left_dirty || file.right_dirty {
                " *"
            } else {
                ""
            };
            let rel_path = display_rel_path(file);
            let clipped = clip_prefix_chars(&rel_path, list_scroll_x);
            ListItem::new(Line::from(format!("{marker} {binary}{clipped}{dirty}"))).style(style)
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default()
        .with_selected(selected_row)
        .with_offset(app.file_list_scroll_y() as usize);
    frame.render_stateful_widget(List::new(items), list_area, &mut state);

    let filter_text = if needs_two_line_filter {
        format!(
            "A:{}   M:{}\nD:{}   R:{}   =:{}",
            visibility_word(app.show_added()),
            visibility_word(app.show_modified()),
            visibility_word(app.show_deleted()),
            visibility_word(app.show_renamed()),
            visibility_word(app.show_unchanged())
        )
    } else {
        filter_single_line
    };
    let filter =
        Paragraph::new(filter_text).block(Block::default().borders(Borders::ALL).title("Filter"));
    frame.render_widget(filter, filter_area);
}

fn status_marker(status: EntryStatus) -> &'static str {
    match status {
        EntryStatus::Pending => "?",
        EntryStatus::Unchanged => "=",
        EntryStatus::Modified => "M",
        EntryStatus::Renamed => "R",
        EntryStatus::Added => "A",
        EntryStatus::Deleted => "D",
    }
}

fn max_vertical_scroll(file_count: usize, viewport_height: u16) -> u16 {
    if viewport_height == 0 {
        return 0;
    }
    scroll::scroll_limit_with_margin(file_count, viewport_height as usize, 0)
}

fn max_horizontal_scroll(app: &App, viewport_width: u16) -> u16 {
    if viewport_width == 0 {
        return 0;
    }
    app.visible_file_indices()
        .iter()
        .map(|&idx| &app.files()[idx])
        .map(|file| {
            let marker_len = status_marker(file.status).chars().count();
            let binary_len = if file.is_binary { 2 } else { 0 };
            let dirty_len = if file.left_dirty || file.right_dirty {
                2
            } else {
                0
            };
            let fixed = marker_len + 1 + binary_len + dirty_len;
            let path_len = display_rel_path(file).chars().count();
            let path_budget = viewport_width.saturating_sub(fixed as u16) as usize;
            scroll::scroll_limit_with_margin(
                path_len,
                path_budget,
                scroll::HORIZONTAL_SCROLL_MARGIN,
            )
        })
        .max()
        .unwrap_or(0)
}

fn visibility_word(visible: bool) -> &'static str {
    if visible { "show" } else { "hide" }
}

fn display_rel_path(file: &crate::model::DiffFile) -> String {
    let Some(old_path) = file.original_rel_path.as_ref() else {
        return file.rel_path.display().to_string();
    };
    format_compact_rename_path(old_path, &file.rel_path)
}

fn format_compact_rename_path(old_path: &std::path::Path, new_path: &std::path::Path) -> String {
    let old = path_segments(old_path);
    let new = path_segments(new_path);
    let mut prefix_len = 0usize;
    let max_prefix = old.len().min(new.len());
    while prefix_len < max_prefix && old[prefix_len] == new[prefix_len] {
        prefix_len += 1;
    }

    let mut suffix_len = 0usize;
    let max_suffix = old.len().min(new.len()).saturating_sub(prefix_len);
    while suffix_len < max_suffix
        && old[old.len() - 1 - suffix_len] == new[new.len() - 1 - suffix_len]
    {
        suffix_len += 1;
    }

    let prefix = if prefix_len == 0 {
        String::new()
    } else {
        format!("{}/", old[..prefix_len].join("/"))
    };
    let suffix = if suffix_len == 0 {
        String::new()
    } else {
        format!("/{}", old[old.len() - suffix_len..].join("/"))
    };
    let old_mid = old[prefix_len..old.len() - suffix_len].join("/");
    let new_mid = new[prefix_len..new.len() - suffix_len].join("/");
    format!("{prefix}{{{old_mid} => {new_mid}}}{suffix}")
}

fn path_segments(path: &std::path::Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect()
}

fn clip_prefix_chars(value: &str, chars: usize) -> String {
    value.chars().skip(chars).collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{clip_prefix_chars, format_compact_rename_path, max_vertical_scroll};

    #[test]
    fn clip_prefix_chars_skips_requested_characters() {
        assert_eq!(clip_prefix_chars("abcdef", 2), "cdef");
        assert_eq!(clip_prefix_chars("abcdef", 10), "");
    }

    #[test]
    fn max_vertical_scroll_uses_viewport_height() {
        assert_eq!(max_vertical_scroll(2, 3), 0);
        assert_eq!(max_vertical_scroll(5, 2), 3);
    }

    #[test]
    fn compact_rename_path_collapses_common_segments() {
        assert_eq!(
            format_compact_rename_path(
                Path::new("src/application/app/core/access.rs"),
                Path::new("src/application/app/access.rs")
            ),
            "src/application/app/{core => }/access.rs"
        );
        assert_eq!(
            format_compact_rename_path(Path::new("src/old.rs"), Path::new("src/new.rs")),
            "src/{old.rs => new.rs}"
        );
    }
}
