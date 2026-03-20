use pontis::diff::{DiffComparePolicies, LineEndingPolicy, WhitespacePolicy};
use pontis::fs_scan::{build_diff_files, build_diff_files_with_config};
use pontis::model::{EntryStatus, Mode};
use pontis::settings::{AppSettings, DEFAULT_HIGHLIGHT_MAX_BYTES};

mod support;

#[test]
fn file_mode_classifies_unchanged_and_modified() {
    let root = support::unique_temp_dir("pontis-file-mode");
    let left = root.join("left.txt");
    let right = root.join("right.txt");

    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&left, "same\n").expect("write left");
    std::fs::write(&right, "same\n").expect("write right");

    let (same_files, same_roots) =
        build_diff_files(left.as_path(), right.as_path()).expect("scan same");
    assert_eq!(same_roots.mode, Mode::File);
    assert_eq!(same_files.len(), 1);
    assert_eq!(same_files[0].status, EntryStatus::Unchanged);
    assert!(same_files[0].hunks.is_empty());

    std::fs::write(&right, "diff\n").expect("overwrite right");
    let (diff_files, diff_roots) =
        build_diff_files(left.as_path(), right.as_path()).expect("scan diff");
    assert_eq!(diff_roots.mode, Mode::File);
    assert_eq!(diff_files.len(), 1);
    assert_eq!(diff_files[0].status, EntryStatus::Modified);
    assert!(!diff_files[0].hunks.is_empty());

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn file_mode_binary_is_marked_and_hunks_are_disabled() {
    let root = support::unique_temp_dir("pontis-file-binary");
    let left = root.join("left.bin");
    let right = root.join("right.bin");

    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&left, [0, 1, 2, 3]).expect("write left");
    std::fs::write(&right, [0, 1, 2, 9]).expect("write right");

    let (files, roots) = build_diff_files(left.as_path(), right.as_path()).expect("scan binary");
    assert_eq!(roots.mode, Mode::File);
    assert_eq!(files.len(), 1);
    assert!(files[0].is_binary);
    assert_eq!(files[0].status, EntryStatus::Modified);
    assert!(files[0].hunks.is_empty());

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn file_mode_equal_binary_is_marked_unchanged() {
    let root = support::unique_temp_dir("pontis-file-binary-same");
    let left = root.join("left.bin");
    let right = root.join("right.bin");

    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&left, [0, 1, 2, 3]).expect("write left");
    std::fs::write(&right, [0, 1, 2, 3]).expect("write right");

    let (files, roots) = build_diff_files(left.as_path(), right.as_path()).expect("scan binary");
    assert_eq!(roots.mode, Mode::File);
    assert_eq!(files.len(), 1);
    assert!(files[0].is_binary);
    assert_eq!(files[0].status, EntryStatus::Unchanged);
    assert!(files[0].hunks.is_empty());

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn large_file_disables_syntax_highlight() {
    let root = support::unique_temp_dir("pontis-file-large");
    let left = root.join("left.txt");
    let right = root.join("right.txt");

    std::fs::create_dir_all(&root).expect("mkdir");
    let large = "a".repeat(DEFAULT_HIGHLIGHT_MAX_BYTES + 1024);
    std::fs::write(&left, &large).expect("write left");
    std::fs::write(&right, &large).expect("write right");

    let (files, _) = build_diff_files(left.as_path(), right.as_path()).expect("scan large");
    assert_eq!(files.len(), 1);
    assert!(files[0].highlight_limited);
    assert!(files[0].should_use_plain_render());

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn file_mode_bom_only_difference_is_unchanged() {
    let root = support::unique_temp_dir("pontis-file-bom");
    let left = root.join("left.txt");
    let right = root.join("right.txt");

    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&left, b"\xEF\xBB\xBFhello\n").expect("write left");
    std::fs::write(&right, b"hello\n").expect("write right");

    let (files, roots) = build_diff_files(left.as_path(), right.as_path()).expect("scan bom");
    assert_eq!(roots.mode, Mode::File);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].status, EntryStatus::Unchanged);
    assert!(files[0].hunks.is_empty());
    assert!(files[0].left_has_utf8_bom);
    assert!(!files[0].right_has_utf8_bom);

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn file_mode_can_ignore_whitespace_only_changes() {
    let root = support::unique_temp_dir("pontis-file-whitespace");
    let left = root.join("left.txt");
    let right = root.join("right.txt");

    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&left, "fn main() {\n    let x = 1;\n}\n").expect("write left");
    std::fs::write(&right, "fn main() {\n\t let x = 1;   \n}\n").expect("write right");

    let (strict_files, strict_roots) =
        build_diff_files(left.as_path(), right.as_path()).expect("scan strict");
    assert_eq!(strict_roots.mode, Mode::File);
    assert_eq!(strict_files.len(), 1);
    assert_eq!(strict_files[0].status, EntryStatus::Modified);
    assert!(!strict_files[0].hunks.is_empty());

    let cfg = AppSettings {
        compare_policies: DiffComparePolicies::with_whitespace(WhitespacePolicy::Ignore),
        ..AppSettings::default()
    };
    let (ignore_files, ignore_roots) =
        build_diff_files_with_config(left.as_path(), right.as_path(), &cfg).expect("scan ignore");
    assert_eq!(ignore_roots.mode, Mode::File);
    assert_eq!(ignore_files.len(), 1);
    assert_eq!(ignore_files[0].status, EntryStatus::Unchanged);
    assert!(ignore_files[0].hunks.is_empty());

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn file_mode_can_ignore_line_ending_only_changes() {
    let root = support::unique_temp_dir("pontis-file-line-endings");
    let left = root.join("left.txt");
    let right = root.join("right.txt");

    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&left, "a\r\nb\r\n").expect("write left");
    std::fs::write(&right, "a\nb\n").expect("write right");

    let (strict_files, strict_roots) =
        build_diff_files(left.as_path(), right.as_path()).expect("scan strict");
    assert_eq!(strict_roots.mode, Mode::File);
    assert_eq!(strict_files.len(), 1);
    assert_eq!(strict_files[0].status, EntryStatus::Modified);
    assert!(!strict_files[0].hunks.is_empty());

    let cfg = AppSettings {
        compare_policies: DiffComparePolicies::new(
            WhitespacePolicy::Compare,
            LineEndingPolicy::Ignore,
        ),
        ..AppSettings::default()
    };
    let (ignore_files, ignore_roots) =
        build_diff_files_with_config(left.as_path(), right.as_path(), &cfg)
            .expect("scan ignore line endings");
    assert_eq!(ignore_roots.mode, Mode::File);
    assert_eq!(ignore_files.len(), 1);
    assert_eq!(ignore_files[0].status, EntryStatus::Unchanged);
    assert!(ignore_files[0].hunks.is_empty());

    std::fs::remove_dir_all(&root).expect("cleanup");
}
