use std::path::{Path, PathBuf};

use anyhow::Result;

use super::GitCompareMode;
use crate::infrastructure::git_scan::command::{
    GitStatusEntry, git_diff_entries, git_diff_entries_between, git_untracked_entries,
};

pub(super) fn collect_git_entries(
    repo_root: &Path,
    resolved_left_rev: &str,
    resolved_right_rev: Option<&str>,
    mode: &GitCompareMode,
) -> Result<Vec<GitStatusEntry>> {
    let mut entries = match mode {
        GitCompareMode::WorkingTree => git_diff_entries(repo_root, resolved_left_rev, false)?,
        GitCompareMode::Staged => git_diff_entries(repo_root, resolved_left_rev, true)?,
        GitCompareMode::RevisionPair { .. } => git_diff_entries_between(
            repo_root,
            resolved_left_rev,
            resolved_right_rev.expect("revision pair right rev"),
        )?,
    };
    if *mode == GitCompareMode::WorkingTree {
        entries.extend(git_untracked_entries(repo_root)?);
    }
    Ok(entries)
}

pub(super) fn build_left_root(repo_root: &Path, left_rev: &str) -> PathBuf {
    if left_rev == "HEAD" {
        repo_root.to_path_buf()
    } else {
        repo_root.join(format!(".git:{left_rev}"))
    }
}
