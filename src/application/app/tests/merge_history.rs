use super::helpers::{
    diff_file_with_leading_context, multi_hunk_file_with_expanding_first_hunk, sample_diff_file,
    sample_roots,
};
use super::test_loader;
use crate::app::{App, MergeDirection};
use crate::settings::AppSettings;

#[test]
fn merge_left_to_right_applies_hunk() {
    let file = sample_diff_file();
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );
    app.merge_current_hunk(MergeDirection::LeftToRight);
    assert_eq!(app.files[0].right_text, "left\n");
    assert!(app.files[0].right_dirty);
}

#[test]
fn undo_and_redo_restore_merge_state() {
    let file = sample_diff_file();
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );
    app.merge_current_hunk(MergeDirection::LeftToRight);
    assert_eq!(app.files[0].right_text, "left\n");
    app.undo_merge();
    assert_eq!(app.files[0].right_text, "right\n");
    app.redo_merge();
    assert_eq!(app.files[0].right_text, "left\n");
}

#[test]
fn merge_into_read_only_side_is_blocked() {
    let file = sample_diff_file();
    let roots = sample_roots();

    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        false,
        true,
    );
    app.merge_current_hunk(MergeDirection::RightToLeft);
    assert_eq!(app.files[0].left_text, "left\n");
    assert!(app.status_line.contains("read-only"));
}

#[test]
fn scroll_is_clamped_to_known_diff_bounds() {
    let file = sample_diff_file();
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.update_diff_scroll_limits(3, 4);
    app.scroll_down(20);
    app.scroll_right(20);
    assert_eq!(app.scroll_y(), 3);
    assert_eq!(app.scroll_x(), 4);

    app.update_diff_scroll_limits(0, 0);
    assert_eq!(app.scroll_y(), 0);
    assert_eq!(app.scroll_x(), 0);

    app.scroll_down(1);
    app.scroll_right(1);
    assert_eq!(app.scroll_y(), 0);
    assert_eq!(app.scroll_x(), 0);
}

#[test]
fn initial_loaded_file_focuses_first_hunk_once_scroll_limits_are_known() {
    let file = diff_file_with_leading_context(12);
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    assert_eq!(app.scroll_y(), 0);

    app.update_diff_scroll_limits(20, 0);
    app.sync_pending_hunk_focus();

    assert_eq!(app.scroll_y(), 10);
}

#[test]
fn pending_hunk_focus_sync_does_not_override_later_manual_scroll() {
    let file = diff_file_with_leading_context(12);
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.update_diff_scroll_limits(20, 0);
    app.sync_pending_hunk_focus();
    assert_eq!(app.scroll_y(), 10);

    app.scroll_up(4);
    app.sync_pending_hunk_focus();

    assert_eq!(app.scroll_y(), 6);
}

#[test]
fn next_hunk_focus_uses_aligned_display_row_not_raw_line_index() {
    let file = multi_hunk_file_with_expanding_first_hunk();
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.update_diff_scroll_limits(20, 0);
    app.sync_pending_hunk_focus();
    assert_eq!(app.scroll_y(), 0);

    app.next_hunk_or_file();
    app.sync_pending_hunk_focus();

    assert_eq!(app.current_hunk(), 1);
    assert_eq!(app.scroll_y(), 1);
}

#[test]
fn merge_is_noop_when_no_visible_file_is_selected() {
    let file = sample_diff_file();
    let roots = sample_roots();
    let mut app = App::new(
        vec![file],
        roots,
        false,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.toggle_show_modified();
    app.merge_current_hunk(MergeDirection::LeftToRight);

    assert_eq!(app.status_line(), "no visible file selected");
    assert_eq!(app.files()[0].right_text, "right\n");
    assert!(!app.files()[0].right_dirty);
}
