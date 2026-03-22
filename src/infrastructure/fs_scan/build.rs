use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use similar::TextDiff;
use walkdir::WalkDir;

use crate::model::{DiffFile, EntryStatus, Mode, Roots};
use crate::settings::AppSettings;

use super::load::load_diff_file_with_config;
use super::reader::{FileContent, read_file_content};

const RENAME_TEXT_RATIO_THRESHOLD: f32 = 0.75;
const RENAME_MAX_SIDE_CANDIDATES: usize = 64;
const RENAME_MAX_PAIR_EVALUATIONS: usize = 1024;
const RENAME_MAX_BASENAME_CANDIDATES: usize = 4;
const RENAME_MAX_FALLBACK_CANDIDATES: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecomputedRename {
    pub old_rel: PathBuf,
    pub new_rel: PathBuf,
}

pub fn build_diff_files(left: &Path, right: &Path) -> Result<(Vec<DiffFile>, Roots)> {
    build_diff_files_with_config(left, right, &AppSettings::default())
}

pub fn build_diff_files_with_config(
    left: &Path,
    right: &Path,
    cfg: &AppSettings,
) -> Result<(Vec<DiffFile>, Roots)> {
    build_diff_files_with_precomputed_renames(left, right, cfg, &[])
}

pub fn build_diff_files_with_precomputed_renames(
    left: &Path,
    right: &Path,
    cfg: &AppSettings,
    precomputed_renames: &[PrecomputedRename],
) -> Result<(Vec<DiffFile>, Roots)> {
    let left_meta =
        fs::metadata(left).with_context(|| format!("cannot read metadata: {}", left.display()))?;
    let right_meta = fs::metadata(right)
        .with_context(|| format!("cannot read metadata: {}", right.display()))?;

    if left_meta.is_file() && right_meta.is_file() {
        let left_size = left_meta.len() as usize;
        let right_size = right_meta.len() as usize;
        let rel = left
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("(file)"));
        let status = EntryStatus::Modified;

        let mut file = DiffFile::new_unloaded(
            rel,
            Some(left.to_path_buf()),
            Some(right.to_path_buf()),
            left_size,
            right_size,
            left_size.max(right_size) > cfg.highlight.max_bytes,
            status,
        );
        load_diff_file_with_config(&mut file, cfg)?;

        return Ok((
            vec![file],
            Roots {
                left: left.to_path_buf(),
                right: right.to_path_buf(),
                mode: Mode::File,
                left_label: None,
                right_label: None,
            },
        ));
    }

    if left_meta.is_dir() && right_meta.is_dir() {
        let left_map = collect_files(left)?;
        let right_map = collect_files(right)?;
        let rename_pairs = if precomputed_renames.is_empty() {
            detect_renamed_pairs(&left_map, &right_map)?
        } else {
            materialize_precomputed_renames(&left_map, &right_map, precomputed_renames)
        };

        let mut all_keys: BTreeSet<PathBuf> = BTreeSet::new();
        all_keys.extend(left_map.keys().cloned());
        all_keys.extend(right_map.keys().cloned());

        let mut files = Vec::with_capacity(all_keys.len().saturating_sub(rename_pairs.len()));
        let renamed_new_paths = rename_pairs
            .values()
            .map(|renamed| renamed.new_rel.clone())
            .collect::<BTreeSet<_>>();
        let renamed_old_paths = rename_pairs.keys().cloned().collect::<BTreeSet<_>>();

        for rel in all_keys {
            if renamed_old_paths.contains(&rel) || renamed_new_paths.contains(&rel) {
                continue;
            }
            let left_abs = left_map.get(&rel).cloned();
            let right_abs = right_map.get(&rel).cloned();
            let status = derive_initial_status(left_abs.is_some(), right_abs.is_some());
            files.push(build_unloaded_file(rel, left_abs, right_abs, status));
        }

        for renamed in rename_pairs.into_values() {
            let mut file = build_unloaded_file(
                renamed.new_rel.clone(),
                Some(renamed.left_abs),
                Some(renamed.right_abs),
                EntryStatus::Renamed,
            );
            file.set_original_rel_path(renamed.old_rel);
            files.push(file);
        }

        files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        if let Some(first) = files.first_mut() {
            load_diff_file_with_config(first, cfg)?;
        }

        return Ok((
            files,
            Roots {
                left: left.to_path_buf(),
                right: right.to_path_buf(),
                mode: Mode::Directory,
                left_label: None,
                right_label: None,
            },
        ));
    }

    bail!("unsupported input pair: file/dir mixed input is not supported");
}

