use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

use crate::settings::AppSettings;

use super::super::parse::{
    parse_line_ending_policy, parse_line_ending_visibility, parse_whitespace_policy,
    set_line_ending_policy, set_whitespace_policy,
};
use super::super::push_invalid_config_warning;

pub(crate) fn apply_env_config(cfg: &mut AppSettings, warnings: &mut Vec<String>) {
    if let Some(v) = env::var_os("PONTIS_BACKUP_ON_SAVE") {
        match parse_bool(v.to_string_lossy().as_ref()) {
            Some(value) => cfg.backup_on_save = value,
            None => push_invalid_config_warning(warnings, "PONTIS_BACKUP_ON_SAVE"),
        }
    }

    if let Some(v) = env::var_os("PONTIS_HIGHLIGHT_MAX_BYTES") {
        match v.to_string_lossy().parse::<usize>() {
            Ok(value) => cfg.highlight_max_bytes = value,
            Err(_) => push_invalid_config_warning(warnings, "PONTIS_HIGHLIGHT_MAX_BYTES"),
        }
    }

    if let Some(v) = env::var_os("PONTIS_HIGHLIGHT_MAX_LINES") {
        match v.to_string_lossy().parse::<usize>() {
            Ok(value) => cfg.highlight_max_lines = value,
            Err(_) => push_invalid_config_warning(warnings, "PONTIS_HIGHLIGHT_MAX_LINES"),
        }
    }

    if let Some(v) = env::var_os("PONTIS_THEME") {
        let value = v.to_string_lossy().trim().to_string();
        if value.is_empty() {
            push_invalid_config_warning(warnings, "PONTIS_THEME");
        } else {
            cfg.theme = value;
        }
    }

    if let Some(v) = env::var_os("PONTIS_INLINE_DIFF") {
        match parse_bool(v.to_string_lossy().as_ref()) {
            Some(value) => cfg.inline_diff = value,
            None => push_invalid_config_warning(warnings, "PONTIS_INLINE_DIFF"),
        }
    }

    if let Some(v) = env::var_os("PONTIS_LINE_NUMBERS") {
        match parse_bool(v.to_string_lossy().as_ref()) {
            Some(value) => cfg.line_numbers = value,
            None => push_invalid_config_warning(warnings, "PONTIS_LINE_NUMBERS"),
        }
    }

    if let Some(v) = env::var_os("PONTIS_LINE_ENDING_VISIBILITY") {
        match parse_line_ending_visibility(v.to_string_lossy().as_ref()) {
            Some(value) => cfg.line_ending_visibility = value,
            None => push_invalid_config_warning(warnings, "PONTIS_LINE_ENDING_VISIBILITY"),
        }
    }

    if let Some(v) = env::var_os("PONTIS_WHITESPACE_POLICY") {
        match parse_whitespace_policy(v.to_string_lossy().as_ref()) {
            Some(value) => set_whitespace_policy(cfg, value),
            None => push_invalid_config_warning(warnings, "PONTIS_WHITESPACE_POLICY"),
        }
    }

    if let Some(v) = env::var_os("PONTIS_LINE_ENDING_POLICY") {
        match parse_line_ending_policy(v.to_string_lossy().as_ref()) {
            Some(value) => set_line_ending_policy(cfg, value),
            None => push_invalid_config_warning(warnings, "PONTIS_LINE_ENDING_POLICY"),
        }
    }
}

pub(crate) fn default_config_path() -> Option<PathBuf> {
    default_config_dir().map(|dir| dir.join("config.toml"))
}

pub(crate) fn default_config_dir() -> Option<PathBuf> {
    resolve_default_config_dir(env::var_os("XDG_CONFIG_HOME"), env::var_os("HOME"))
}

fn resolve_default_config_dir(
    xdg: Option<impl AsRef<OsStr>>,
    home: Option<impl AsRef<OsStr>>,
) -> Option<PathBuf> {
    if let Some(xdg) = xdg {
        return Some(PathBuf::from(xdg.as_ref()).join("pontis"));
    }

    home.map(|home| PathBuf::from(home.as_ref()).join(".config/pontis"))
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::diff::{LineEndingPolicy, WhitespacePolicy};
    use crate::settings::{AppSettings, LineEndingVisibility};

    use super::{
        super::super::parse::{
            parse_line_ending_policy, parse_line_ending_visibility, parse_whitespace_policy,
        },
        apply_env_config, parse_bool, resolve_default_config_dir,
    };

    #[test]
    fn parse_bool_variants() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("OFF"), Some(false));
        assert_eq!(parse_bool("bad"), None);
    }

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
    fn apply_env_config_reads_inline_diff() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        unsafe {
            std::env::set_var("PONTIS_INLINE_DIFF", "off");
        }
        apply_env_config(&mut cfg, &mut warnings);
        unsafe {
            std::env::remove_var("PONTIS_INLINE_DIFF");
        }

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
    fn apply_env_config_reads_line_ending_policy() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        unsafe {
            std::env::set_var("PONTIS_LINE_ENDING_POLICY", "ignore");
        }
        apply_env_config(&mut cfg, &mut warnings);
        unsafe {
            std::env::remove_var("PONTIS_LINE_ENDING_POLICY");
        }

        assert_eq!(cfg.line_ending_policy(), LineEndingPolicy::Ignore);
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
    fn apply_env_config_reads_whitespace_policy() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        unsafe {
            std::env::set_var("PONTIS_WHITESPACE_POLICY", "ignore");
        }
        apply_env_config(&mut cfg, &mut warnings);
        unsafe {
            std::env::remove_var("PONTIS_WHITESPACE_POLICY");
        }

        assert_eq!(cfg.whitespace_policy(), WhitespacePolicy::Ignore);
        assert!(warnings.is_empty());
    }

    #[test]
    fn apply_env_config_reads_theme() {
        let mut cfg = AppSettings::default();
        let mut warnings = Vec::new();

        unsafe {
            std::env::set_var("PONTIS_THEME", "InspiredGitHub");
        }
        apply_env_config(&mut cfg, &mut warnings);
        unsafe {
            std::env::remove_var("PONTIS_THEME");
        }

        assert_eq!(cfg.theme, "InspiredGitHub");
        assert!(warnings.is_empty());
    }

    #[test]
    fn resolve_default_config_dir_prefers_xdg() {
        let got = resolve_default_config_dir(Some("/tmp/xdg"), Some("/tmp/home"));
        assert_eq!(got, Some(PathBuf::from("/tmp/xdg/pontis")));
    }

    #[test]
    fn resolve_default_config_dir_uses_home_when_xdg_missing() {
        let got = resolve_default_config_dir(None::<&str>, Some("/tmp/home"));
        assert_eq!(got, Some(PathBuf::from("/tmp/home/.config/pontis")));
    }

    #[test]
    fn resolve_default_config_dir_returns_none_when_env_unavailable() {
        let got = resolve_default_config_dir(None::<&str>, None::<&str>);
        assert_eq!(got, None);
    }
}
