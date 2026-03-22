use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{EntryStatus, Mode};
use crate::settings::AppSettings;

use super::Cli;
use super::GitCommand;
use super::cli::Commands;
use super::reload_supported;
use super::targets::build_comparison_targets;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{uniq}"))
}

#[test]
fn file_system_mode_allows_writing_both_sides() {
    let root = unique_temp_dir("pontis-bootstrap-fs");
    let left = root.join("left");
    let right = root.join("right");
    std::fs::create_dir_all(&left).expect("mkdir left");
    std::fs::create_dir_all(&right).expect("mkdir right");
    std::fs::write(left.join("a.txt"), "x\n").expect("write left");
    std::fs::write(right.join("a.txt"), "y\n").expect("write right");

    let cli = Cli {
        left: Some(left.clone()),
        right: Some(right.clone()),
        config: None,
        command: None,
    };

    let (_files, roots, allow_left_write, allow_right_write) =
        build_comparison_targets(&cli, &AppSettings::default()).expect("targets");
    assert!(allow_left_write);
    assert!(allow_right_write);
    assert_eq!(roots.mode, Mode::Directory);

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn git_mode_makes_left_side_read_only() {
    let repo = unique_temp_dir("pontis-bootstrap-git");
    std::fs::create_dir_all(&repo).expect("mkdir repo");

    let status = Command::new("git")
        .arg("-c")
        .arg("init.defaultBranch=main")
        .arg("init")
        .arg("-q")
        .arg(&repo)
        .status()
        .expect("run git init");
    assert!(status.success());

    let cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(repo.clone()),
            staged: false,
            rev: None,
            diff: None,
            difftool_left_dir: None,
            difftool_right_dir: None,
        })),
    };

    let (_files, roots, allow_left_write, allow_right_write) =
        build_comparison_targets(&cli, &AppSettings::default()).expect("targets");
    assert!(!allow_left_write);
    assert!(allow_right_write);
    assert_eq!(roots.mode, Mode::Directory);
    assert_eq!(roots.left, repo);

    std::fs::remove_dir_all(roots.left).expect("cleanup");
}

#[test]
fn git_staged_mode_makes_both_sides_read_only() {
    let repo = unique_temp_dir("pontis-bootstrap-git-staged");
    std::fs::create_dir_all(&repo).expect("mkdir repo");

    let status = Command::new("git")
        .arg("-c")
        .arg("init.defaultBranch=main")
        .arg("init")
        .arg("-q")
        .arg(&repo)
        .status()
        .expect("run git init");
    assert!(status.success());

    let cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(repo.clone()),
            staged: true,
            rev: None,
            diff: None,
            difftool_left_dir: None,
            difftool_right_dir: None,
        })),
    };

    let (_files, roots, allow_left_write, allow_right_write) =
        build_comparison_targets(&cli, &AppSettings::default()).expect("targets");
    assert!(!allow_left_write);
    assert!(!allow_right_write);
    assert_eq!(roots.mode, Mode::Directory);
    assert_eq!(roots.left, repo);

    std::fs::remove_dir_all(roots.left).expect("cleanup");
}

#[test]
fn git_mode_with_explicit_revision_changes_left_root_label() {
    let repo = unique_temp_dir("pontis-bootstrap-git-rev");
    std::fs::create_dir_all(&repo).expect("mkdir repo");

    let status = Command::new("git")
        .arg("-c")
        .arg("init.defaultBranch=main")
        .arg("init")
        .arg("-q")
        .arg(&repo)
        .status()
        .expect("run git init");
    assert!(status.success());

    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("config")
        .arg("user.name")
        .arg("pontis")
        .status()
        .expect("git config user.name");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("config")
        .arg("user.email")
        .arg("pontis@example.com")
        .status()
        .expect("git config user.email");
    std::fs::write(repo.join("a.txt"), "v1\n").expect("write");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("add")
        .arg("a.txt")
        .status()
        .expect("git add");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("commit")
        .arg("-qm")
        .arg("init")
        .status()
        .expect("git commit");

    let cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(repo.clone()),
            staged: false,
            rev: Some("HEAD~0".to_string()),
            diff: None,
            difftool_left_dir: None,
            difftool_right_dir: None,
        })),
    };

    let (_files, roots, _allow_left_write, _allow_right_write) =
        build_comparison_targets(&cli, &AppSettings::default()).expect("targets");
    assert!(roots.left.ends_with(".git:HEAD~0"));

    std::fs::remove_dir_all(roots.right).expect("cleanup");
}

