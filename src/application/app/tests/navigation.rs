use super::helpers::{
    added_loaded_file, deleted_loaded_file, multi_hunk_file_with_expanding_first_hunk,
    sample_roots, unchanged_loaded_file,
};
use super::test_loader;
use crate::app::{App, Focus};
use crate::settings::AppSettings;

#[test]
fn next_hunk_or_file_skips_loaded_files_without_hunks() {
    let files = vec![
        multi_hunk_file_with_expanding_first_hunk(),
        unchanged_loaded_file("unchanged.txt"),
        multi_hunk_file_with_expanding_first_hunk(),
    ];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.next_hunk_or_file();
    assert_eq!(app.current_hunk(), 1);

    app.next_hunk_or_file();

    assert_eq!(app.current_file_index(), 2);
    assert_eq!(app.current_hunk(), 0);
}

#[test]
fn prev_hunk_or_file_skips_loaded_files_without_hunks_and_selects_last_hunk() {
    let files = vec![
        multi_hunk_file_with_expanding_first_hunk(),
        unchanged_loaded_file("unchanged.txt"),
        multi_hunk_file_with_expanding_first_hunk(),
    ];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.select_file(2);
    app.prev_hunk_or_file();

    assert_eq!(app.current_file_index(), 0);
    assert_eq!(app.current_hunk(), 1);
}

#[test]
fn file_list_scroll_follows_selection_when_it_goes_outside_viewport() {
    let files = vec![
        unchanged_loaded_file("a.txt"),
        unchanged_loaded_file("b.txt"),
        unchanged_loaded_file("c.txt"),
        unchanged_loaded_file("d.txt"),
        unchanged_loaded_file("e.txt"),
    ];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.select_file(4);
    app.update_file_list_scroll_limits(3, 0, 2);

    assert_eq!(app.file_list_scroll_y(), 3);
}

#[test]
fn file_list_horizontal_scroll_is_clamped_to_limits() {
    let files = vec![unchanged_loaded_file(
        "deep/nested/path/with/very/long/name.txt",
    )];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.update_file_list_scroll_limits(0, 8, 1);
    app.scroll_file_list_right(100);
    assert_eq!(app.file_list_scroll_x(), 8);

    app.scroll_file_list_left(3);
    assert_eq!(app.file_list_scroll_x(), 5);
}

#[test]
fn file_list_page_navigation_moves_selection_by_ten_rows() {
    let files = (0..20)
        .map(|idx| unchanged_loaded_file(&format!("{idx}.txt")))
        .collect();
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.select_next_file_page();
    assert_eq!(app.current_file_index(), 10);

    app.select_next_file_page();
    assert_eq!(app.current_file_index(), 19);

    app.select_prev_file_page();
    assert_eq!(app.current_file_index(), 9);
}

#[test]
fn file_list_horizontal_edge_navigation_moves_to_scroll_limits() {
    let files = vec![unchanged_loaded_file(
        "deep/nested/path/with/very/long/name.txt",
    )];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.update_file_list_scroll_limits(0, 8, 1);
    app.scroll_file_list_right(3);
    app.scroll_file_list_right_edge();
    assert_eq!(app.file_list_scroll_x(), 8);

    app.scroll_file_list_left_edge();
    assert_eq!(app.file_list_scroll_x(), 0);
}

#[test]
fn focus_specific_hint_updates_when_focus_changes() {
    let files = vec![unchanged_loaded_file("a.txt")];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    assert_eq!(app.focus(), Focus::FileList);
    assert_eq!(app.status_line(), App::FILE_LIST_HINT);

    app.focus_diff();
    assert_eq!(app.focus(), Focus::Diff);
    assert_eq!(app.status_line(), App::DIFF_HINT);

    app.focus_file_list();
    assert_eq!(app.focus(), Focus::FileList);
    assert_eq!(app.status_line(), App::FILE_LIST_HINT);
}

#[test]
fn status_filter_toggle_retargets_selection_when_current_is_hidden() {
    let files = vec![
        multi_hunk_file_with_expanding_first_hunk(),
        unchanged_loaded_file("same.txt"),
        added_loaded_file("new.txt"),
        deleted_loaded_file("removed.txt"),
    ];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    assert_eq!(app.current_file_index(), 0);
    assert_eq!(app.visible_file_indices(), &[0, 1, 2, 3]);

    app.toggle_show_modified();
    assert_eq!(app.current_file_index(), 1);
    assert_eq!(app.visible_file_indices(), &[1, 2, 3]);
}

#[test]
fn file_navigation_skips_hidden_status_entries() {
    let files = vec![
        multi_hunk_file_with_expanding_first_hunk(),
        added_loaded_file("new.txt"),
        deleted_loaded_file("removed.txt"),
    ];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.toggle_show_added();
    assert_eq!(app.visible_file_indices(), &[0, 2]);

    app.select_next_file();
    assert_eq!(app.current_file_index(), 2);

    app.select_prev_file();
    assert_eq!(app.current_file_index(), 0);
}

#[test]
fn turning_off_all_status_filters_hides_all_files() {
    let files = vec![
        multi_hunk_file_with_expanding_first_hunk(),
        unchanged_loaded_file("same.txt"),
        added_loaded_file("new.txt"),
        deleted_loaded_file("removed.txt"),
    ];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.toggle_show_modified();
    app.toggle_show_unchanged();
    app.toggle_show_added();
    app.toggle_show_deleted();

    assert!(app.visible_file_indices().is_empty());
    assert!(app.current_file().is_none());
    assert_eq!(app.status_line(), App::FILE_LIST_HINT);

    app.select_next_file();
    assert_eq!(app.current_file_index(), 3);
    assert!(app.current_file().is_none());
}

#[test]
fn next_hunk_navigation_skips_hidden_files() {
    let files = vec![
        multi_hunk_file_with_expanding_first_hunk(),
        added_loaded_file("new.txt"),
        multi_hunk_file_with_expanding_first_hunk(),
    ];
    let roots = sample_roots();
    let mut app = App::new(
        files,
        roots,
        AppSettings::default(),
        test_loader(),
        true,
        true,
    );

    app.toggle_show_added();
    assert_eq!(app.visible_file_indices(), &[0, 2]);

    app.next_hunk_or_file();
    assert_eq!(app.current_hunk(), 1);

    app.next_hunk_or_file();
    assert_eq!(app.current_file_index(), 2);
}
