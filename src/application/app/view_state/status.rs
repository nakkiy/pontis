use std::time::{Duration, Instant};

use crate::app::{App, Focus};
use crate::model::DiffFile;

const LOADING_STATUS: &str = "loading file diff...";
const NON_UTF8_STATUS: &str =
    "selected file contains non-UTF-8 text: merge/edit/highlight are disabled";
const BINARY_STATUS: &str = "selected binary file: merge and syntax highlight are disabled";
const LARGE_FILE_STATUS: &str = "large file detected: syntax highlight disabled for performance";

impl App {
    pub(crate) fn update_context_status(&mut self) {
        if self.is_temporary_status_active() {
            return;
        }
        self.status_line = context_status_line(self.current_file(), self.focus).to_string();
    }

    pub(crate) fn tick_status(&mut self) -> bool {
        if let Some(until) = self.status_until
            && Instant::now() >= until
        {
            self.status_until = None;
            self.update_context_status();
            return true;
        }
        false
    }

    pub(crate) fn set_error_status(&mut self, msg: &str) {
        self.status_line = msg.to_string();
        self.status_until = Some(Instant::now() + Duration::from_secs(4));
    }

    pub(crate) fn set_temporary_status(&mut self, msg: &str) {
        self.status_line = msg.to_string();
        self.status_until = Some(Instant::now() + Duration::from_secs(2));
    }

    fn is_temporary_status_active(&self) -> bool {
        self.status_until
            .is_some_and(|until| Instant::now() < until)
    }
}

fn context_status_line(file: Option<&DiffFile>, focus: Focus) -> &'static str {
    let Some(file) = file else {
        return default_hint_for_focus(focus);
    };

    if !file.loaded {
        LOADING_STATUS
    } else if file.has_unsupported_encoding {
        NON_UTF8_STATUS
    } else if file.is_binary {
        BINARY_STATUS
    } else if file.highlight_limited {
        LARGE_FILE_STATUS
    } else {
        default_hint_for_focus(focus)
    }
}

fn default_hint_for_focus(focus: Focus) -> &'static str {
    match focus {
        Focus::FileList => App::FILE_LIST_HINT,
        Focus::Diff => App::DIFF_HINT,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        App, BINARY_STATUS, Focus, LARGE_FILE_STATUS, LOADING_STATUS, NON_UTF8_STATUS,
        context_status_line,
    };
    use crate::model::{DiffContent, DiffFile, EntryStatus};

    #[test]
    fn context_status_line_returns_default_for_missing_file() {
        assert_eq!(
            context_status_line(None, Focus::FileList),
            App::FILE_LIST_HINT
        );
        assert_eq!(context_status_line(None, Focus::Diff), App::DIFF_HINT);
    }

    #[test]
    fn context_status_line_prioritizes_loading_before_other_flags() {
        let mut file = sample_file();
        file.loaded = false;
        file.has_unsupported_encoding = true;
        file.is_binary = true;
        file.highlight_limited = true;

        assert_eq!(
            context_status_line(Some(&file), Focus::FileList),
            LOADING_STATUS
        );
    }

    #[test]
    fn context_status_line_prioritizes_non_utf8_before_binary() {
        let mut file = sample_file();
        file.has_unsupported_encoding = true;
        file.is_binary = true;
        file.highlight_limited = true;

        assert_eq!(
            context_status_line(Some(&file), Focus::FileList),
            NON_UTF8_STATUS
        );
    }

    #[test]
    fn context_status_line_covers_binary_and_large_file_cases() {
        let mut binary = sample_file();
        binary.is_binary = true;
        assert_eq!(
            context_status_line(Some(&binary), Focus::FileList),
            BINARY_STATUS
        );

        let mut large = sample_file();
        large.highlight_limited = true;
        assert_eq!(
            context_status_line(Some(&large), Focus::FileList),
            LARGE_FILE_STATUS
        );
    }

    #[test]
    fn context_status_line_returns_default_for_normal_loaded_text_file() {
        let file = sample_file();
        assert_eq!(
            context_status_line(Some(&file), Focus::FileList),
            App::FILE_LIST_HINT
        );
        assert_eq!(
            context_status_line(Some(&file), Focus::Diff),
            App::DIFF_HINT
        );
    }

    fn sample_file() -> DiffFile {
        DiffFile::new(
            PathBuf::from("x.txt"),
            Some(PathBuf::from("/tmp/l/x.txt")),
            Some(PathBuf::from("/tmp/r/x.txt")),
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
            EntryStatus::Modified,
        )
    }
}