#[test]
fn git_diff_mode_makes_both_sides_read_only() {
    let repo = unique_temp_dir("pontis-bootstrap-git-diff");
    std::fs::create_dir_all(&repo).expect("mkdir repo");

    let status = Command::new("git")
        .arg("-c")
        .arg("init.defaultBranch=main")
        .arg("init")
        .arg("-q")
        .arg(&repo)
        .status()
        .expect("run git init");
    assert!(status.success());

    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("config")
        .arg("user.name")
        .arg("pontis")
        .status()
        .expect("git config user.name");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("config")
        .arg("user.email")
        .arg("pontis@example.com")
        .status()
        .expect("git config user.email");
    std::fs::write(repo.join("a.txt"), "v1\n").expect("write v1");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("add")
        .arg("a.txt")
        .status()
        .expect("git add");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("commit")
        .arg("-qm")
        .arg("init")
        .status()
        .expect("git commit");

    std::fs::write(repo.join("a.txt"), "v2\n").expect("write v2");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("commit")
        .arg("-am")
        .arg("second")
        .status()
        .expect("git commit second");

    let cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(repo.clone()),
            staged: false,
            rev: None,
            diff: Some(vec!["HEAD~1".to_string(), "HEAD".to_string()]),
            difftool_left_dir: None,
            difftool_right_dir: None,
        })),
    };

    let (_files, roots, allow_left_write, allow_right_write) =
        build_comparison_targets(&cli, &AppSettings::default()).expect("targets");
    assert!(!allow_left_write);
    assert!(!allow_right_write);
    assert_eq!(roots.mode, Mode::Directory);
    assert!(roots.left.ends_with(".git:HEAD~1"));
    assert_eq!(roots.right, repo);

    std::fs::remove_dir_all(repo).expect("cleanup");
}

#[test]
fn git_difftool_bridge_uses_temp_dirs_and_revision_labels() {
    let repo = unique_temp_dir("pontis-bootstrap-git-difftool");
    std::fs::create_dir_all(&repo).expect("mkdir repo");

    let status = Command::new("git")
        .arg("-c")
        .arg("init.defaultBranch=main")
        .arg("init")
        .arg("-q")
        .arg(&repo)
        .status()
        .expect("run git init");
    assert!(status.success());

    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("config")
        .arg("user.name")
        .arg("pontis")
        .status()
        .expect("git config user.name");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("config")
        .arg("user.email")
        .arg("pontis@example.com")
        .status()
        .expect("git config user.email");

    std::fs::write(repo.join("old.txt"), "same\n").expect("write old");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("add")
        .arg("old.txt")
        .status()
        .expect("git add");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("commit")
        .arg("-qm")
        .arg("init")
        .status()
        .expect("git commit");

    std::fs::rename(repo.join("old.txt"), repo.join("new.txt")).expect("rename");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("add")
        .arg("-A")
        .status()
        .expect("git add -A");
    Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("commit")
        .arg("-qm")
        .arg("rename")
        .status()
        .expect("git commit rename");

    let left_dir = unique_temp_dir("pontis-difftool-left");
    let right_dir = unique_temp_dir("pontis-difftool-right");
    std::fs::create_dir_all(&left_dir).expect("mkdir left temp");
    std::fs::create_dir_all(&right_dir).expect("mkdir right temp");
    std::fs::write(left_dir.join("old.txt"), "same\n").expect("write left temp");
    std::fs::write(right_dir.join("new.txt"), "same\n").expect("write right temp");

    let cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(repo.clone()),
            staged: false,
            rev: None,
            diff: Some(vec!["HEAD~1".to_string(), "HEAD".to_string()]),
            difftool_left_dir: Some(left_dir.clone()),
            difftool_right_dir: Some(right_dir.clone()),
        })),
    };

    let (files, roots, allow_left_write, allow_right_write) =
        build_comparison_targets(&cli, &AppSettings::default()).expect("targets");
    assert!(!allow_left_write);
    assert!(!allow_right_write);
    assert_eq!(roots.left_label.as_deref(), Some("HEAD~1"));
    assert_eq!(roots.right_label.as_deref(), Some("HEAD"));
    let entry = files
        .into_iter()
        .find(|file| file.rel_path == Path::new("new.txt"))
        .expect("rename entry");
    assert_eq!(entry.status, EntryStatus::Renamed);
    assert_eq!(
        entry.original_rel_path.as_deref(),
        Some(Path::new("old.txt"))
    );

    std::fs::remove_dir_all(repo).expect("cleanup repo");
    std::fs::remove_dir_all(left_dir).expect("cleanup left");
    std::fs::remove_dir_all(right_dir).expect("cleanup right");
}

#[test]
fn reload_is_only_supported_for_mutable_compare_modes() {
    let fs_cli = Cli {
        left: Some(PathBuf::from("/tmp/left")),
        right: Some(PathBuf::from("/tmp/right")),
        config: None,
        command: None,
    };
    assert!(reload_supported(&fs_cli));

    let git_worktree_cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(PathBuf::from("/tmp/repo")),
            staged: false,
            rev: Some("HEAD".to_string()),
            diff: None,
            difftool_left_dir: None,
            difftool_right_dir: None,
        })),
    };
    assert!(reload_supported(&git_worktree_cli));

    let staged_cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(PathBuf::from("/tmp/repo")),
            staged: true,
            rev: None,
            diff: None,
            difftool_left_dir: None,
            difftool_right_dir: None,
        })),
    };
    assert!(!reload_supported(&staged_cli));

    let revision_pair_cli = Cli {
        left: None,
        right: None,
        config: None,
        command: Some(Commands::Git(GitCommand {
            repo: Some(PathBuf::from("/tmp/repo")),
            staged: false,
            rev: None,
            diff: Some(vec!["HEAD~1".to_string(), "HEAD".to_string()]),
            difftool_left_dir: None,
            difftool_right_dir: None,
        })),
    };
    assert!(!reload_supported(&revision_pair_cli));
}
