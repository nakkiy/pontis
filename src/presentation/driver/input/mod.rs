use std::fmt::Display;

use crossterm::event::KeyEvent;

use crate::app::{App, MergeDirection};

mod keymap;

use self::keymap::{ActionCommand, resolve_command};

pub(super) fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    if let Some(cmd) = resolve_command(key, app.focus()) {
        execute_command(app, cmd);
        return true;
    }
    false
}

fn execute_command(app: &mut App, cmd: ActionCommand) {
    match cmd {
        ActionCommand::Quit => app.request_quit(),
        ActionCommand::FocusFileList => app.focus_file_list(),
        ActionCommand::FocusDiff => app.focus_diff(),
        ActionCommand::SelectNextFile => app.select_next_file(),
        ActionCommand::SelectPrevFile => app.select_prev_file(),
        ActionCommand::SelectNextFilePage => app.select_next_file_page(),
        ActionCommand::SelectPrevFilePage => app.select_prev_file_page(),
        ActionCommand::ScrollFileListLeft(cols) => app.scroll_file_list_left(cols),
        ActionCommand::ScrollFileListRight(cols) => app.scroll_file_list_right(cols),
        ActionCommand::ScrollFileListLeftEdge => app.scroll_file_list_left_edge(),
        ActionCommand::ScrollFileListRightEdge => app.scroll_file_list_right_edge(),
        ActionCommand::ScrollDown(lines) => app.scroll_down(lines),
        ActionCommand::ScrollUp(lines) => app.scroll_up(lines),
        ActionCommand::ScrollLeft(cols) => app.scroll_left(cols),
        ActionCommand::ScrollRight(cols) => app.scroll_right(cols),
        ActionCommand::ScrollLeftEdge => app.scroll_left_edge(),
        ActionCommand::ScrollRightEdge => app.scroll_right_edge(),
        ActionCommand::NextHunkOrFile => app.next_hunk_or_file(),
        ActionCommand::PrevHunkOrFile => app.prev_hunk_or_file(),
        ActionCommand::MergeLeftToRight => app.merge_current_hunk(MergeDirection::LeftToRight),
        ActionCommand::MergeRightToLeft => app.merge_current_hunk(MergeDirection::RightToLeft),
        ActionCommand::SaveCurrent => {
            let result = app.save_current();
            report_action_result(app, "save failed", result);
        }
        ActionCommand::SaveAll => {
            let result = app.save_all();
            report_action_result(app, "save failed", result);
        }
        ActionCommand::UndoMerge => app.undo_merge(),
        ActionCommand::RedoMerge => app.redo_merge(),
        ActionCommand::EditRightWithEditor => {
            let result = app.edit_current_side_with_editor(false);
            report_action_result(app, "editor failed", result);
        }
        ActionCommand::EditLeftWithEditor => {
            let result = app.edit_current_side_with_editor(true);
            report_action_result(app, "editor failed", result);
        }
        ActionCommand::ToggleAddedVisibility => app.toggle_show_added(),
        ActionCommand::ToggleModifiedVisibility => app.toggle_show_modified(),
        ActionCommand::ToggleDeletedVisibility => app.toggle_show_deleted(),
        ActionCommand::ToggleRenamedVisibility => app.toggle_show_renamed(),
        ActionCommand::ToggleUnchangedVisibility => app.toggle_show_unchanged(),
        ActionCommand::ResetStatusFilter => app.reset_file_status_filter(),
    }
}

fn report_action_result<T, E>(app: &mut App, prefix: &str, result: Result<T, E>)
where
    E: Display,
{
    if let Err(err) = result {
        app.set_error_status(&format!("{prefix}: {err:#}"));
    }
}
