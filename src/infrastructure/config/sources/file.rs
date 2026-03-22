use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::settings::AppSettings;

use super::super::parse::{
    parse_line_ending_visibility, parse_line_endings, parse_whitespace, set_line_endings,
    set_whitespace,
};
use super::super::push_invalid_config_warning;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct CompareConfig {
    whitespace: Option<String>,
    line_endings: Option<String>,
    inline_diff: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ViewConfig {
    line_numbers: Option<bool>,
    line_ending_visibility: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct HighlightConfig {
    theme: Option<String>,
    max_bytes: Option<usize>,
    max_lines: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct SaveConfig {
    create_backup: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    compare: Option<CompareConfig>,
    view: Option<ViewConfig>,
    highlight: Option<HighlightConfig>,
    save: Option<SaveConfig>,
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
    if let Some(compare) = file_cfg.compare {
        if let Some(v) = compare.inline_diff {
            cfg.compare.inline_diff = v;
        }
        if let Some(v) = compare.line_endings {
            match parse_line_endings(&v) {
                Some(policy) => set_line_endings(cfg, policy),
                None => push_invalid_config_warning(warnings, "compare.line_endings"),
            }
        }
        if let Some(v) = compare.whitespace {
            match parse_whitespace(&v) {
                Some(policy) => set_whitespace(cfg, policy),
                None => push_invalid_config_warning(warnings, "compare.whitespace"),
            }
        }
    }

    if let Some(view) = file_cfg.view {
        if let Some(v) = view.line_numbers {
            cfg.view.line_numbers = v;
        }
        if let Some(v) = view.line_ending_visibility {
            match parse_line_ending_visibility(&v) {
                Some(mode) => cfg.view.line_ending_visibility = mode,
                None => push_invalid_config_warning(warnings, "view.line_ending_visibility"),
            }
        }
    }

    if let Some(highlight) = file_cfg.highlight {
        if let Some(v) = highlight.max_bytes {
            cfg.highlight.max_bytes = v;
        }
        if let Some(v) = highlight.max_lines {
            cfg.highlight.max_lines = v;
        }
        if let Some(v) = highlight.theme
            && !v.trim().is_empty()
        {
            cfg.highlight.theme = v;
        }
    }

    if let Some(save) = file_cfg.save
        && let Some(v) = save.create_backup
    {
        cfg.save.create_backup = v;
    }
}

#[cfg(test)]
mod tests {
    use crate::diff::{LineEndingPolicy, WhitespacePolicy};
    use crate::settings::{AppSettings, LineEndingVisibility};

    use super::{
        super::super::parse::{parse_line_ending_visibility, parse_line_endings, parse_whitespace},
        CompareConfig, FileConfig, HighlightConfig, SaveConfig, ViewConfig, apply_file_config,
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
                compare: Some(CompareConfig {
                    inline_diff: Some(false),
                    ..CompareConfig::default()
                }),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert!(!cfg.compare.inline_diff);
        assert!(warnings.is_empty());
    }

    #[test]
    fn parse_line_endings_variants() {
        assert_eq!(
            parse_line_endings("compare"),
            Some(LineEndingPolicy::Compare)
        );
        assert_eq!(parse_line_endings("ignore"), Some(LineEndingPolicy::Ignore));
        assert_eq!(parse_line_endings("bad"), None);
    }

    #[test]
    fn apply_file_config_updates_line_endings() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                compare: Some(CompareConfig {
                    line_endings: Some("ignore".to_string()),
                    ..CompareConfig::default()
                }),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert_eq!(cfg.line_endings(), LineEndingPolicy::Ignore);
        assert!(warnings.is_empty());
    }

    #[test]
    fn apply_file_config_updates_theme() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                highlight: Some(HighlightConfig {
                    theme: Some("InspiredGitHub".to_string()),
                    ..HighlightConfig::default()
                }),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert_eq!(cfg.highlight.theme, "InspiredGitHub");
        assert!(warnings.is_empty());
    }

    #[test]
    fn parse_whitespace_variants() {
        assert_eq!(parse_whitespace("compare"), Some(WhitespacePolicy::Compare));
        assert_eq!(parse_whitespace("ignore"), Some(WhitespacePolicy::Ignore));
        assert_eq!(parse_whitespace("bad"), None);
    }

    #[test]
    fn apply_file_config_updates_whitespace() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                compare: Some(CompareConfig {
                    whitespace: Some("ignore".to_string()),
                    ..CompareConfig::default()
                }),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert_eq!(cfg.whitespace(), WhitespacePolicy::Ignore);
        assert!(warnings.is_empty());
    }

    #[test]
    fn apply_file_config_updates_save_create_backup() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                save: Some(SaveConfig {
                    create_backup: Some(true),
                }),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert!(cfg.save.create_backup);
        assert!(warnings.is_empty());
    }

    #[test]
    fn apply_file_config_updates_view_settings() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        apply_file_config(
            &mut cfg,
            FileConfig {
                view: Some(ViewConfig {
                    line_numbers: Some(true),
                    line_ending_visibility: Some("all".to_string()),
                }),
                ..FileConfig::default()
            },
            &mut warnings,
        );

        assert!(cfg.view.line_numbers);
        assert_eq!(cfg.view.line_ending_visibility, LineEndingVisibility::All);
        assert!(warnings.is_empty());
    }

    #[test]
    fn flat_keys_are_rejected() {
        let raw = "backup_on_save = true\nhighlight_max_bytes = 1\n";
        assert!(toml::from_str::<FileConfig>(raw).is_err());
    }

    #[test]
    fn section_keys_are_deserialized() {
        let raw = r#"
            [compare]
            whitespace = "ignore"
            line_endings = "ignore"
            inline_diff = false

            [view]
            line_numbers = true
            line_ending_visibility = "all"

            [highlight]
            theme = "InspiredGitHub"
            max_bytes = 123
            max_lines = 456

            [save]
            create_backup = true
        "#;

        let parsed = toml::from_str::<FileConfig>(raw).expect("section config");
        let compare = parsed.compare.expect("compare");
        assert_eq!(compare.whitespace.as_deref(), Some("ignore"));
        assert_eq!(compare.line_endings.as_deref(), Some("ignore"));
        assert_eq!(compare.inline_diff, Some(false));
        let view = parsed.view.expect("view");
        assert_eq!(view.line_numbers, Some(true));
        assert_eq!(view.line_ending_visibility.as_deref(), Some("all"));
        let highlight = parsed.highlight.expect("highlight");
        assert_eq!(highlight.theme.as_deref(), Some("InspiredGitHub"));
        assert_eq!(highlight.max_bytes, Some(123));
        assert_eq!(highlight.max_lines, Some(456));
        let save = parsed.save.expect("save");
        assert_eq!(save.create_backup, Some(true));
    }
}
