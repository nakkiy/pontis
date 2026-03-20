use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Focus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ActionCommand {
    Quit,
    FocusFileList,
    FocusDiff,
    SelectNextFile,
    SelectPrevFile,
    SelectNextFilePage,
    SelectPrevFilePage,
    ScrollFileListLeft(u16),
    ScrollFileListRight(u16),
    ScrollFileListLeftEdge,
    ScrollFileListRightEdge,
    ScrollDown(u16),
    ScrollUp(u16),
    ScrollLeft(u16),
    ScrollRight(u16),
    ScrollLeftEdge,
    ScrollRightEdge,
    NextHunkOrFile,
    PrevHunkOrFile,
    MergeLeftToRight,
    MergeRightToLeft,
    SaveCurrent,
    SaveAll,
    UndoMerge,
    RedoMerge,
    EditRightWithEditor,
    EditLeftWithEditor,
    ToggleAddedVisibility,
    ToggleModifiedVisibility,
    ToggleDeletedVisibility,
    ToggleRenamedVisibility,
    ToggleUnchangedVisibility,
    ResetStatusFilter,
}

pub(super) fn resolve_command(key: KeyEvent, focus: Focus) -> Option<ActionCommand> {
    if key.code == KeyCode::Char('q') {
        return Some(ActionCommand::Quit);
    }

    if key.modifiers.contains(KeyModifiers::ALT) {
        return resolve_alt_command(key.code);
    }

    resolve_plain_command(key.code, focus)
}

fn resolve_alt_command(code: KeyCode) -> Option<ActionCommand> {
    match code {
        KeyCode::Down => Some(ActionCommand::NextHunkOrFile),
        KeyCode::Up => Some(ActionCommand::PrevHunkOrFile),
        KeyCode::Right => Some(ActionCommand::MergeLeftToRight),
        KeyCode::Left => Some(ActionCommand::MergeRightToLeft),
        _ => None,
    }
}

