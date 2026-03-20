mod parse;
mod resolve;
mod sources;

pub(crate) use parse::CliOverrides;
pub(crate) use resolve::resolve_config;
pub(crate) use sources::default_config_dir;

pub(crate) fn push_invalid_config_warning(warnings: &mut Vec<String>, key: &str) {
    warnings.push(format!("invalid {key}; ignoring"));
}
