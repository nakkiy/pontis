use anyhow::{Result, bail};

use crate::model::{DiffFile, Roots};
use crate::settings::AppSettings;
use crate::{fs_scan, git_scan};

use super::Cli;
use super::GitCommand;
use super::cli::Commands;

pub(super) fn build_comparison_targets(
    cli: &Cli,
    cfg: &AppSettings,
) -> Result<(Vec<DiffFile>, Roots, bool, bool)> {
    if let Some(Commands::Git(GitCommand {
        repo,
        staged,
        rev,
        diff,
        difftool_left_dir,
        difftool_right_dir,
    })) = &cli.command
    {
        let repo = repo
            .clone()
            .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
        if let Some(revs) = diff {
            let left_rev = &revs[0];
            let right_rev = &revs[1];
            if let (Some(left_dir), Some(right_dir)) =
                (difftool_left_dir.as_ref(), difftool_right_dir.as_ref())
            {
                let (repo_root, _resolved_left_rev, _resolved_right_rev, renames) =
                    git_scan::collect_revision_pair_renames(&repo, left_rev, right_rev)?;
                let (files, mut roots) = fs_scan::build_diff_files_with_precomputed_renames(
                    left_dir, right_dir, cfg, &renames,
                )?;
                roots.left = repo_root.join(format!(".git:{left_rev}"));
                roots.right = if right_rev == "HEAD" {
                    repo_root
                } else {
                    repo_root.join(format!(".git:{right_rev}"))
                };
                roots.left_label = Some(left_rev.clone());
                roots.right_label = Some(right_rev.clone());
                return Ok((files, roots, false, false));
            }

            let mode = git_scan::GitCompareMode::RevisionPair {
                left: left_rev.clone(),
                right: right_rev.clone(),
            };
            let (files, roots) = git_scan::build_git_diff_files(&repo, cfg, &mode, left_rev)?;
            return Ok((files, roots, false, false));
        }

        let mode = if *staged {
            git_scan::GitCompareMode::Staged
        } else {
            git_scan::GitCompareMode::WorkingTree
        };
        let left_rev = rev.as_deref().unwrap_or("HEAD");
        let (files, roots) = git_scan::build_git_diff_files(&repo, cfg, &mode, left_rev)?;
        let allow_right_write = mode == git_scan::GitCompareMode::WorkingTree;
        return Ok((files, roots, false, allow_right_write));
    }

    let Some(left) = cli.left.as_ref() else {
        bail!("left path is required unless git subcommand is specified");
    };
    let Some(right) = cli.right.as_ref() else {
        bail!("right path is required unless git subcommand is specified");
    };
    let (files, roots) = fs_scan::build_diff_files_with_config(left, right, cfg)?;
    Ok((files, roots, true, true))
}
