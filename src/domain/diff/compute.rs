use std::ops::Range;

use similar::DiffTag;

use super::Hunk;
use crate::diff::split_lines_keep_newline;
use crate::diff::{LineEndingPolicy, WhitespacePolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LineOp {
    pub(crate) tag: DiffTag,
    pub(crate) old_range: Range<usize>,
    pub(crate) new_range: Range<usize>,
}

pub fn compute_hunks(
    left: &str,
    right: &str,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> Vec<Hunk> {
    let (left_lines, right_lines, ops) =
        line_ops_from_texts(left, right, whitespace_policy, line_ending_policy);
    let mut hunks = Vec::new();

    for op in ops {
        if op.tag == DiffTag::Equal {
            continue;
        }

        hunks.push(Hunk {
            old_start: op.old_range.start,
            old_end: op.old_range.end,
            new_start: op.new_range.start,
            new_end: op.new_range.end,
            old_lines: left_lines[op.old_range.clone()].to_vec(),
            new_lines: right_lines[op.new_range.clone()].to_vec(),
        });
    }

    hunks
}

pub(crate) fn hunk_display_row(
    left: &str,
    right: &str,
    hunk_idx: usize,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> Option<usize> {
    let (_left_lines, _right_lines, ops) =
        line_ops_from_texts(left, right, whitespace_policy, line_ending_policy);
    let mut current_hunk = 0usize;
    let mut row = 0usize;

    for op in ops {
        if op.tag == DiffTag::Equal {
            row += op.old_range.len();
            continue;
        }

        if current_hunk == hunk_idx {
            return Some(row);
        }

        row += op.old_range.len().max(op.new_range.len()).max(1);
        current_hunk += 1;
    }

    None
}

pub(crate) fn compute_line_ops(
    left_lines: &[String],
    right_lines: &[String],
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> Vec<LineOp> {
    let left_keys = normalized_line_keys(left_lines, whitespace_policy, line_ending_policy);
    let right_keys = normalized_line_keys(right_lines, whitespace_policy, line_ending_policy);
    let left_refs = left_keys.iter().map(String::as_str).collect::<Vec<_>>();
    let right_refs = right_keys.iter().map(String::as_str).collect::<Vec<_>>();
    similar::TextDiff::from_slices(&left_refs, &right_refs)
        .ops()
        .iter()
        .map(|op| LineOp {
            tag: op.tag(),
            old_range: op.old_range(),
            new_range: op.new_range(),
        })
        .collect()
}

pub(crate) fn texts_equal(
    left: &str,
    right: &str,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> bool {
    normalized_line_keys_from_text(left, whitespace_policy, line_ending_policy)
        == normalized_line_keys_from_text(right, whitespace_policy, line_ending_policy)
}

pub(crate) fn texts_equal_fast(
    left: &str,
    right: &str,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> bool {
    let mut left_norm =
        normalized_line_keys_from_text(left, whitespace_policy, line_ending_policy).into_iter();
    let mut right_norm =
        normalized_line_keys_from_text(right, whitespace_policy, line_ending_policy).into_iter();

    iter_eq(&mut left_norm, &mut right_norm)
}

fn iter_eq(
    left: &mut impl Iterator<Item = String>,
    right: &mut impl Iterator<Item = String>,
) -> bool {
    loop {
        match (left.next(), right.next()) {
            (Some(lhs), Some(rhs)) if lhs == rhs => {}
            (Some(_), Some(_)) => return false,
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn normalized_line_keys_from_text(
    text: &str,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> Vec<String> {
    let lines = split_lines_keep_newline(text);
    normalized_line_keys(&lines, whitespace_policy, line_ending_policy)
}

fn line_ops_from_texts(
    left: &str,
    right: &str,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> (Vec<String>, Vec<String>, Vec<LineOp>) {
    let left_lines = split_lines_keep_newline(left);
    let right_lines = split_lines_keep_newline(right);
    let ops = compute_line_ops(
        &left_lines,
        &right_lines,
        whitespace_policy,
        line_ending_policy,
    );
    (left_lines, right_lines, ops)
}

fn normalized_line_keys(
    lines: &[String],
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> Vec<String> {
    lines
        .iter()
        .map(|line| normalize_line_for_compare(line, whitespace_policy, line_ending_policy))
        .collect()
}

fn normalize_line_for_compare(
    line: &str,
    whitespace_policy: WhitespacePolicy,
    line_ending_policy: LineEndingPolicy,
) -> String {
    use LineEndingPolicy::{Compare as LeCompare, Ignore as LeIgnore};
    use WhitespacePolicy::{Compare, Ignore};

    match (whitespace_policy, line_ending_policy) {
        (Compare, LeCompare) => line.to_string(),
        (Compare, LeIgnore) => line.chars().filter(|&ch| ch != '\r').collect(),
        (Ignore, LeCompare) => line.chars().filter(|&ch| !ch.is_whitespace()).collect(),
        (Ignore, LeIgnore) => line
            .chars()
            .filter(|&ch| !ch.is_whitespace() && ch != '\r')
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_hunks, hunk_display_row, texts_equal, texts_equal_fast};
    use crate::diff::{LineEndingPolicy, WhitespacePolicy};

    #[test]
    fn compute_hunks_detects_changed_line() {
        let left = "a\nb\nc\n";
        let right = "a\nx\nc\n";
        let hunks = compute_hunks(
            left,
            right,
            WhitespacePolicy::Compare,
            LineEndingPolicy::Compare,
        );
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_start, 1);
        assert_eq!(hunks[0].new_start, 1);
    }

    #[test]
    fn hunk_display_row_accounts_for_prior_insert_delete_height() {
        let left = "a\nb\nc\nd\ne\n";
        let right = "a\nx\ny\nc\nz\ne\n";

        assert_eq!(
            hunk_display_row(
                left,
                right,
                0,
                WhitespacePolicy::Compare,
                LineEndingPolicy::Compare,
            ),
            Some(1)
        );
        assert_eq!(
            hunk_display_row(
                left,
                right,
                1,
                WhitespacePolicy::Compare,
                LineEndingPolicy::Compare,
            ),
            Some(4)
        );
    }

    #[test]
    fn compute_hunks_ignores_whitespace_when_requested() {
        let left = "fn main() {\n    let x = 1;\n}\n";
        let right = "fn main() {\n  let x = 1;   \n}\n";

        assert!(
            compute_hunks(
                left,
                right,
                WhitespacePolicy::Compare,
                LineEndingPolicy::Compare,
            )
            .len()
                == 1
        );
        assert!(
            compute_hunks(
                left,
                right,
                WhitespacePolicy::Ignore,
                LineEndingPolicy::Compare,
            )
            .is_empty()
        );
    }

    #[test]
    fn compute_hunks_can_ignore_line_ending_differences() {
        let left = "a\r\nb\r\n";
        let right = "a\nb\n";

        assert_eq!(
            compute_hunks(
                left,
                right,
                WhitespacePolicy::Compare,
                LineEndingPolicy::Compare,
            )
            .len(),
            1
        );
        assert!(
            compute_hunks(
                left,
                right,
                WhitespacePolicy::Compare,
                LineEndingPolicy::Ignore,
            )
            .is_empty()
        );
    }

    #[test]
    fn texts_equal_fast_matches_texts_equal() {
        let cases = [
            (
                "a\nb\n",
                "a\nb\n",
                WhitespacePolicy::Compare,
                LineEndingPolicy::Compare,
            ),
            (
                "a\r\nb\r\n",
                "a\nb\n",
                WhitespacePolicy::Compare,
                LineEndingPolicy::Ignore,
            ),
            (
                "a  b\n\tc\n",
                "ab\nc\n",
                WhitespacePolicy::Ignore,
                LineEndingPolicy::Compare,
            ),
            (
                "a \r\n\tb\r\n",
                "ab\n",
                WhitespacePolicy::Ignore,
                LineEndingPolicy::Ignore,
            ),
            (
                "x\n",
                "y\n",
                WhitespacePolicy::Compare,
                LineEndingPolicy::Compare,
            ),
        ];
        for (left, right, whitespace_policy, line_ending_policy) in cases {
            assert_eq!(
                texts_equal_fast(left, right, whitespace_policy, line_ending_policy),
                texts_equal(left, right, whitespace_policy, line_ending_policy)
            );
        }
    }
}
