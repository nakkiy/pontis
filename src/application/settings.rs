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
pub struct CompareSettings {
    pub inline_diff: bool,
    pub policies: DiffComparePolicies,
}

#[derive(Debug, Clone)]
pub struct ViewSettings {
    pub line_numbers: bool,
    pub line_ending_visibility: LineEndingVisibility,
}

#[derive(Debug, Clone)]
pub struct HighlightSettings {
    pub theme: String,
    pub max_bytes: usize,
    pub max_lines: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveSettings {
    pub create_backup: bool,
}

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub compare: CompareSettings,
    pub view: ViewSettings,
    pub highlight: HighlightSettings,
    pub save: SaveSettings,
}

impl AppSettings {
    pub const fn whitespace(&self) -> WhitespacePolicy {
        self.compare.policies.whitespace_policy
    }

    pub const fn line_endings(&self) -> LineEndingPolicy {
        self.compare.policies.line_ending_policy
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            compare: CompareSettings {
                inline_diff: true,
                policies: DiffComparePolicies::compare(),
            },
            view: ViewSettings {
                line_numbers: false,
                line_ending_visibility: LineEndingVisibility::Hidden,
            },
            highlight: HighlightSettings {
                theme: String::new(),
                max_bytes: DEFAULT_HIGHLIGHT_MAX_BYTES,
                max_lines: DEFAULT_HIGHLIGHT_MAX_LINES,
            },
            save: SaveSettings {
                create_backup: false,
            },
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
            AppSettings::default().view.line_ending_visibility,
            LineEndingVisibility::Hidden
        );
    }

    #[test]
    fn inline_diff_is_enabled_by_default() {
        assert!(AppSettings::default().compare.inline_diff);
    }

    #[test]
    fn line_ending_policy_defaults_to_compare() {
        assert_eq!(
            AppSettings::default().line_endings(),
            LineEndingPolicy::Compare
        );
    }

    #[test]
    fn whitespace_policy_defaults_to_compare() {
        assert_eq!(
            AppSettings::default().whitespace(),
            WhitespacePolicy::Compare
        );
    }
}
