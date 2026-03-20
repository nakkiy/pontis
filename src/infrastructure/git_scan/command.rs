use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

const EMPTY_TREE_HASH: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

#[derive(Debug, Clone)]
pub(crate) struct GitStatusEntry {
    pub(crate) path: PathBuf,
    pub(crate) original_path: Option<PathBuf>,
    pub(crate) change: GitEntryChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitEntryChange {
    None,
    Renamed,
}

pub(crate) fn git_resolve_compare_rev(repo: &Path, rev: &str) -> Result<String> {
    if git_revision_exists(repo, rev)? {
        return Ok(rev.to_string());
    }
    if rev == "HEAD" {
        return Ok(EMPTY_TREE_HASH.to_string());
    }
    bail!("unknown git revision: {rev}");
}

pub(super) fn git_diff_entries(
    repo: &Path,
    rev: &str,
    staged: bool,
) -> Result<Vec<GitStatusEntry>> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .arg("diff")
        .arg("--name-status")
        .arg("-z")
        .arg("-M");
    if staged {
        command.arg("--cached");
    }
    command.arg(rev);

    let output = command
        .output()
        .with_context(|| format!("failed to run git diff in {}", repo.display()))?;

    if !output.status.success() {
        bail!(
            "git diff failed in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(parse_name_status_z_records(&output.stdout))
}

pub(crate) fn git_diff_entries_between(
    repo: &Path,
    left_rev: &str,
    right_rev: &str,
) -> Result<Vec<GitStatusEntry>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("diff")
        .arg("--name-status")
        .arg("-z")
        .arg("-M")
        .arg(left_rev)
        .arg(right_rev)
        .output()
        .with_context(|| format!("failed to run git diff in {}", repo.display()))?;

    if !output.status.success() {
        bail!(
            "git diff failed in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(parse_name_status_z_records(&output.stdout))
}

pub(super) fn git_untracked_entries(repo: &Path) -> Result<Vec<GitStatusEntry>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("ls-files")
        .arg("--others")
        .arg("--exclude-standard")
        .arg("-z")
        .output()
        .with_context(|| format!("failed to run git ls-files in {}", repo.display()))?;

    if !output.status.success() {
        bail!(
            "git ls-files failed in {}: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output
        .stdout
        .split(|b| *b == 0)
        .filter(|part| !part.is_empty())
        .map(|part| GitStatusEntry {
            path: PathBuf::from(String::from_utf8_lossy(part).to_string()),
            original_path: None,
            change: GitEntryChange::None,
        })
        .collect())
}

pub(super) fn git_show_rev(repo: &Path, rev: &str, rel: &Path) -> Result<Vec<u8>> {
    let spec = format!("{rev}:{}", rel.to_string_lossy());
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("show")
        .arg(spec)
        .output()
        .with_context(|| format!("failed to run git show in {}", repo.display()))?;

    if !output.status.success() {
        bail!(
            "git show failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.stdout)
}

pub(super) fn git_show_rev_if_exists(
    repo: &Path,
    rev: &str,
    rel: &Path,
) -> Result<Option<Vec<u8>>> {
    if !git_rev_path_exists(repo, rev, rel)? {
        return Ok(None);
    }
    git_show_rev(repo, rev, rel).map(Some)
}

pub(super) fn git_show_index(repo: &Path, rel: &Path) -> Result<Vec<u8>> {
    let spec = format!(":{}", rel.to_string_lossy());
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("show")
        .arg(spec)
        .output()
        .with_context(|| format!("failed to run git show in {}", repo.display()))?;

    if !output.status.success() {
        bail!(
            "git show failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.stdout)
}

pub(super) fn git_show_index_if_exists(repo: &Path, rel: &Path) -> Result<Option<Vec<u8>>> {
    if !git_index_path_exists(repo, rel)? {
        return Ok(None);
    }
    git_show_index(repo, rel).map(Some)
}

pub(crate) fn git_toplevel(path: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .with_context(|| format!("failed to run git rev-parse in {}", path.display()))?;

    if !output.status.success() {
        bail!(
            "not a git repository: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

fn parse_name_status_z_records(stdout: &[u8]) -> Vec<GitStatusEntry> {
    let records = stdout.split(|b| *b == 0).collect::<Vec<_>>();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < records.len() {
        let status_raw = records[i];
        i += 1;
        if status_raw.is_empty() {
            continue;
        }

        let status = String::from_utf8_lossy(status_raw);
        let kind = status.chars().next().unwrap_or(' ');

        let change = match kind {
            'R' => GitEntryChange::Renamed,
            _ => GitEntryChange::None,
        };

        let (path, original_path) = if matches!(change, GitEntryChange::Renamed) {
            let from = records.get(i).copied().unwrap_or_default();
            i += 1;
            let to = records.get(i).copied().unwrap_or_default();
            i += 1;
            if to.is_empty() {
                continue;
            }
            let path = PathBuf::from(String::from_utf8_lossy(to).to_string());
            let original_path = if from.is_empty() {
                None
            } else {
                Some(PathBuf::from(String::from_utf8_lossy(from).to_string()))
            };
            (path, original_path)
        } else {
            let Some(path_rec) = records.get(i).copied() else {
                break;
            };
            i += 1;
            if path_rec.is_empty() {
                continue;
            }
            (
                PathBuf::from(String::from_utf8_lossy(path_rec).to_string()),
                None,
            )
        };

        out.push(GitStatusEntry {
            path,
            original_path,
            change,
        });
    }
    out
}

fn git_rev_path_exists(repo: &Path, rev: &str, rel: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("ls-tree")
        .arg("--name-only")
        .arg(rev)
        .arg("--")
        .arg(rel)
        .output()
        .with_context(|| format!("failed to run git ls-tree in {}", repo.display()))?;

    if !output.status.success() {
        bail!(
            "git ls-tree failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(!output.stdout.is_empty())
}

fn git_revision_exists(repo: &Path, rev: &str) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("rev-parse")
        .arg("--verify")
        .arg("--quiet")
        .arg(rev)
        .output()
        .with_context(|| format!("failed to run git rev-parse in {}", repo.display()))?;

    if output.status.success() {
        return Ok(true);
    }
    if output.status.code() == Some(1) {
        return Ok(false);
    }
    bail!(
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_index_path_exists(repo: &Path, rel: &Path) -> Result<bool> {
    let spec = format!(":{}", rel.to_string_lossy());
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("cat-file")
        .arg("-e")
        .arg(spec)
        .output()
        .with_context(|| format!("failed to run git cat-file in {}", repo.display()))?;

    if output.status.success() {
        return Ok(true);
    }
    if output.status.code() == Some(128) {
        return Ok(false);
    }
    bail!(
        "git cat-file failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{GitEntryChange, parse_name_status_z_records};

    #[test]
    fn parse_regular_and_rename_records() {
        let mut raw = Vec::new();
        raw.extend_from_slice(b"M");
        raw.push(0);
        raw.extend_from_slice(b"a.txt");
        raw.push(0);
        raw.extend_from_slice(b"R100");
        raw.push(0);
        raw.extend_from_slice(b"old.txt");
        raw.push(0);
        raw.extend_from_slice(b"new.txt");
        raw.push(0);

        let entries = parse_name_status_z_records(&raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, PathBuf::from("a.txt"));
        assert_eq!(entries[1].path, PathBuf::from("new.txt"));
        assert_eq!(entries[1].original_path, Some(PathBuf::from("old.txt")));
        assert_eq!(entries[1].change, GitEntryChange::Renamed);
    }
}
