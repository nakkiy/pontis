use std::ffi::OsStr;

use crate::infrastructure::config::{CliOverrides, resolve_config};
use crate::settings::AppSettings;

use super::Cli;

pub(super) fn resolve_runtime_config(cli: &Cli) -> AppSettings {
    let (cfg, cfg_path, warnings) = resolve_config(CliOverrides {
        config_path: cli.config.clone(),
    });
    if let Some(path) = cfg_path
        && should_emit_config_path_log(std::env::var_os("RUST_LOG").as_deref())
    {
        eprintln!("pontis config path: {}", path.display());
    }
    for warning in warnings {
        eprintln!("pontis config warning: {warning}");
    }
    cfg
}

fn should_emit_config_path_log(rust_log: Option<&OsStr>) -> bool {
    rust_log.is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::should_emit_config_path_log;

    #[test]
    fn config_path_log_is_disabled_without_rust_log() {
        assert!(!should_emit_config_path_log(None));
        assert!(!should_emit_config_path_log(Some(OsStr::new(""))));
    }

    #[test]
    fn config_path_log_is_enabled_with_non_empty_rust_log() {
        assert!(should_emit_config_path_log(Some(OsStr::new("debug"))));
        assert!(should_emit_config_path_log(Some(OsStr::new(
            "pontis=trace"
        ))));
    }
}