fn materialize_precomputed_renames(
    left_map: &HashMap<PathBuf, PathBuf>,
    right_map: &HashMap<PathBuf, PathBuf>,
    precomputed_renames: &[PrecomputedRename],
) -> HashMap<PathBuf, RenamePair> {
    let mut out = HashMap::new();
    for rename in precomputed_renames {
        let Some(left_abs) = left_map.get(&rename.old_rel) else {
            continue;
        };
        let Some(right_abs) = right_map.get(&rename.new_rel) else {
            continue;
        };
        out.insert(
            rename.old_rel.clone(),
            RenamePair {
                old_rel: rename.old_rel.clone(),
                new_rel: rename.new_rel.clone(),
                left_abs: left_abs.clone(),
                right_abs: right_abs.clone(),
            },
        );
    }
    out
}

fn collect_files(root: &Path) -> Result<HashMap<PathBuf, PathBuf>> {
    let mut map = HashMap::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let is_file_like = entry.file_type().is_file()
            || (entry.file_type().is_symlink()
                && fs::metadata(path)
                    .map(|meta| meta.is_file())
                    .unwrap_or(false));
        if !is_file_like {
            continue;
        }
        let abs = entry.into_path();
        let rel = abs
            .strip_prefix(root)
            .with_context(|| format!("failed to get relative path for {}", abs.display()))?
            .to_path_buf();
        map.insert(rel, abs);
    }
    Ok(map)
}

fn derive_initial_status(left_exists: bool, right_exists: bool) -> EntryStatus {
    match (left_exists, right_exists) {
        (true, true) => EntryStatus::Pending,
        (true, false) => EntryStatus::Deleted,
        (false, true) => EntryStatus::Added,
        (false, false) => EntryStatus::Unchanged,
    }
}

fn build_unloaded_file(
    rel_path: PathBuf,
    left_path: Option<PathBuf>,
    right_path: Option<PathBuf>,
    status: EntryStatus,
) -> DiffFile {
    DiffFile::new_unloaded(rel_path, left_path, right_path, 0, 0, false, status)
}

#[derive(Debug, Clone)]
struct RenamePair {
    old_rel: PathBuf,
    new_rel: PathBuf,
    left_abs: PathBuf,
    right_abs: PathBuf,
}

#[derive(Debug, Clone)]
struct RenameProbe {
    rel: PathBuf,
    abs: PathBuf,
    bytes: u64,
}

fn detect_renamed_pairs(
    left_map: &HashMap<PathBuf, PathBuf>,
    right_map: &HashMap<PathBuf, PathBuf>,
) -> Result<HashMap<PathBuf, RenamePair>> {
    let deleted = left_map
        .iter()
        .filter(|(rel, _)| !right_map.contains_key(*rel))
        .map(|(rel, abs)| build_probe(rel, abs))
        .collect::<Result<Vec<_>>>()?;
    let added = right_map
        .iter()
        .filter(|(rel, _)| !left_map.contains_key(*rel))
        .map(|(rel, abs)| build_probe(rel, abs))
        .collect::<Result<Vec<_>>>()?;

    if deleted.is_empty()
        || added.is_empty()
        || deleted.len() > RENAME_MAX_SIDE_CANDIDATES
        || added.len() > RENAME_MAX_SIDE_CANDIDATES
        || deleted.len().saturating_mul(added.len()) > RENAME_MAX_PAIR_EVALUATIONS
    {
        return Ok(HashMap::new());
    }

    let mut matched_left = vec![false; deleted.len()];
    let mut matched_right = vec![false; added.len()];
    let mut deleted_content = vec![None; deleted.len()];
    let mut added_content = vec![None; added.len()];
    let mut out = HashMap::new();
    let deleted_is_primary = deleted.len() <= added.len();

    for basename_only in [true, false] {
        if deleted_is_primary {
            for left_idx in 0..deleted.len() {
                if matched_left[left_idx] {
                    continue;
                }
                let Some(right_idx) = find_best_match(
                    left_idx,
                    &deleted,
                    &mut deleted_content,
                    &added,
                    &mut added_content,
                    &matched_right,
                    basename_only,
                )?
                else {
                    continue;
                };
                matched_left[left_idx] = true;
                matched_right[right_idx] = true;
                let left = &deleted[left_idx];
                let right = &added[right_idx];
                out.insert(
                    left.rel.clone(),
                    RenamePair {
                        old_rel: left.rel.clone(),
                        new_rel: right.rel.clone(),
                        left_abs: left.abs.clone(),
                        right_abs: right.abs.clone(),
                    },
                );
            }
        } else {
            for right_idx in 0..added.len() {
                if matched_right[right_idx] {
                    continue;
                }
                let Some(left_idx) = find_best_match(
                    right_idx,
                    &added,
                    &mut added_content,
                    &deleted,
                    &mut deleted_content,
                    &matched_left,
                    basename_only,
                )?
                else {
                    continue;
                };
                matched_left[left_idx] = true;
                matched_right[right_idx] = true;
                let left = &deleted[left_idx];
                let right = &added[right_idx];
                out.insert(
                    left.rel.clone(),
                    RenamePair {
                        old_rel: left.rel.clone(),
                        new_rel: right.rel.clone(),
                        left_abs: left.abs.clone(),
                        right_abs: right.abs.clone(),
                    },
                );
            }
        }
    }

    Ok(out)
}

