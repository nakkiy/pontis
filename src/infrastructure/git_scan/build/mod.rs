use std::path::Path;

use anyhow::Result;

use crate::model::{DiffFile, Mode, Roots};
use crate::settings::AppSettings;

use super::command::{GitEntryChange, git_resolve_compare_rev, git_toplevel};
use crate::infrastructure::fs_scan::PrecomputedRename;

mod entries;
mod materialize;

use self::entries::{build_left_root, collect_git_entries};
use self::materialize::build_files_from_entries;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitCompareMode {
    WorkingTree,
    Staged,
    RevisionPair { left: String, right: String },
}

pub fn build_git_diff_files(
    repo: &Path,
    cfg: &AppSettings,
    mode: &GitCompareMode,
    default_left_rev: &str,
) -> Result<(Vec<DiffFile>, Roots)> {
    let repo_root = git_toplevel(repo)?;
    let (requested_left_rev, requested_right_rev) = match mode {
        GitCompareMode::WorkingTree | GitCompareMode::Staged => {
            (default_left_rev.to_string(), None)
        }
        GitCompareMode::RevisionPair { left, right } => (left.clone(), Some(right.clone())),
    };
    let resolved_left_rev = git_resolve_compare_rev(&repo_root, &requested_left_rev)?;
    let resolved_right_rev = requested_right_rev
        .as_deref()
        .map(|rev| git_resolve_compare_rev(&repo_root, rev))
        .transpose()?;
    let entries = collect_git_entries(
        &repo_root,
        &resolved_left_rev,
        resolved_right_rev.as_deref(),
        mode,
    )?;
    let mut files = build_files_from_entries(
        &repo_root,
        cfg,
        mode,
        &resolved_left_rev,
        resolved_right_rev.as_deref(),
        entries,
    )?;

    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    files.dedup_by(|a, b| a.rel_path == b.rel_path);

    let left_root = build_left_root(&repo_root, &requested_left_rev);
    let right_root = match mode {
        GitCompareMode::WorkingTree | GitCompareMode::Staged => repo_root.clone(),
        GitCompareMode::RevisionPair { right, .. } => build_left_root(&repo_root, right),
    };

    Ok((
        files,
        Roots {
            left: left_root,
            right: right_root,
            mode: Mode::Directory,
            left_label: None,
            right_label: None,
        },
    ))
}

