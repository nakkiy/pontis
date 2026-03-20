use std::path::PathBuf;

use super::parse::CliOverrides;
use super::sources::{apply_env_config, apply_file_config_from_path, default_config_path};
use crate::settings::AppSettings;
use crate::settings::{DEFAULT_HIGHLIGHT_MAX_BYTES, DEFAULT_HIGHLIGHT_MAX_LINES};

pub(crate) fn resolve_config(cli: CliOverrides) -> (AppSettings, Option<PathBuf>, Vec<String>) {
    let mut cfg = AppSettings::default();
    let mut warnings = Vec::new();
    let path = cli.config_path.or_else(default_config_path);

    if let Some(path) = &path {
        apply_file_config_from_path(&mut cfg, path, &mut warnings);
    }

    apply_env_config(&mut cfg, &mut warnings);

    if cfg.highlight_max_bytes == 0 {
        warnings.push("highlight_max_bytes cannot be 0; using default".to_string());
        cfg.highlight_max_bytes = DEFAULT_HIGHLIGHT_MAX_BYTES;
    }
    if cfg.highlight_max_lines == 0 {
        warnings.push("highlight_max_lines cannot be 0; using default".to_string());
        cfg.highlight_max_lines = DEFAULT_HIGHLIGHT_MAX_LINES;
    }

    (cfg, path, warnings)
}