fn find_best_match(
    primary_idx: usize,
    primary: &[RenameProbe],
    primary_content: &mut [Option<FileContent>],
    secondary: &[RenameProbe],
    secondary_content: &mut [Option<FileContent>],
    matched_secondary: &[bool],
    basename_only: bool,
) -> Result<Option<usize>> {
    let primary_probe = &primary[primary_idx];
    let mut candidates = secondary
        .iter()
        .enumerate()
        .filter(|(secondary_idx, secondary_probe)| {
            !matched_secondary[*secondary_idx]
                && rename_candidates_are_comparable(primary_probe, secondary_probe)
                && (!basename_only || same_file_name(primary_probe, secondary_probe))
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|(_, a), (_, b)| compare_candidate_priority(primary_probe, a, b));
    let candidate_limit = if basename_only {
        RENAME_MAX_BASENAME_CANDIDATES
    } else {
        RENAME_MAX_FALLBACK_CANDIDATES
    };
    candidates.truncate(candidate_limit);

    let primary_file_content = ensure_loaded_content(primary_idx, primary, primary_content)?;

    let mut best_match = None;
    let mut best_score = RENAME_TEXT_RATIO_THRESHOLD;
    for (secondary_idx, secondary_probe) in candidates {
        let secondary_file_content =
            ensure_loaded_content(secondary_idx, secondary, secondary_content)?;
        let Some(score) = similarity_score(primary_file_content, secondary_file_content) else {
            continue;
        };
        if score < best_score {
            continue;
        }
        best_score = score;
        best_match = Some(secondary_idx);
        if score >= 0.999 || (same_file_name(primary_probe, secondary_probe) && score >= 0.9) {
            break;
        }
    }

    Ok(best_match)
}

fn ensure_loaded_content<'a>(
    idx: usize,
    probes: &[RenameProbe],
    cache: &'a mut [Option<FileContent>],
) -> Result<&'a FileContent> {
    if cache[idx].is_none() {
        cache[idx] = Some(read_file_content(&probes[idx].abs)?);
    }
    Ok(cache[idx].as_ref().expect("content cached"))
}

fn build_probe(rel: &Path, abs: &Path) -> Result<RenameProbe> {
    let bytes = fs::metadata(abs)
        .with_context(|| format!("cannot read metadata: {}", abs.display()))?
        .len();
    Ok(RenameProbe {
        rel: rel.to_path_buf(),
        abs: abs.to_path_buf(),
        bytes,
    })
}

fn rename_candidates_are_comparable(left: &RenameProbe, right: &RenameProbe) -> bool {
    let left_ext = left.rel.extension();
    let right_ext = right.rel.extension();
    if left_ext.is_some() && right_ext.is_some() && left_ext != right_ext {
        return false;
    }

    let max_bytes = left.bytes.max(right.bytes);
    let min_bytes = left.bytes.min(right.bytes);
    min_bytes == 0 || max_bytes <= min_bytes.saturating_mul(4)
}

fn same_file_name(left: &RenameProbe, right: &RenameProbe) -> bool {
    left.rel.file_name() == right.rel.file_name()
}

fn compare_candidate_priority(
    left: &RenameProbe,
    a: &RenameProbe,
    b: &RenameProbe,
) -> std::cmp::Ordering {
    same_file_name(left, b)
        .cmp(&same_file_name(left, a))
        .then_with(|| common_suffix_segments(left, b).cmp(&common_suffix_segments(left, a)))
        .then_with(|| byte_distance(left, a).cmp(&byte_distance(left, b)))
        .then_with(|| a.rel.cmp(&b.rel))
}

fn common_suffix_segments(left: &RenameProbe, right: &RenameProbe) -> usize {
    let left_segments = path_segments(&left.rel);
    let right_segments = path_segments(&right.rel);
    let mut count = 0usize;
    while count < left_segments.len().min(right_segments.len())
        && left_segments[left_segments.len() - 1 - count]
            == right_segments[right_segments.len() - 1 - count]
    {
        count += 1;
    }
    count
}