pub(crate) fn collect_revision_pair_renames(
    repo: &Path,
    left_rev: &str,
    right_rev: &str,
) -> Result<(std::path::PathBuf, String, String, Vec<PrecomputedRename>)> {
    let repo_root = git_toplevel(repo)?;
    let resolved_left_rev = git_resolve_compare_rev(&repo_root, left_rev)?;
    let resolved_right_rev = git_resolve_compare_rev(&repo_root, right_rev)?;
    let entries = super::command::git_diff_entries_between(
        &repo_root,
        &resolved_left_rev,
        &resolved_right_rev,
    )?;
    let renames = entries
        .into_iter()
        .filter(|entry| entry.change == GitEntryChange::Renamed)
        .filter_map(|entry| {
            Some(PrecomputedRename {
                old_rel: entry.original_path?,
                new_rel: entry.path,
            })
        })
        .collect();
    Ok((repo_root, resolved_left_rev, resolved_right_rev, renames))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::diff::{DiffComparePolicies, WhitespacePolicy};
    use crate::model::EntryStatus;
    use crate::settings::AppSettings;
    use crate::text::{BINARY_PLACEHOLDER, DecodedKind, decode_bytes_for_diff};

    use super::materialize::{EntryPresence, derive_status};
    use super::{GitCompareMode, build_git_diff_files};

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{uniq}"))
    }

    fn git(repo: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .expect("run git");
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn derive_status_from_presence_and_equality() {
        assert_eq!(
            derive_status(
                EntryPresence {
                    left_exists: true,
                    right_exists: true,
                },
                true,
                true,
                false
            ),
            EntryStatus::Unchanged
        );
        assert_eq!(
            derive_status(
                EntryPresence {
                    left_exists: true,
                    right_exists: true,
                },
                false,
                false,
                false
            ),
            EntryStatus::Modified
        );
        assert_eq!(
            derive_status(
                EntryPresence {
                    left_exists: false,
                    right_exists: true,
                },
                false,
                false,
                false
            ),
            EntryStatus::Added
        );
        assert_eq!(
            derive_status(
                EntryPresence {
                    left_exists: true,
                    right_exists: true,
                },
                true,
                true,
                true
            ),
            EntryStatus::Renamed
        );
    }

    #[test]
    fn display_text_marks_binary() {
        let decoded = decode_bytes_for_diff(vec![0, 1, 2]);
        assert_ne!(decoded.kind, DecodedKind::TextUtf8);
        assert_eq!(decoded.bytes, 3);
        assert_eq!(decoded.text, BINARY_PLACEHOLDER);
    }

    #[test]
    fn git_mode_marks_new_file_as_added_when_missing_in_head() {
        let repo = unique_temp_dir("pontis-git-added");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("tracked.txt"), "base\n").expect("write tracked");
        git(&repo, &["add", "tracked.txt"]);
        git(&repo, &["commit", "-qm", "init"]);

        fs::write(repo.join("added.txt"), "new\n").expect("write added");
        git(&repo, &["add", "added.txt"]);

        let (files, _) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::WorkingTree,
            "HEAD",
        )
        .expect("scan repo");
        let added = files
            .into_iter()
            .find(|file| file.rel_path == Path::new("added.txt"))
            .expect("added entry");

        assert_eq!(added.status, EntryStatus::Added);
        assert!(added.left_path.is_none());
        assert!(added.right_path.is_some());

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_mode_keeps_rename_entries_visible() {
        let repo = unique_temp_dir("pontis-git-rename");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::create_dir_all(repo.join("dir")).expect("mkdir dir");
        fs::write(repo.join("dir/old.txt"), "rename me\n").expect("write old");
        git(&repo, &["add", "dir/old.txt"]);
        git(&repo, &["commit", "-qm", "init"]);

        fs::rename(repo.join("dir/old.txt"), repo.join("dir/new.txt")).expect("rename");
        git(&repo, &["add", "-A"]);

        let (files, _) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::WorkingTree,
            "HEAD",
        )
        .expect("scan repo");
        let renamed = files
            .into_iter()
            .find(|file| file.rel_path == Path::new("dir/new.txt"))
            .expect("rename entry");

        assert_eq!(renamed.status, EntryStatus::Renamed);
        assert_eq!(
            renamed.original_rel_path.as_deref(),
            Some(Path::new("dir/old.txt"))
        );
        assert_eq!(
            renamed.left_path.as_deref(),
            Some(repo.join("dir/old.txt").as_path())
        );
        assert_eq!(
            renamed.right_path.as_deref(),
            Some(repo.join("dir/new.txt").as_path())
        );

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_mode_treats_bom_only_difference_as_unchanged() {
        let repo = unique_temp_dir("pontis-git-bom");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("bom.txt"), b"\xEF\xBB\xBFhello\n").expect("write with bom");
        git(&repo, &["add", "bom.txt"]);
        git(&repo, &["commit", "-qm", "init"]);

        fs::write(repo.join("bom.txt"), b"hello\n").expect("write without bom");

        let (files, _) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::WorkingTree,
            "HEAD",
        )
        .expect("scan repo");
        let entry = files
            .into_iter()
            .find(|file| file.rel_path == Path::new("bom.txt"))
            .expect("bom entry");
        assert_eq!(entry.status, EntryStatus::Unchanged);

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_staged_mode_uses_index_content_only() {
        let repo = unique_temp_dir("pontis-git-staged");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("a.txt"), "base\n").expect("write base");
        git(&repo, &["add", "a.txt"]);
        git(&repo, &["commit", "-qm", "init"]);

        fs::write(repo.join("a.txt"), "staged\n").expect("write staged");
        git(&repo, &["add", "a.txt"]);
        fs::write(repo.join("a.txt"), "worktree\n").expect("write worktree");

        let (files, _) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::Staged,
            "HEAD",
        )
        .expect("scan repo");
        let entry = files
            .into_iter()
            .find(|file| file.rel_path == Path::new("a.txt"))
            .expect("staged entry");
        assert_eq!(entry.left_text, "base\n");
        assert_eq!(entry.right_text, "staged\n");

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_staged_mode_ignores_unstaged_only_changes() {
        let repo = unique_temp_dir("pontis-git-staged-filter");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("a.txt"), "base\n").expect("write base");
        git(&repo, &["add", "a.txt"]);
        git(&repo, &["commit", "-qm", "init"]);

        fs::write(repo.join("a.txt"), "unstaged\n").expect("write unstaged");

        let (files, _) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::Staged,
            "HEAD",
        )
        .expect("scan repo");
        assert!(files.is_empty());

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_mode_can_ignore_whitespace_only_worktree_changes() {
        let repo = unique_temp_dir("pontis-git-whitespace-worktree");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("a.txt"), "fn main() {\n    let x = 1;\n}\n").expect("write base");
        git(&repo, &["add", "a.txt"]);
        git(&repo, &["commit", "-qm", "init"]);

        fs::write(repo.join("a.txt"), "fn main() {\n\t let x = 1;   \n}\n")
            .expect("write whitespace-only change");

        let (strict_files, _) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::WorkingTree,
            "HEAD",
        )
        .expect("scan strict");
        let strict_entry = strict_files
            .into_iter()
            .find(|file| file.rel_path == Path::new("a.txt"))
            .expect("worktree entry");
        assert_eq!(strict_entry.status, EntryStatus::Modified);
        assert!(!strict_entry.hunks.is_empty());

        let cfg = AppSettings {
            compare_policies: DiffComparePolicies::with_whitespace(WhitespacePolicy::Ignore),
            ..AppSettings::default()
        };
        let (ignore_files, _) =
            build_git_diff_files(&repo, &cfg, &GitCompareMode::WorkingTree, "HEAD")
                .expect("scan ignore");
        let ignore_entry = ignore_files
            .into_iter()
            .find(|file| file.rel_path == Path::new("a.txt"))
            .expect("worktree entry");
        assert_eq!(ignore_entry.status, EntryStatus::Unchanged);
        assert!(ignore_entry.hunks.is_empty());
        assert_eq!(ignore_entry.left_text, "fn main() {\n    let x = 1;\n}\n");
        assert_eq!(
            ignore_entry.right_text,
            "fn main() {\n\t let x = 1;   \n}\n"
        );

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_staged_mode_can_ignore_whitespace_only_index_changes() {
        let repo = unique_temp_dir("pontis-git-whitespace-staged");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("a.txt"), "fn main() {\n    let x = 1;\n}\n").expect("write base");
        git(&repo, &["add", "a.txt"]);
        git(&repo, &["commit", "-qm", "init"]);

        fs::write(repo.join("a.txt"), "fn main() {\n\t let x = 1;   \n}\n").expect("write staged");
        git(&repo, &["add", "a.txt"]);

        let (strict_files, _) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::Staged,
            "HEAD",
        )
        .expect("scan strict");
        let strict_entry = strict_files
            .into_iter()
            .find(|file| file.rel_path == Path::new("a.txt"))
            .expect("staged entry");
        assert_eq!(strict_entry.status, EntryStatus::Modified);
        assert!(!strict_entry.hunks.is_empty());

        let cfg = AppSettings {
            compare_policies: DiffComparePolicies::with_whitespace(WhitespacePolicy::Ignore),
            ..AppSettings::default()
        };
        let (ignore_files, _) = build_git_diff_files(&repo, &cfg, &GitCompareMode::Staged, "HEAD")
            .expect("scan ignore");
        let ignore_entry = ignore_files
            .into_iter()
            .find(|file| file.rel_path == Path::new("a.txt"))
            .expect("staged entry");
        assert_eq!(ignore_entry.status, EntryStatus::Unchanged);
        assert!(ignore_entry.hunks.is_empty());
        assert_eq!(ignore_entry.left_text, "fn main() {\n    let x = 1;\n}\n");
        assert_eq!(
            ignore_entry.right_text,
            "fn main() {\n\t let x = 1;   \n}\n"
        );

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_mode_can_compare_history_revision_to_working_tree() {
        let repo = unique_temp_dir("pontis-git-history");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("a.txt"), "v1\n").expect("write v1");
        git(&repo, &["add", "a.txt"]);
        git(&repo, &["commit", "-qm", "c1"]);
        fs::write(repo.join("a.txt"), "v2\n").expect("write v2");
        git(&repo, &["add", "a.txt"]);
        git(&repo, &["commit", "-qm", "c2"]);

        let (files, roots) = build_git_diff_files(
            &repo,
            &AppSettings::default(),
            &GitCompareMode::WorkingTree,
            "HEAD~1",
        )
        .expect("scan repo");
        let entry = files
            .into_iter()
            .find(|file| file.rel_path == Path::new("a.txt"))
            .expect("history entry");
        assert_eq!(entry.left_text, "v1\n");
        assert_eq!(entry.right_text, "v2\n");
        assert!(roots.left.ends_with(".git:HEAD~1"));

        fs::remove_dir_all(&repo).expect("cleanup");
    }

    #[test]
    fn git_mode_can_compare_two_revisions_directly() {
        let repo = unique_temp_dir("pontis-git-revision-pair");
        fs::create_dir_all(&repo).expect("mkdir repo");
        git(&repo, &["-c", "init.defaultBranch=main", "init", "-q"]);
        git(&repo, &["config", "user.name", "pontis"]);
        git(&repo, &["config", "user.email", "pontis@example.com"]);
        fs::write(repo.join("a.txt"), "line1\nline2\nline3\n").expect("write v1");
        git(&repo, &["add", "a.txt"]);
        git(&repo, &["commit", "-qm", "c1"]);
        fs::rename(repo.join("a.txt"), repo.join("b.txt")).expect("rename");
        fs::write(repo.join("b.txt"), "line1\nline2\nline3\n").expect("write v2");
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "c2"]);

        let mode = GitCompareMode::RevisionPair {
            left: "HEAD~1".to_string(),
            right: "HEAD".to_string(),
        };
        let (files, roots) =
            build_git_diff_files(&repo, &AppSettings::default(), &mode, "HEAD").expect("scan repo");
        let entry = files
            .into_iter()
            .find(|file| file.rel_path == Path::new("b.txt"))
            .expect("revision pair entry");
        assert_eq!(entry.status, EntryStatus::Renamed);
        assert_eq!(entry.left_text, "line1\nline2\nline3\n");
        assert_eq!(entry.right_text, "line1\nline2\nline3\n");
        assert_eq!(entry.original_rel_path.as_deref(), Some(Path::new("a.txt")));
        assert!(roots.left.ends_with(".git:HEAD~1"));
        assert_eq!(roots.right, repo);

        fs::remove_dir_all(&repo).expect("cleanup");
    }
}