fn resolve_plain_command(code: KeyCode, focus: Focus) -> Option<ActionCommand> {
    match code {
        KeyCode::Esc if focus == Focus::Diff => Some(ActionCommand::FocusFileList),
        KeyCode::Enter if focus == Focus::FileList => Some(ActionCommand::FocusDiff),
        KeyCode::Down if focus == Focus::FileList => Some(ActionCommand::SelectNextFile),
        KeyCode::Down => Some(ActionCommand::ScrollDown(1)),
        KeyCode::Up if focus == Focus::FileList => Some(ActionCommand::SelectPrevFile),
        KeyCode::Up => Some(ActionCommand::ScrollUp(1)),
        KeyCode::Left if focus == Focus::Diff => Some(ActionCommand::ScrollLeft(2)),
        KeyCode::Left if focus == Focus::FileList => Some(ActionCommand::ScrollFileListLeft(2)),
        KeyCode::Right if focus == Focus::Diff => Some(ActionCommand::ScrollRight(2)),
        KeyCode::Right if focus == Focus::FileList => Some(ActionCommand::ScrollFileListRight(2)),
        KeyCode::PageDown if focus == Focus::FileList => Some(ActionCommand::SelectNextFilePage),
        KeyCode::PageDown => Some(ActionCommand::ScrollDown(10)),
        KeyCode::PageUp if focus == Focus::FileList => Some(ActionCommand::SelectPrevFilePage),
        KeyCode::PageUp => Some(ActionCommand::ScrollUp(10)),
        KeyCode::Home if focus == Focus::FileList => Some(ActionCommand::ScrollFileListLeftEdge),
        KeyCode::Home if focus == Focus::Diff => Some(ActionCommand::ScrollLeftEdge),
        KeyCode::End if focus == Focus::FileList => Some(ActionCommand::ScrollFileListRightEdge),
        KeyCode::End if focus == Focus::Diff => Some(ActionCommand::ScrollRightEdge),
        KeyCode::Char('s') => Some(ActionCommand::SaveCurrent),
        KeyCode::Char('S') => Some(ActionCommand::SaveAll),
        KeyCode::Char('u') => Some(ActionCommand::UndoMerge),
        KeyCode::Char('r') => Some(ActionCommand::RedoMerge),
        KeyCode::Char('e') => Some(ActionCommand::EditRightWithEditor),
        KeyCode::Char('E') => Some(ActionCommand::EditLeftWithEditor),
        KeyCode::Char('A') if focus == Focus::FileList => {
            Some(ActionCommand::ToggleAddedVisibility)
        }
        KeyCode::Char('M') if focus == Focus::FileList => {
            Some(ActionCommand::ToggleModifiedVisibility)
        }
        KeyCode::Char('D') if focus == Focus::FileList => {
            Some(ActionCommand::ToggleDeletedVisibility)
        }
        KeyCode::Char('R') if focus == Focus::FileList => {
            Some(ActionCommand::ToggleRenamedVisibility)
        }
        KeyCode::Char('=') if focus == Focus::FileList => {
            Some(ActionCommand::ToggleUnchangedVisibility)
        }
        KeyCode::Char('f') if focus == Focus::FileList => Some(ActionCommand::ResetStatusFilter),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{ActionCommand, resolve_command};
    use crate::app::Focus;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn down_maps_by_focus() {
        assert_eq!(
            resolve_command(key(KeyCode::Down), Focus::FileList),
            Some(ActionCommand::SelectNextFile)
        );
        assert_eq!(
            resolve_command(key(KeyCode::Down), Focus::Diff),
            Some(ActionCommand::ScrollDown(1))
        );
    }

    #[test]
    fn enter_only_maps_in_file_list() {
        assert_eq!(
            resolve_command(key(KeyCode::Enter), Focus::FileList),
            Some(ActionCommand::FocusDiff)
        );
        assert_eq!(resolve_command(key(KeyCode::Enter), Focus::Diff), None);
    }

    #[test]
    fn esc_only_maps_in_diff_focus() {
        assert_eq!(
            resolve_command(key(KeyCode::Esc), Focus::Diff),
            Some(ActionCommand::FocusFileList)
        );
        assert_eq!(resolve_command(key(KeyCode::Esc), Focus::FileList), None);
    }

    #[test]
    fn alt_merge_and_navigation_are_mapped() {
        let alt = KeyModifiers::ALT;
        assert_eq!(
            resolve_command(KeyEvent::new(KeyCode::Right, alt), Focus::Diff),
            Some(ActionCommand::MergeLeftToRight)
        );
        assert_eq!(
            resolve_command(KeyEvent::new(KeyCode::Down, alt), Focus::Diff),
            Some(ActionCommand::NextHunkOrFile)
        );
    }

    #[test]
    fn horizontal_scroll_maps_in_diff_focus() {
        assert_eq!(
            resolve_command(key(KeyCode::Right), Focus::Diff),
            Some(ActionCommand::ScrollRight(2))
        );
        assert_eq!(
            resolve_command(key(KeyCode::Left), Focus::Diff),
            Some(ActionCommand::ScrollLeft(2))
        );
        assert_eq!(
            resolve_command(key(KeyCode::Home), Focus::Diff),
            Some(ActionCommand::ScrollLeftEdge)
        );
        assert_eq!(
            resolve_command(key(KeyCode::End), Focus::Diff),
            Some(ActionCommand::ScrollRightEdge)
        );
        assert_eq!(
            resolve_command(key(KeyCode::Right), Focus::FileList),
            Some(ActionCommand::ScrollFileListRight(2))
        );
        assert_eq!(
            resolve_command(key(KeyCode::Left), Focus::FileList),
            Some(ActionCommand::ScrollFileListLeft(2))
        );
        assert_eq!(
            resolve_command(key(KeyCode::Home), Focus::FileList),
            Some(ActionCommand::ScrollFileListLeftEdge)
        );
        assert_eq!(
            resolve_command(key(KeyCode::End), Focus::FileList),
            Some(ActionCommand::ScrollFileListRightEdge)
        );
    }

    #[test]
    fn page_keys_follow_focus_specific_behavior() {
        assert_eq!(
            resolve_command(key(KeyCode::PageDown), Focus::FileList),
            Some(ActionCommand::SelectNextFilePage)
        );
        assert_eq!(
            resolve_command(key(KeyCode::PageUp), Focus::FileList),
            Some(ActionCommand::SelectPrevFilePage)
        );
        assert_eq!(
            resolve_command(key(KeyCode::PageDown), Focus::Diff),
            Some(ActionCommand::ScrollDown(10))
        );
        assert_eq!(
            resolve_command(key(KeyCode::PageUp), Focus::Diff),
            Some(ActionCommand::ScrollUp(10))
        );
    }

    #[test]
    fn file_list_filter_keys_only_map_in_file_list_focus() {
        assert_eq!(
            resolve_command(key(KeyCode::Char('A')), Focus::FileList),
            Some(ActionCommand::ToggleAddedVisibility)
        );
        assert_eq!(
            resolve_command(key(KeyCode::Char('M')), Focus::FileList),
            Some(ActionCommand::ToggleModifiedVisibility)
        );
        assert_eq!(
            resolve_command(key(KeyCode::Char('D')), Focus::FileList),
            Some(ActionCommand::ToggleDeletedVisibility)
        );
        assert_eq!(
            resolve_command(key(KeyCode::Char('R')), Focus::FileList),
            Some(ActionCommand::ToggleRenamedVisibility)
        );
        assert_eq!(
            resolve_command(key(KeyCode::Char('=')), Focus::FileList),
            Some(ActionCommand::ToggleUnchangedVisibility)
        );
        assert_eq!(
            resolve_command(key(KeyCode::Char('f')), Focus::FileList),
            Some(ActionCommand::ResetStatusFilter)
        );
        assert_eq!(resolve_command(key(KeyCode::Char('A')), Focus::Diff), None);
    }
}
