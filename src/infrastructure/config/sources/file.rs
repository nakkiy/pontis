use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::settings::AppSettings;

use super::super::parse::{
    parse_line_ending_policy, parse_line_ending_visibility, parse_whitespace_policy,
    set_line_ending_policy, set_whitespace_policy,
};
use super::super::push_invalid_config_warning;

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    backup_on_save: Option<bool>,
    highlight_max_bytes: Option<usize>,
    highlight_max_lines: Option<usize>,
    theme: Option<String>,
    inline_diff: Option<bool>,
    line_ending_policy: Option<String>,
    line_numbers: Option<bool>,
    line_ending_visibility: Option<String>,
    whitespace_policy: Option<String>,
}

pub(crate) fn apply_file_config_from_path(
    cfg: &mut AppSettings,
    path: &Path,
    warnings: &mut Vec<String>,
) {
    match fs::read_to_string(path) {
        Ok(raw) => match toml::from_str::<FileConfig>(&raw) {
            Ok(file_cfg) => apply_file_config(cfg, file_cfg, warnings),
            Err(err) => warnings.push(format!("config parse failed at {}: {err}", path.display())),
        },
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                warnings.push(format!("config read failed at {}: {err}", path.display()));
            }
        }
    }
}

fn apply_file_config(cfg: &mut AppSettings, file_cfg: FileConfig, warnings: &mut Vec<String>) {
    if let Some(v) = file_cfg.backup_on_save {
        cfg.backup_on_save = v;
    }
    if let Some(v) = file_cfg.highlight_max_bytes {
        cfg.highlight_max_bytes = v;
    }
    if let Some(v) = file_cfg.highlight_max_lines {
        cfg.highlight_max_lines = v;
    }
    if let Some(v) = file_cfg.theme
        && !v.trim().is_empty()
    {
        cfg.theme = v;
    }
    if let Some(v) = file_cfg.inline_diff {
        cfg.inline_diff = v;
    }
    if let Some(v) = file_cfg.line_ending_policy {
        match parse_line_ending_policy(&v) {
            Some(policy) => set_line_ending_policy(cfg, policy),
            None => push_invalid_config_warning(warnings, "line_ending_policy"),
        }
    }
    if let Some(v) = file_cfg.line_numbers {
        cfg.line_numbers = v;
    }
    if let Some(v) = file_cfg.line_ending_visibility {
        match parse_line_ending_visibility(&v) {
            Some(mode) => cfg.line_ending_visibility = mode,
            None => push_invalid_config_warning(warnings, "line_ending_visibility"),
        }
    }
    if let Some(v) = file_cfg.whitespace_policy {
        match parse_whitespace_policy(&v) {
            Some(policy) => set_whitespace_policy(cfg, policy),
            None => push_invalid_config_warning(warnings, "whitespace_policy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::diff::{LineEndingPolicy, WhitespacePolicy};
    use crate::settings::{AppSettings, LineEndingVisibility};

    use super::{
        super::super::parse::{
            parse_line_ending_policy, parse_line_ending_visibility, parse_whitespace_policy,
        },
        FileConfig, apply_file_config,
    };

    #[test]
    fn parse_line_ending_visibility_variants() {
        assert_eq!(
            parse_line_ending_visibility("hidden"),
            Some(LineEndingVisibility::Hidden)
        );
        assert_eq!(
            parse_line_ending_visibility("all"),
            Some(LineEndingVisibility::All)
        );
        assert_eq!(
            parse_line_ending_visibility("diff_only"),
            Some(LineEndingVisibility::DiffOnly)
        );
        assert_eq!(parse_line_ending_visibility("bad"), None);
    }

    #[test]
    fn apply_file_config_updates_inline_diff() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                inline_diff: Some(false),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert!(!cfg.inline_diff);
        assert!(warnings.is_empty());
    }

    #[test]
    fn parse_line_ending_policy_variants() {
        assert_eq!(
            parse_line_ending_policy("compare"),
            Some(LineEndingPolicy::Compare)
        );
        assert_eq!(
            parse_line_ending_policy("ignore"),
            Some(LineEndingPolicy::Ignore)
        );
        assert_eq!(parse_line_ending_policy("bad"), None);
    }

    #[test]
    fn apply_file_config_updates_line_ending_policy() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                line_ending_policy: Some("ignore".to_string()),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert_eq!(cfg.line_ending_policy(), LineEndingPolicy::Ignore);
        assert!(warnings.is_empty());
    }

    #[test]
    fn apply_file_config_updates_theme() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                theme: Some("InspiredGitHub".to_string()),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert_eq!(cfg.theme, "InspiredGitHub");
        assert!(warnings.is_empty());
    }

    #[test]
    fn parse_whitespace_policy_variants() {
        assert_eq!(
            parse_whitespace_policy("compare"),
            Some(WhitespacePolicy::Compare)
        );
        assert_eq!(
            parse_whitespace_policy("ignore"),
            Some(WhitespacePolicy::Ignore)
        );
        assert_eq!(parse_whitespace_policy("bad"), None);
    }

    #[test]
    fn apply_file_config_updates_whitespace_policy() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                whitespace_policy: Some("ignore".to_string()),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert_eq!(cfg.whitespace_policy(), WhitespacePolicy::Ignore);
        assert!(warnings.is_empty());
    }
}
