use std::path::PathBuf;

use super::helpers::{sample_diff_file, sample_roots, unique_temp_root};
use super::test_loader;
use crate::app::App;
use crate::model::{DiffContent, DiffFile, EntryStatus, Mode, Roots};
use crate::settings::AppSettings;

#[test]
fn save_all_writes_dirty_sides() {
    let root = unique_temp_root("pontis-save-all");
    let left_root = root.join("left");
    let right_root = root.join("right");

    let mut file = DiffFile::new(
        PathBuf::from("x.txt"),
        None,
        None,
        DiffContent {
            left_text: "left\n".to_string(),
            right_text: "right\n".to_string(),
            left_bytes: 5,
            right_bytes: 6,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Added,
    );
    file.left_dirty = true;
    file.right_dirty = true;

    let roots = Roots {
        left: left_root.clone(),
        right: right_root.clone(),
        mode: Mode::Directory,
        left_label: None,
        right_label: None,
    };

    let mut app = App::new(
        vec![file],
        roots,
        AppSettings {
            save: crate::settings::SaveSettings {
                create_backup: true,
            },
            ..AppSettings::default()
        },
        test_loader(),
        true,
        true,
    );
    app.save_all().expect("save all");

    let left_written = std::fs::read_to_string(left_root.join("x.txt")).expect("left file");
    let right_written = std::fs::read_to_string(right_root.join("x.txt")).expect("right file");
    assert_eq!(left_written, "left\n");
    assert_eq!(right_written, "right\n");
    assert!(!app.files[0].left_dirty);
    assert!(!app.files[0].right_dirty);

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn save_current_creates_backup_when_enabled() {
    let root = unique_temp_root("pontis-save-backup");
    let left_root = root.join("left");
    std::fs::create_dir_all(&left_root).expect("mkdir");
    let target = left_root.join("x.txt");
    std::fs::write(&target, "old\n").expect("seed target");

    let mut file = DiffFile::new(
        PathBuf::from("x.txt"),
        Some(target.clone()),
        None,
        DiffContent {
            left_text: "new\n".to_string(),
            right_text: String::new(),
            left_bytes: 4,
            right_bytes: 0,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Deleted,
    );
    file.left_dirty = true;

    let roots = Roots {
        left: left_root.clone(),
        right: root.join("right"),
        mode: Mode::Directory,
        left_label: None,
        right_label: None,
    };
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings {
            save: crate::settings::SaveSettings {
                create_backup: true,
            },
            ..AppSettings::default()
        },
        test_loader(),
        true,
        true,
    );
    app.save_current().expect("save current");

    let backup = left_root.join("x.txt.bak");
    let backup_text = std::fs::read_to_string(&backup).expect("backup exists");
    let new_text = std::fs::read_to_string(&target).expect("new content");
    assert_eq!(backup_text, "old\n");
    assert_eq!(new_text, "new\n");

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn save_current_preserves_utf8_bom_when_source_had_bom() {
    let root = unique_temp_root("pontis-save-bom");
    let left_root = root.join("left");
    std::fs::create_dir_all(&left_root).expect("mkdir");
    let target = left_root.join("bom.txt");

    let mut file = DiffFile::new(
        PathBuf::from("bom.txt"),
        Some(target.clone()),
        None,
        DiffContent {
            left_text: "new\n".to_string(),
            right_text: String::new(),
            left_bytes: 7,
            right_bytes: 0,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: true,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Deleted,
    );
    file.left_dirty = true;

    let roots = Roots {
        left: left_root,
        right: root.join("right"),
        mode: Mode::Directory,
        left_label: None,
        right_label: None,
    };
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );
    app.save_current().expect("save current");

    let written = std::fs::read(&target).expect("read bytes");
    assert!(written.starts_with(&[0xEF, 0xBB, 0xBF]));
    assert_eq!(&written[3..], b"new\n");

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn save_current_keeps_read_only_dirty_side_unsaved() {
    let mut file = sample_diff_file();
    file.left_dirty = true;

    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        false,
        true,
    );
    app.save_current().expect("save current");

    assert!(app.files[0].left_dirty);
    assert_eq!(
        app.status_line(),
        "no writable pending changes for current file"
    );
}

#[test]
fn save_all_only_writes_writable_sides() {
    let root = unique_temp_root("pontis-save-read-only");
    let left_root = root.join("left");
    let right_root = root.join("right");

    let mut file = DiffFile::new(
        PathBuf::from("x.txt"),
        None,
        None,
        DiffContent {
            left_text: "left\n".to_string(),
            right_text: "right\n".to_string(),
            left_bytes: 5,
            right_bytes: 6,
            left_is_binary: false,
            right_is_binary: false,
            is_binary: false,
            has_unsupported_encoding: false,
            left_has_unsupported_encoding: false,
            right_has_unsupported_encoding: false,
            left_has_utf8_bom: false,
            right_has_utf8_bom: false,
            highlight_limited: false,
        },
        EntryStatus::Added,
    );
    file.left_dirty = true;
    file.right_dirty = true;

    let roots = Roots {
        left: left_root.clone(),
        right: right_root.clone(),
        mode: Mode::Directory,
        left_label: None,
        right_label: None,
    };

    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        false,
        true,
    );
    app.save_all().expect("save all");

    assert!(app.files[0].left_dirty);
    assert!(!app.files[0].right_dirty);
    assert!(!left_root.join("x.txt").exists());
    let right_written = std::fs::read_to_string(right_root.join("x.txt")).expect("right file");
    assert_eq!(right_written, "right\n");
    assert_eq!(app.status_line(), "saved 1 side(s)");

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn save_current_is_noop_when_no_visible_file_is_selected() {
    let root = unique_temp_root("pontis-save-hidden");
    let left_root = root.join("left");
    let right_root = root.join("right");
    let right_path = right_root.join("a.txt");

    let mut file = sample_diff_file();
    file.right_path = Some(right_path.clone());
    file.right_dirty = true;

    let roots = Roots {
        left: left_root,
        right: right_root,
        mode: Mode::Directory,
        left_label: None,
        right_label: None,
    };
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.toggle_show_modified();
    app.save_current().expect("save current no-op");

    assert_eq!(app.status_line(), "no visible file selected");
    assert!(!right_path.exists());
    assert!(app.files()[0].right_dirty);

    if root.exists() {
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
