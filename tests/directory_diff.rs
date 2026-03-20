use std::path::Path;

use pontis::fs_scan::build_diff_files;
use pontis::model::EntryStatus;

#[test]
fn directory_union_and_provisional_pending_statuses_are_built() {
    let left = Path::new("tests/fixtures/dir/left");
    let right = Path::new("tests/fixtures/dir/right");

    let (files, roots) = build_diff_files(left, right).expect("scan directories");

    assert_eq!(roots.left, left.to_path_buf());
    assert_eq!(roots.right, right.to_path_buf());
    assert_eq!(files.len(), 4);

    let mut saw_pending = false;
    let mut saw_deleted = false;
    let mut saw_added = false;
    let mut saw_same_path = false;

    for file in files {
        match file.rel_path.to_string_lossy().as_ref() {
            "src/main.rs" => {
                assert_eq!(file.status, EntryStatus::Pending);
                saw_pending = true;
            }
            "only_left.txt" => {
                assert_eq!(file.status, EntryStatus::Deleted);
                saw_deleted = true;
            }
            "only_right.txt" => {
                assert_eq!(file.status, EntryStatus::Added);
                saw_added = true;
            }
            "same.txt" => {
                assert_eq!(file.status, EntryStatus::Pending);
                saw_same_path = true;
            }
            other => panic!("unexpected file path in union: {other}"),
        }
    }

    assert!(saw_pending);
    assert!(saw_deleted);
    assert!(saw_added);
    assert!(saw_same_path);
}
