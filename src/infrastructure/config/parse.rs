use std::path::PathBuf;

use crate::diff::{DiffComparePolicies, LineEndingPolicy, WhitespacePolicy};
use crate::settings::{AppSettings, LineEndingVisibility};

pub(crate) fn parse_line_ending_visibility(value: &str) -> Option<LineEndingVisibility> {
    match value.trim().to_ascii_lowercase().as_str() {
        "hidden" => Some(LineEndingVisibility::Hidden),
        "all" => Some(LineEndingVisibility::All),
        "diff_only" => Some(LineEndingVisibility::DiffOnly),
        _ => None,
    }
}

pub(crate) fn parse_line_endings(value: &str) -> Option<LineEndingPolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "compare" => Some(LineEndingPolicy::Compare),
        "ignore" => Some(LineEndingPolicy::Ignore),
        _ => None,
    }
}

pub(crate) fn parse_whitespace(value: &str) -> Option<WhitespacePolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "compare" => Some(WhitespacePolicy::Compare),
        "ignore" => Some(WhitespacePolicy::Ignore),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CliOverrides {
    pub config_path: Option<PathBuf>,
}

pub(crate) fn set_line_endings(cfg: &mut AppSettings, policy: LineEndingPolicy) {
    cfg.compare.policies = DiffComparePolicies::new(cfg.whitespace(), policy);
}

pub(crate) fn set_whitespace(cfg: &mut AppSettings, policy: WhitespacePolicy) {
    cfg.compare.policies = DiffComparePolicies::new(policy, cfg.line_endings());
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, LineEndingPolicy, LineEndingVisibility, WhitespacePolicy};

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
