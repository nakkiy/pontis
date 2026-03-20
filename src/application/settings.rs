use crate::diff::{DiffComparePolicies, LineEndingPolicy, WhitespacePolicy};

pub const DEFAULT_HIGHLIGHT_MAX_BYTES: usize = 512 * 1024;
pub const DEFAULT_HIGHLIGHT_MAX_LINES: usize = 8_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndingVisibility {
    Hidden,
    All,
    DiffOnly,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub backup_on_save: bool,
    pub highlight_max_bytes: usize,
    pub highlight_max_lines: usize,
    pub theme: String,
    pub inline_diff: bool,
    pub line_numbers: bool,
    pub line_ending_visibility: LineEndingVisibility,
    pub compare_policies: DiffComparePolicies,
}

impl AppSettings {
    pub const fn whitespace_policy(&self) -> WhitespacePolicy {
        self.compare_policies.whitespace_policy
    }

    pub const fn line_ending_policy(&self) -> LineEndingPolicy {
        self.compare_policies.line_ending_policy
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            backup_on_save: false,
            highlight_max_bytes: DEFAULT_HIGHLIGHT_MAX_BYTES,
            highlight_max_lines: DEFAULT_HIGHLIGHT_MAX_LINES,
            theme: String::new(),
            inline_diff: true,
            line_numbers: false,
            line_ending_visibility: LineEndingVisibility::Hidden,
            compare_policies: DiffComparePolicies::compare(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, LineEndingVisibility};
    use crate::diff::{LineEndingPolicy, WhitespacePolicy};

    #[test]
    fn default_line_ending_visibility_is_hidden() {
        assert_eq!(
            AppSettings::default().line_ending_visibility,
            LineEndingVisibility::Hidden
        );
    }

    #[test]
    fn inline_diff_is_enabled_by_default() {
        assert!(AppSettings::default().inline_diff);
    }

    #[test]
    fn line_ending_policy_defaults_to_compare() {
        assert_eq!(
            AppSettings::default().line_ending_policy(),
            LineEndingPolicy::Compare
        );
    }

    #[test]
    fn whitespace_policy_defaults_to_compare() {
        assert_eq!(
            AppSettings::default().whitespace_policy(),
            WhitespacePolicy::Compare
        );
    }
}