fn path_segments(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect()
}

fn byte_distance(left: &RenameProbe, right: &RenameProbe) -> u64 {
    left.bytes.abs_diff(right.bytes)
}

fn similarity_score(left: &FileContent, right: &FileContent) -> Option<f32> {
    match (left.is_binary, right.is_binary) {
        (true, true) => (left.raw == right.raw).then_some(1.0),
        (false, false) => Some(TextDiff::from_chars(&left.text, &right.text).ratio()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        PrecomputedRename, build_diff_files_with_precomputed_renames, collect_files,
        derive_initial_status, detect_renamed_pairs,
    };
    use crate::model::EntryStatus;
    use crate::settings::AppSettings;

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{uniq}"))
    }

    #[test]
    fn derive_initial_status_covers_presence_matrix() {
        assert_eq!(derive_initial_status(true, true), EntryStatus::Pending);
        assert_eq!(derive_initial_status(true, false), EntryStatus::Deleted);
        assert_eq!(derive_initial_status(false, true), EntryStatus::Added);
        assert_eq!(derive_initial_status(false, false), EntryStatus::Unchanged);
    }

    #[cfg(unix)]
    #[test]
    fn collect_files_includes_symlinked_files() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("pontis-fs-symlink");
        let target_dir = unique_temp_dir("pontis-fs-symlink-target");
        fs::create_dir_all(&root).expect("mkdir root");
        fs::create_dir_all(&target_dir).expect("mkdir target");
        fs::write(target_dir.join("target.txt"), "hello\n").expect("write target");
        fs::create_dir_all(root.join("nested")).expect("mkdir nested");
        symlink(
            target_dir.join("target.txt"),
            root.join("nested").join("link.txt"),
        )
        .expect("symlink");

        let map = collect_files(&root).expect("collect");
        assert!(map.contains_key(std::path::Path::new("nested/link.txt")));

        fs::remove_dir_all(&root).expect("cleanup root");
        fs::remove_dir_all(&target_dir).expect("cleanup target");
    }

    #[test]
    fn detect_renamed_pairs_matches_lightly_edited_files() {
        let left = unique_temp_dir("pontis-fs-left");
        let right = unique_temp_dir("pontis-fs-right");
        fs::create_dir_all(left.join("src")).expect("mkdir left");
        fs::create_dir_all(right.join("src/moved")).expect("mkdir right");
        fs::write(left.join("src/old.txt"), "alpha\nbeta\ngamma\n").expect("write left");
        fs::write(
            right.join("src/moved/new.txt"),
            "alpha\nbeta updated\ngamma\n",
        )
        .expect("write right");

        let mut left_map = HashMap::new();
        left_map.insert(
            Path::new("src/old.txt").to_path_buf(),
            left.join("src/old.txt"),
        );
        let mut right_map = HashMap::new();
        right_map.insert(
            Path::new("src/moved/new.txt").to_path_buf(),
            right.join("src/moved/new.txt"),
        );

        let renamed = detect_renamed_pairs(&left_map, &right_map).expect("detect renamed");
        let pair = renamed
            .get(Path::new("src/old.txt"))
            .expect("renamed pair present");
        assert_eq!(pair.new_rel, Path::new("src/moved/new.txt"));

        fs::remove_dir_all(&left).expect("cleanup left");
        fs::remove_dir_all(&right).expect("cleanup right");
    }

    #[test]
    fn precomputed_renames_replace_added_deleted_pair() {
        let root = unique_temp_dir("pontis-fs-precomputed-rename");
        let left = root.join("left");
        let right = root.join("right");
        fs::create_dir_all(left.join("src/app")).expect("mkdir left");
        fs::create_dir_all(right.join("src/app")).expect("mkdir right");
        fs::write(left.join("src/app/old.rs"), "fn main() {}\n").expect("write left");
        fs::write(right.join("src/app/new.rs"), "fn main() {}\n").expect("write right");

        let (files, _) = build_diff_files_with_precomputed_renames(
            &left,
            &right,
            &AppSettings::default(),
            &[PrecomputedRename {
                old_rel: Path::new("src/app/old.rs").to_path_buf(),
                new_rel: Path::new("src/app/new.rs").to_path_buf(),
            }],
        )
        .expect("build precomputed");

        assert_eq!(files.len(), 1);
        let renamed = &files[0];
        assert_eq!(renamed.status, EntryStatus::Renamed);
        assert_eq!(renamed.rel_path, Path::new("src/app/new.rs"));
        assert_eq!(
            renamed.original_rel_path.as_deref(),
            Some(Path::new("src/app/old.rs"))
        );

        fs::remove_dir_all(root).expect("cleanup");
    }
}
