use std::fs;
use std::sync::{Mutex, OnceLock};

use super::helpers::{sample_diff_file, sample_roots, unique_temp_root};
use super::test_loader;
use crate::app::{App, MergeDirection};
use crate::settings::AppSettings;

fn editor_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(unix)]
#[test]
fn external_edit_clears_undo_and_redo_history() {
    use std::os::unix::fs::PermissionsExt;
    let _guard = editor_env_lock().lock().expect("lock EDITOR env");

    let temp_root = unique_temp_root("pontis-edit-history");
    fs::create_dir_all(temp_root.join("l")).expect("create left root");
    fs::create_dir_all(temp_root.join("r")).expect("create right root");
    let script_path = temp_root.join("editor.sh");
    fs::write(&script_path, "#!/bin/sh\nprintf 'edited\\n' > \"$1\"\n")
        .expect("write editor script");
    let mut perms = fs::metadata(&script_path)
        .expect("script metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("chmod script");

    let original_editor = std::env::var_os("EDITOR");
    // SAFETY: this test serially updates a process-global environment variable and restores it.
    unsafe {
        std::env::set_var("EDITOR", &script_path);
    }

    let mut roots = sample_roots();
    roots.left = temp_root.join("l");
    roots.right = temp_root.join("r");

    let mut file = sample_diff_file();
    file.left_path = Some(roots.left.join("a.txt"));
    file.right_path = Some(roots.right.join("a.txt"));

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
    app.merge_current_hunk(MergeDirection::LeftToRight);
    app.undo_merge();

    app.edit_current_side_with_editor(false)
        .expect("edit right side with external editor");

    assert_eq!(app.files()[0].right_text, "edited\n");

    app.undo_merge();
    assert_eq!(app.status_line(), "undo stack is empty");
    assert_eq!(app.files()[0].right_text, "edited\n");

    app.redo_merge();
    assert_eq!(app.status_line(), "redo stack is empty");
    assert_eq!(app.files()[0].right_text, "edited\n");

    if let Some(value) = original_editor {
        // SAFETY: this test restores the process-global environment variable before exit.
        unsafe {
            std::env::set_var("EDITOR", value);
        }
    } else {
        // SAFETY: this test restores the process-global environment variable before exit.
        unsafe {
            std::env::remove_var("EDITOR");
        }
    }

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[cfg(unix)]
#[test]
fn external_edit_does_not_create_backup_even_when_create_backup_is_enabled() {
    use std::os::unix::fs::PermissionsExt;
    let _guard = editor_env_lock().lock().expect("lock EDITOR env");

    let temp_root = unique_temp_root("pontis-edit-no-backup");
    fs::create_dir_all(temp_root.join("l")).expect("create left root");
    fs::create_dir_all(temp_root.join("r")).expect("create right root");
    let script_path = temp_root.join("editor.sh");
    fs::write(&script_path, "#!/bin/sh\n: > /dev/null\n").expect("write editor script");
    let mut perms = fs::metadata(&script_path)
        .expect("script metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("chmod script");

    let original_editor = std::env::var_os("EDITOR");
    // SAFETY: this test serially updates a process-global environment variable and restores it.
    unsafe {
        std::env::set_var("EDITOR", &script_path);
    }

    let mut roots = sample_roots();
    roots.left = temp_root.join("l");
    roots.right = temp_root.join("r");
    let right_path = roots.right.join("a.txt");
    fs::write(&right_path, "right\n").expect("seed right file");

    let mut file = sample_diff_file();
    file.left_path = Some(roots.left.join("a.txt"));
    file.right_path = Some(right_path.clone());

    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.edit_current_side_with_editor(false)
        .expect("edit right side with external editor");

    assert!(!right_path.with_file_name("a.txt.bak").exists());

    if let Some(value) = original_editor {
        // SAFETY: this test restores the process-global environment variable before exit.
        unsafe {
            std::env::set_var("EDITOR", value);
        }
    } else {
        // SAFETY: this test restores the process-global environment variable before exit.
        unsafe {
            std::env::remove_var("EDITOR");
        }
    }

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[cfg(unix)]
#[test]
fn external_edit_without_changes_preserves_dirty_merge_state_and_does_not_save() {
    use std::os::unix::fs::PermissionsExt;
    let _guard = editor_env_lock().lock().expect("lock EDITOR env");

    let temp_root = unique_temp_root("pontis-edit-no-change");
    fs::create_dir_all(temp_root.join("l")).expect("create left root");
    fs::create_dir_all(temp_root.join("r")).expect("create right root");
    let script_path = temp_root.join("editor.sh");
    fs::write(&script_path, "#!/bin/sh\nexit 0\n").expect("write editor script");
    let mut perms = fs::metadata(&script_path)
        .expect("script metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("chmod script");

    let original_editor = std::env::var_os("EDITOR");
    unsafe {
        std::env::set_var("EDITOR", &script_path);
    }

    let mut roots = sample_roots();
    roots.left = temp_root.join("l");
    roots.right = temp_root.join("r");
    let left_path = roots.left.join("a.txt");
    let right_path = roots.right.join("a.txt");
    fs::write(&left_path, "left\n").expect("seed left file");
    fs::write(&right_path, "right\n").expect("seed right file");

    let mut file = sample_diff_file();
    file.left_path = Some(left_path);
    file.right_path = Some(right_path.clone());

    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.merge_current_hunk(MergeDirection::LeftToRight);
    assert_eq!(app.files()[0].right_text, "left\n");
    assert!(app.files()[0].right_dirty);

    app.edit_current_side_with_editor(false)
        .expect("edit right side with no changes");

    assert_eq!(app.files()[0].right_text, "left\n");
    assert!(app.files()[0].right_dirty);
    assert_eq!(
        fs::read_to_string(&right_path).expect("read right path"),
        "right\n"
    );

    if let Some(value) = original_editor {
        unsafe {
            std::env::set_var("EDITOR", value);
        }
    } else {
        unsafe {
            std::env::remove_var("EDITOR");
        }
    }

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[cfg(unix)]
#[test]
fn external_edit_failure_does_not_modify_target_file() {
    use std::os::unix::fs::PermissionsExt;
    let _guard = editor_env_lock().lock().expect("lock EDITOR env");

    let temp_root = unique_temp_root("pontis-edit-failure");
    fs::create_dir_all(temp_root.join("l")).expect("create left root");
    fs::create_dir_all(temp_root.join("r")).expect("create right root");
    let script_path = temp_root.join("editor.sh");
    fs::write(&script_path, "#!/bin/sh\nexit 7\n").expect("write editor script");
    let mut perms = fs::metadata(&script_path)
        .expect("script metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("chmod script");

    let original_editor = std::env::var_os("EDITOR");
    unsafe {
        std::env::set_var("EDITOR", &script_path);
    }

    let mut roots = sample_roots();
    roots.left = temp_root.join("l");
    roots.right = temp_root.join("r");
    let right_path = roots.right.join("a.txt");
    fs::write(&right_path, "disk\n").expect("seed right file");

    let mut file = sample_diff_file();
    file.right_text = "memory\n".to_string();
    file.right_dirty = true;
    file.left_path = Some(roots.left.join("a.txt"));
    file.right_path = Some(right_path.clone());

    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    let err = app
        .edit_current_side_with_editor(false)
        .expect_err("editor failure should be returned");
    assert!(
        err.to_string()
            .contains("editor exited with non-zero status")
    );
    assert_eq!(
        fs::read_to_string(&right_path).expect("read right path"),
        "disk\n"
    );
    assert_eq!(app.files()[0].right_text, "memory\n");
    assert!(app.files()[0].right_dirty);

    if let Some(value) = original_editor {
        unsafe {
            std::env::set_var("EDITOR", value);
        }
    } else {
        unsafe {
            std::env::remove_var("EDITOR");
        }
    }

    fs::remove_dir_all(temp_root).expect("cleanup temp root");
}

#[test]
fn external_edit_is_noop_when_no_visible_file_is_selected() {
    let mut file = sample_diff_file();
    file.right_dirty = true;

    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.toggle_show_modified();

    app.edit_current_side_with_editor(false)
        .expect("no visible file should be a no-op");

    assert_eq!(app.status_line(), "no visible file selected");
    assert_eq!(app.files()[0].right_text, "right\n");
    assert!(app.files()[0].right_dirty);
}
