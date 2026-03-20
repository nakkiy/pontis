use std::path::PathBuf;

use crate::diff::WhitespacePolicy;
use crate::model::{DiffContent, DiffFile, EntryStatus};

use super::compute_hunks;
use super::policy::DiffComparePolicies;

impl DiffFile {
    pub fn new(
        rel_path: PathBuf,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        content: DiffContent,
        status: EntryStatus,
    ) -> Self {
        Self::new_with_whitespace_policy(
            rel_path,
            left_path,
            right_path,
            content,
            status,
            WhitespacePolicy::Compare,
        )
    }

    pub fn new_with_whitespace_policy(
        rel_path: PathBuf,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        content: DiffContent,
        status: EntryStatus,
        whitespace_policy: WhitespacePolicy,
    ) -> Self {
        Self::new_with_policies(
            rel_path,
            left_path,
            right_path,
            content,
            status,
            DiffComparePolicies::with_whitespace(whitespace_policy),
        )
    }

    pub fn new_with_policies(
        rel_path: PathBuf,
        left_path: Option<PathBuf>,
        right_path: Option<PathBuf>,
        content: DiffContent,
        status: EntryStatus,
        policies: DiffComparePolicies,
    ) -> Self {
        let hunks = if content.is_binary {
            Vec::new()
        } else {
            compute_hunks(
                &content.left_text,
                &content.right_text,
                policies.whitespace_policy,
                policies.line_ending_policy,
            )
        };

        Self {
            rel_path,
            original_rel_path: None,
            left_path,
            right_path,
            left_text: content.left_text,
            right_text: content.right_text,
            hunks,
            status,
            left_is_binary: content.left_is_binary,
            right_is_binary: content.right_is_binary,
            is_binary: content.is_binary,
            has_unsupported_encoding: content.has_unsupported_encoding,
            left_has_unsupported_encoding: content.left_has_unsupported_encoding,
            right_has_unsupported_encoding: content.right_has_unsupported_encoding,
            left_has_utf8_bom: content.left_has_utf8_bom,
            right_has_utf8_bom: content.right_has_utf8_bom,
            left_bytes: content.left_bytes,
            right_bytes: content.right_bytes,
            highlight_limited: content.highlight_limited,
            loaded: true,
            left_dirty: false,
            right_dirty: false,
        }
    }

    pub fn recompute_hunks(&mut self) {
        self.recompute_hunks_with_whitespace_policy(WhitespacePolicy::Compare);
    }

    pub fn recompute_hunks_with_whitespace_policy(&mut self, whitespace_policy: WhitespacePolicy) {
        self.recompute_hunks_with_policies(DiffComparePolicies::with_whitespace(whitespace_policy));
    }

    pub fn recompute_hunks_with_policies(&mut self, policies: DiffComparePolicies) {
        if self.is_binary {
            self.hunks.clear();
            self.loaded = true;
            return;
        }
        self.hunks = compute_hunks(
            &self.left_text,
            &self.right_text,
            policies.whitespace_policy,
            policies.line_ending_policy,
        );
        self.status = match (self.left_path.is_some(), self.right_path.is_some()) {
            (true, true) => {
                if self.is_renamed() {
                    EntryStatus::Renamed
                } else if self.hunks.is_empty() {
                    EntryStatus::Unchanged
                } else {
                    EntryStatus::Modified
                }
            }
            (true, false) => EntryStatus::Deleted,
            (false, true) => EntryStatus::Added,
            (false, false) => EntryStatus::Unchanged,
        };
        self.loaded = true;
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::model::{DiffContent, DiffFile, EntryStatus};

    #[test]
    fn recompute_hunks_marks_equal_existing_files_unchanged() {
        let mut file = DiffFile::new(
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
        );

        file.right_text = file.left_text.clone();
        file.recompute_hunks();

        assert!(file.hunks.is_empty());
        assert_eq!(file.status, EntryStatus::Unchanged);
    }

    #[test]
    fn recompute_hunks_preserves_added_and_deleted_status_by_presence() {
        let mut added = DiffFile::new(
            PathBuf::from("added.txt"),
            None,
            Some(PathBuf::from("/tmp/r/added.txt")),
            DiffContent {
                left_text: String::new(),
                right_text: "right\n".to_string(),
                left_bytes: 0,
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
        added.recompute_hunks();
        assert_eq!(added.status, EntryStatus::Added);

        let mut deleted = DiffFile::new(
            PathBuf::from("deleted.txt"),
            Some(PathBuf::from("/tmp/l/deleted.txt")),
            None,
            DiffContent {
                left_text: "left\n".to_string(),
                right_text: String::new(),
                left_bytes: 5,
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
        deleted.recompute_hunks();
        assert_eq!(deleted.status, EntryStatus::Deleted);
    }
}
