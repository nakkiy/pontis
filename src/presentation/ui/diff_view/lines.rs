use ratatui::style::Style;
use ratatui::text::{Line, Span};
use similar::{ChangeTag, TextDiff};

use crate::diff::WhitespacePolicy;
use crate::model::DiffFile;
use crate::settings::LineEndingVisibility;
use crate::syntax::SyntaxPainter;

use super::super::style;
use super::align::{AlignedRow, LineEnding, Mark};

#[derive(Clone, Copy)]
pub(super) struct DiffLineRenderOpts {
    pub(super) inline_diff: bool,
    pub(super) whitespace_policy: WhitespacePolicy,
    pub(super) show_line_numbers: bool,
    pub(super) line_ending_visibility: LineEndingVisibility,
}

pub(super) fn build_side_lines(
    rows: &[AlignedRow],
    file: &DiffFile,
    is_left: bool,
    opts: DiffLineRenderOpts,
    painter: &SyntaxPainter,
) -> Vec<Line<'static>> {
    let lines = rows
        .iter()
        .map(|r| row_text(r, is_left))
        .collect::<Vec<_>>();
    let mut out: Vec<Line<'static>> = lines.iter().cloned().map(Line::from).collect();

    if !file.should_use_plain_render() {
        let highlighted = painter.highlight(&file.rel_path, &lines);
        for (idx, line) in out.iter_mut().enumerate() {
            *line = highlighted[idx].clone();
        }
    }

    if opts.inline_diff && should_apply_inline_diff(file, opts.whitespace_policy) {
        apply_inline_diff(&mut out, rows, is_left, opts.whitespace_policy);
    }

    if opts.show_line_numbers {
        prepend_line_numbers(&mut out, rows, is_left);
    }
    append_line_ending_symbols(&mut out, rows, is_left, opts.line_ending_visibility);
    apply_diff_background(&mut out, rows);

    out
}

pub(super) fn max_render_width(
    rows: &[AlignedRow],
    show_line_numbers: bool,
    line_ending_visibility: LineEndingVisibility,
) -> usize {
    let left_width = side_render_width(rows, true, show_line_numbers, line_ending_visibility);
    let right_width = side_render_width(rows, false, show_line_numbers, line_ending_visibility);
    left_width.max(right_width)
}

pub(super) fn pane_title(file: &DiffFile, is_left: bool) -> String {
    let side = if is_left { "Left" } else { "Right" };
    let side_unsupported = if is_left {
        file.left_has_unsupported_encoding
    } else {
        file.right_has_unsupported_encoding
    };
    let side_binary = if is_left {
        file.left_is_binary
    } else {
        file.right_is_binary
    };
    if side_unsupported {
        format!("{side} (non-utf8)")
    } else if side_binary {
        format!("{side} (binary)")
    } else {
        let has_bom = if is_left {
            file.left_has_utf8_bom
        } else {
            file.right_has_utf8_bom
        };
        let encoding = if has_bom { "UTF-8+BOM" } else { "UTF-8" };
        if file.highlight_limited {
            format!("{side} [{encoding}] (plain)")
        } else {
            format!("{side} [{encoding}]")
        }
    }
}

fn row_text(row: &AlignedRow, is_left: bool) -> String {
    if is_left {
        row.left.clone()
    } else {
        row.right.clone()
    }
}

fn row_line_ending(row: &AlignedRow, is_left: bool) -> LineEnding {
    if is_left {
        row.left_ending
    } else {
        row.right_ending
    }
}

fn row_line_no(row: &AlignedRow, is_left: bool) -> Option<usize> {
    if is_left { row.left_no } else { row.right_no }
}

