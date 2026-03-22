use super::helpers::{multi_hunk_file_with_expanding_first_hunk, sample_roots};
use super::test_loader;
use crate::app::{App, ReloadDecision};
use crate::settings::AppSettings;

#[test]
fn request_reload_rejects_when_mode_is_not_supported() {
    let mut app = App::new(
        vec![multi_hunk_file_with_expanding_first_hunk()],
        sample_roots(),
        AppSettings::default(),
        test_loader(),
        true,
        false,
        true,
    );

    assert_eq!(app.request_reload(), ReloadDecision::Rejected);
    assert_eq!(app.status_line(), "reload not available in this mode");
}

#[test]
fn request_reload_rejects_when_unsaved_changes_exist() {
    let mut file = multi_hunk_file_with_expanding_first_hunk();
    file.right_dirty = true;
    let mut app = App::new(
        vec![file],
        sample_roots(),
        AppSettings::default(),
        test_loader(),
        true,
        true,
        true,
    );

    assert_eq!(app.request_reload(), ReloadDecision::Rejected);
    assert_eq!(app.status_line(), "reload unavailable with unsaved changes");
}

#[test]
fn request_reload_starts_when_mode_is_supported_and_clean() {
    let mut app = App::new(
        vec![multi_hunk_file_with_expanding_first_hunk()],
        sample_roots(),
        AppSettings::default(),
        test_loader(),
        true,
        true,
        true,
    );

    assert_eq!(app.request_reload(), ReloadDecision::Start);
}
