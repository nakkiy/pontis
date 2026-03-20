mod build;
mod command;

pub(crate) use build::collect_revision_pair_renames;
pub use build::{GitCompareMode, build_git_diff_files};