fn side_render_width(
    rows: &[AlignedRow],
    is_left: bool,
    show_line_numbers: bool,
    line_ending_visibility: LineEndingVisibility,
) -> usize {
    let prefix_width = if show_line_numbers {
        let max_no = rows
            .iter()
            .filter_map(|row| row_line_no(row, is_left))
            .max()
            .unwrap_or(0);
        max_no.max(1).to_string().len() + 1
    } else {
        0
    };

    rows.iter()
        .map(|row| {
            prefix_width
                + row_text(row, is_left).chars().count()
                + line_ending_symbol(row_line_ending(row, is_left))
                    .filter(|_| should_show_line_ending_symbol(row.mark, line_ending_visibility))
                    .map(|s| s.chars().count())
                    .unwrap_or(0)
        })
        .max()
        .unwrap_or(prefix_width)
}

fn prepend_line_numbers(out: &mut [Line<'static>], rows: &[AlignedRow], is_left: bool) {
    let max_no = rows
        .iter()
        .filter_map(|row| row_line_no(row, is_left))
        .max()
        .unwrap_or(0);
    let width = max_no.max(1).to_string().len();
    for (line, row) in out.iter_mut().zip(rows.iter()) {
        let prefix = match row_line_no(row, is_left) {
            Some(n) => format!("{n:>width$} ", width = width),
            None => " ".repeat(width + 1),
        };
        let mut spans = Vec::with_capacity(line.spans.len() + 1);
        spans.push(Span::styled(prefix, Style::default().fg(style::LINE_NO_FG)));
        spans.extend(line.spans.clone());
        line.spans = spans;
    }
}

fn append_line_ending_symbols(
    out: &mut [Line<'static>],
    rows: &[AlignedRow],
    is_left: bool,
    line_ending_visibility: LineEndingVisibility,
) {
    for (line, row) in out.iter_mut().zip(rows.iter()) {
        if !should_show_line_ending_symbol(row.mark, line_ending_visibility) {
            continue;
        }
        let Some(symbol) = line_ending_symbol(row_line_ending(row, is_left)) else {
            continue;
        };
        let color = line_ending_color(row_line_ending(row, is_left));
        line.spans
            .push(Span::styled(symbol, Style::default().fg(color)));
    }
}

fn should_show_line_ending_symbol(mark: Mark, visibility: LineEndingVisibility) -> bool {
    match visibility {
        LineEndingVisibility::Hidden => false,
        LineEndingVisibility::All => true,
        LineEndingVisibility::DiffOnly => mark != Mark::None,
    }
}

fn line_ending_symbol(ending: LineEnding) -> Option<&'static str> {
    match ending {
        LineEnding::None => None,
        LineEnding::Cr => Some("←"),
        LineEnding::Lf => Some("↓"),
        LineEnding::CrLf => Some("↩"),
    }
}

fn line_ending_color(ending: LineEnding) -> ratatui::style::Color {
    match ending {
        LineEnding::None => style::STATUS_FG,
        LineEnding::Cr => style::LINE_ENDING_CR_FG,
        LineEnding::Lf => style::LINE_ENDING_LF_FG,
        LineEnding::CrLf => style::LINE_ENDING_CRLF_FG,
    }
}

fn apply_diff_background(out: &mut [Line<'static>], rows: &[AlignedRow]) {
    for (line, row) in out.iter_mut().zip(rows.iter()) {
        let bg = match row.mark {
            Mark::None => None,
            Mark::Changed => Some(style::DIFF_BG_CHANGED),
            Mark::Current => Some(style::DIFF_BG_CURRENT),
        };

        if let Some(bg_color) = bg {
            if line.spans.is_empty() {
                line.spans.push(Span::raw(" "));
            }
            for span in &mut line.spans {
                if span.style.bg.is_none() {
                    span.style = span.style.bg(bg_color);
                }
            }
        }
    }
}

fn apply_inline_diff(
    out: &mut [Line<'static>],
    rows: &[AlignedRow],
    is_left: bool,
    whitespace_policy: WhitespacePolicy,
) {
    for (line, row) in out.iter_mut().zip(rows.iter()) {
        if row.mark == Mark::None {
            continue;
        }
        let text = row_text(row, is_left);
        let other = row_text(row, !is_left);
        if text.is_empty() || other.is_empty() || text == other {
            continue;
        }

        let highlight =
            inline_diff_highlight_ranges(&row.left, &row.right, is_left, whitespace_policy);
        if highlight.is_empty() {
            continue;
        }

        let inline_bg = match row.mark {
            Mark::None => continue,
            Mark::Changed => style::INLINE_DIFF_BG_CHANGED,
            Mark::Current => style::INLINE_DIFF_BG_CURRENT,
        };
        line.spans = highlight_ranges_in_spans(&line.spans, &highlight, inline_bg);
    }
}

fn should_apply_inline_diff(file: &DiffFile, whitespace_policy: WhitespacePolicy) -> bool {
    let _ = whitespace_policy;
    !file.highlight_limited && !file.has_unsupported_encoding
}

fn inline_diff_highlight_ranges(
    left: &str,
    right: &str,
    is_left: bool,
    whitespace_policy: WhitespacePolicy,
) -> Vec<(usize, usize)> {
    let (left_cmp, left_map) = diff_text_and_index_map(left, whitespace_policy);
    let (right_cmp, right_map) = diff_text_and_index_map(right, whitespace_policy);
    let diff = TextDiff::from_chars(&left_cmp, &right_cmp);
    let mut ranges = Vec::new();
    let mut offset = 0usize;

    for change in diff.iter_all_changes() {
        let len = change.value().chars().count();
        let target = if is_left {
            ChangeTag::Delete
        } else {
            ChangeTag::Insert
        };

        match change.tag() {
            ChangeTag::Equal => offset += len,
            tag if tag == target => {
                let map = if is_left { &left_map } else { &right_map };
                for normalized_idx in offset..(offset + len) {
                    if let Some(&original_idx) = map.get(normalized_idx) {
                        push_or_extend_range(
                            &mut ranges,
                            original_idx,
                            original_idx.saturating_add(1),
                        );
                    }
                }
                offset += len;
            }
            _ => {}
        }
    }

    ranges
}

fn diff_text_and_index_map(
    text: &str,
    whitespace_policy: WhitespacePolicy,
) -> (String, Vec<usize>) {
    let mut diff_text = String::new();
    let mut index_map = Vec::new();

    for (idx, ch) in text.chars().enumerate() {
        if whitespace_policy == WhitespacePolicy::Ignore && ch.is_whitespace() {
            continue;
        }
        diff_text.push(ch);
        index_map.push(idx);
    }

    (diff_text, index_map)
}

fn push_or_extend_range(ranges: &mut Vec<(usize, usize)>, start: usize, end: usize) {
    if let Some((_, prev_end)) = ranges.last_mut()
        && *prev_end == start
    {
        *prev_end = end;
        return;
    }
    ranges.push((start, end));
}

fn highlight_ranges_in_spans(
    spans: &[Span<'static>],
    ranges: &[(usize, usize)],
    bg: ratatui::style::Color,
) -> Vec<Span<'static>> {
    let mut out = Vec::new();
    let mut cursor = 0usize;

    for span in spans {
        let content = span.content.as_ref();
        let char_len = content.chars().count();
        if char_len == 0 {
            out.push(span.clone());
            continue;
        }

        let span_start = cursor;
        let span_end = cursor + char_len;
        let mut local = span_start;

        for &(range_start, range_end) in ranges {
            if range_end <= span_start || span_end <= range_start {
                continue;
            }

            let overlap_start = range_start.max(span_start);
            let overlap_end = range_end.min(span_end);

            if local < overlap_start {
                out.push(slice_span(
                    span,
                    local - span_start,
                    overlap_start - span_start,
                    false,
                    bg,
                ));
            }
            out.push(slice_span(
                span,
                overlap_start - span_start,
                overlap_end - span_start,
                true,
                bg,
            ));
            local = overlap_end;
        }

        if local < span_end {
            out.push(slice_span(
                span,
                local - span_start,
                span_end - span_start,
                false,
                bg,
            ));
        }

        cursor = span_end;
    }

    out
}

fn slice_span(
    span: &Span<'static>,
    start: usize,
    end: usize,
    highlight: bool,
    bg: ratatui::style::Color,
) -> Span<'static> {
    let content = span
        .content
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect::<String>();
    let style = if highlight {
        span.style.bg(bg)
    } else {
        span.style
    };
    Span::styled(content, style)
}

#[cfg(test)]
mod tests {
    use super::{
        LineEnding, apply_diff_background, diff_text_and_index_map, highlight_ranges_in_spans,
        inline_diff_highlight_ranges, line_ending_symbol, pane_title, should_apply_inline_diff,
        should_show_line_ending_symbol,
    };
    use crate::diff::WhitespacePolicy;
    use crate::model::{DiffContent, DiffFile, EntryStatus};
    use crate::settings::LineEndingVisibility;
    use crate::ui::diff_view::align::Mark;
    use crate::ui::style;
    use ratatui::style::Style;
    use ratatui::text::{Line, Span};
    use std::path::PathBuf;

    #[test]
    fn line_ending_symbol_maps_each_code_to_distinct_marker() {
        assert_eq!(line_ending_symbol(LineEnding::Cr), Some("←"));
        assert_eq!(line_ending_symbol(LineEnding::Lf), Some("↓"));
        assert_eq!(line_ending_symbol(LineEnding::CrLf), Some("↩"));
        assert_eq!(line_ending_symbol(LineEnding::None), None);
    }

    #[test]
    fn line_ending_visibility_controls_symbol_output() {
        assert!(!should_show_line_ending_symbol(
            Mark::Changed,
            LineEndingVisibility::Hidden
        ));
        assert!(should_show_line_ending_symbol(
            Mark::None,
            LineEndingVisibility::All
        ));
        assert!(!should_show_line_ending_symbol(
            Mark::None,
            LineEndingVisibility::DiffOnly
        ));
        assert!(should_show_line_ending_symbol(
            Mark::Current,
            LineEndingVisibility::DiffOnly
        ));
    }

    #[test]
    fn pane_title_shows_side_specific_bom_status() {
        let file = DiffFile::new(
            PathBuf::from("a.txt"),
            None,
            None,
            DiffContent {
                left_text: "hello\n".to_string(),
                right_text: "hello\n".to_string(),
                left_bytes: 6,
                right_bytes: 6,
                left_is_binary: false,
                right_is_binary: false,
                is_binary: false,
                has_unsupported_encoding: false,
                left_has_unsupported_encoding: false,
                right_has_unsupported_encoding: false,
                left_has_utf8_bom: true,
                right_has_utf8_bom: false,
                highlight_limited: false,
            },
            EntryStatus::Unchanged,
        );
        assert_eq!(pane_title(&file, true), "Left [UTF-8+BOM]");
        assert_eq!(pane_title(&file, false), "Right [UTF-8]");
    }

    #[test]
    fn pane_title_shows_non_utf8_only_on_affected_side() {
        let file = DiffFile::new(
            PathBuf::from("a.txt"),
            None,
            None,
            DiffContent {
                left_text: "[unsupported text encoding]".to_string(),
                right_text: "hello\n".to_string(),
                left_bytes: 3,
                right_bytes: 6,
                left_is_binary: true,
                right_is_binary: false,
                is_binary: true,
                has_unsupported_encoding: true,
                left_has_unsupported_encoding: true,
                right_has_unsupported_encoding: false,
                left_has_utf8_bom: false,
                right_has_utf8_bom: true,
                highlight_limited: false,
            },
            EntryStatus::Modified,
        );
        assert_eq!(pane_title(&file, true), "Left (non-utf8)");
        assert_eq!(pane_title(&file, false), "Right [UTF-8+BOM]");
    }

    #[test]
    fn inline_diff_highlight_ranges_marks_replaced_segments_per_side() {
        assert_eq!(
            inline_diff_highlight_ranges("abcXYZ", "abc123", true, WhitespacePolicy::Compare),
            vec![(3, 6)]
        );
        assert_eq!(
            inline_diff_highlight_ranges("abcXYZ", "abc123", false, WhitespacePolicy::Compare),
            vec![(3, 6)]
        );
    }

    #[test]
    fn inline_diff_highlight_ranges_ignore_whitespace_only_highlights_non_whitespace_changes() {
        assert_eq!(
            inline_diff_highlight_ranges(
                "int a = 0;",
                "double  a = 0;",
                true,
                WhitespacePolicy::Ignore
            ),
            vec![(0, 3)]
        );
        assert_eq!(
            inline_diff_highlight_ranges(
                "int a = 0;",
                "double  a = 0;",
                false,
                WhitespacePolicy::Ignore
            ),
            vec![(0, 6)]
        );
    }

    #[test]
    fn diff_text_and_index_map_skips_whitespace_under_ignore_policy() {
        let (diff_text, index_map) =
            diff_text_and_index_map("double  a = 0;", WhitespacePolicy::Ignore);

        assert_eq!(diff_text, "doublea=0;");
        assert_eq!(index_map, vec![0, 1, 2, 3, 4, 5, 8, 10, 12, 13]);
    }

    #[test]
    fn highlight_ranges_in_spans_preserves_unhighlighted_segments() {
        let spans = vec![Span::styled("abcdef", Style::default())];
        let result = highlight_ranges_in_spans(&spans, &[(2, 4)], style::INLINE_DIFF_BG_CHANGED);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].content.as_ref(), "ab");
        assert_eq!(result[1].content.as_ref(), "cd");
        assert_eq!(result[2].content.as_ref(), "ef");
        assert_eq!(result[1].style.bg, Some(style::INLINE_DIFF_BG_CHANGED));
    }

    #[test]
    fn diff_background_preserves_existing_inline_background() {
        let mut lines = vec![Line::from(vec![
            Span::styled("ab", Style::default().bg(style::INLINE_DIFF_BG_CHANGED)),
            Span::styled("cd", Style::default()),
        ])];
        let rows = vec![crate::ui::diff_view::align::AlignedRow {
            left: "abcd".to_string(),
            right: "abXY".to_string(),
            left_ending: LineEnding::None,
            right_ending: LineEnding::None,
            left_no: Some(1),
            right_no: Some(1),
            mark: Mark::Changed,
        }];

        apply_diff_background(&mut lines, &rows);

        assert_eq!(
            lines[0].spans[0].style.bg,
            Some(style::INLINE_DIFF_BG_CHANGED)
        );
        assert_eq!(lines[0].spans[1].style.bg, Some(style::DIFF_BG_CHANGED));
    }

    #[test]
    fn inline_diff_is_disabled_for_large_or_unsupported_files() {
        let mut large = sample_file();
        large.highlight_limited = true;
        assert!(!should_apply_inline_diff(&large, WhitespacePolicy::Compare));

        let mut unsupported = sample_file();
        unsupported.has_unsupported_encoding = true;
        assert!(!should_apply_inline_diff(
            &unsupported,
            WhitespacePolicy::Compare
        ));

        assert!(should_apply_inline_diff(
            &sample_file(),
            WhitespacePolicy::Compare
        ));
        assert!(should_apply_inline_diff(
            &sample_file(),
            WhitespacePolicy::Ignore
        ));
    }

    fn sample_file() -> DiffFile {
        DiffFile::new(
            PathBuf::from("sample.txt"),
            None,
            None,
            DiffContent {
                left_text: "hello\n".to_string(),
                right_text: "hullo\n".to_string(),
                left_bytes: 6,
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
