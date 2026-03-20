mod compute;
mod file_ops;
mod lines;
mod policy;

pub use crate::model::Hunk;
pub use compute::compute_hunks;
pub(crate) use compute::{compute_line_ops, hunk_display_row, texts_equal, texts_equal_fast};
pub use lines::{join_lines, split_lines_keep_newline};
pub use policy::{DiffComparePolicies, LineEndingPolicy, WhitespacePolicy};
