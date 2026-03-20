mod build;
mod load;
mod reader;

pub use build::{
    PrecomputedRename, build_diff_files, build_diff_files_with_config,
    build_diff_files_with_precomputed_renames,
};
pub(crate) use load::FsDiffLoader;
