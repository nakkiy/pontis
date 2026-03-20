use similar::DiffTag;

use crate::diff::{LineEndingPolicy, WhitespacePolicy};
use crate::diff::{compute_line_ops, split_lines_keep_newline};
use crate::model::DiffFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineEnding {
    None,
    Cr,
    Lf,
    CrLf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mark {
    None,
    Changed,
    Current,
}

#[derive(Clone)]
pub(super) struct AlignedRow {
    pub(super) left: String,
    pub(super) right: String,
    pub(super) left_ending: LineEnding,
    pub(super) right_ending: LineEnding,
    pub(super) left_no: Option<usize>,
    pub(super) right_no: Option<usize>,
    pub(super) mark: Mark,
}

pub(super) fn build_aligned_rows(
    file: &DiffFile,
    current_hunk: usize,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> Vec<AlignedRow> {
    let left_lines = split_lines_keep_newline(&file.left_text);
    let right_lines = split_lines_keep_newline(&file.right_text);
    let ops = compute_line_ops(
        &left_lines,
        &right_lines,
        whitespace_policy,
        line_ending_policy,
    );
    let mut rows = Vec::new();
    let mut hunk_idx = 0usize;
    let mut left_no = 1usize;
    let mut right_no = 1usize;

    for op in ops {
        if op.tag == DiffTag::Equal {
            for (left_line, right_line) in left_lines[op.old_range.clone()]
                .iter()
                .zip(right_lines[op.new_range.clone()].iter())
            {
                let (left_text, left_ending) = split_line_and_ending(left_line);
                let (right_text, right_ending) = split_line_and_ending(right_line);
                rows.push(AlignedRow {
                    left: left_text,
                    right: right_text,
                    left_ending,
                    right_ending,
                    left_no: Some(left_no),
                    right_no: Some(right_no),
                    mark: Mark::None,
                });
                left_no += 1;
                right_no += 1;
            }
            continue;
        }

        let deleted = left_lines[op.old_range.clone()]
            .iter()
            .map(|line| split_line_and_ending(line))
            .collect::<Vec<_>>();
        let inserted = right_lines[op.new_range.clone()]
            .iter()
            .map(|line| split_line_and_ending(line))
            .collect::<Vec<_>>();

        let mark = if hunk_idx == current_hunk {
            Mark::Current
        } else {
            Mark::Changed
        };
        let len = deleted.len().max(inserted.len()).max(1);
        for i in 0..len {
            let has_left = i < deleted.len();
            let has_right = i < inserted.len();
            let (left_text, left_ending) = deleted
                .get(i)
                .cloned()
                .unwrap_or_else(|| (String::new(), LineEnding::None));
            let (right_text, right_ending) = inserted
                .get(i)
                .cloned()
                .unwrap_or_else(|| (String::new(), LineEnding::None));
            rows.push(AlignedRow {
                left: left_text,
                right: right_text,
                left_ending,
                right_ending,
                left_no: has_left.then_some(left_no),
                right_no: has_right.then_some(right_no),
                mark,
            });
            if has_left {
                left_no += 1;
            }
            if has_right {
                right_no += 1;
            }
        }
        hunk_idx += 1;
    }

    if rows.is_empty() {
        rows.push(AlignedRow {
            left: String::new(),
            right: String::new(),
            left_ending: LineEnding::None,
            right_ending: LineEnding::None,
            left_no: None,
            right_no: None,
            mark: Mark::None,
        });
    }
    rows
}

fn split_line_and_ending(value: &str) -> (String, LineEnding) {
    if let Some(line) = value.strip_suffix("\r\n") {
        return (line.to_string(), LineEnding::CrLf);
    }
    if let Some(line) = value.strip_suffix('\n') {
        return (line.to_string(), LineEnding::Lf);
    }
    if let Some(line) = value.strip_suffix('\r') {
        return (line.to_string(), LineEnding::Cr);
    }
    (value.to_string(), LineEnding::None)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{LineEnding, Mark, build_aligned_rows, split_line_and_ending};
    use crate::diff::{LineEndingPolicy, WhitespacePolicy};
    use crate::model::{DiffContent, DiffFile, EntryStatus};

    #[test]
    fn split_line_and_ending_recognizes_crlf() {
        assert_eq!(
            split_line_and_ending("abc\r\n"),
            ("abc".to_string(), LineEnding::CrLf)
        );
    }

    #[test]
    fn split_line_and_ending_recognizes_lf_cr_and_none() {
        assert_eq!(
            split_line_and_ending("abc\n"),
            ("abc".to_string(), LineEnding::Lf)
        );
        assert_eq!(
            split_line_and_ending("abc\r"),
            ("abc".to_string(), LineEnding::Cr)
        );
        assert_eq!(
            split_line_and_ending("abc"),
            ("abc".to_string(), LineEnding::None)
        );
    }

    #[test]
    fn ignore_whitespace_keeps_original_text_on_each_side() {
        let file = DiffFile::new_with_whitespace_policy(
            PathBuf::from("sample.txt"),
            None,
            None,
            DiffContent {
                left_text: "fn main() {\n    let x = 1;\n}\n".to_string(),
                right_text: "fn main() {\n\t let x = 1;   \n}\n".to_string(),
                left_bytes: 30,
                right_bytes: 31,
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
            EntryStatus::Unchanged,
            WhitespacePolicy::Ignore,
        );

        let rows = build_aligned_rows(
            &file,
            0,
            WhitespacePolicy::Ignore,
            LineEndingPolicy::Compare,
        );

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1].left, "    let x = 1;");
        assert_eq!(rows[1].right, "\t let x = 1;   ");
        assert_eq!(rows[1].mark, Mark::None);
        assert_eq!(rows[1].left_no, Some(2));
        assert_eq!(rows[1].right_no, Some(2));
    }
}
