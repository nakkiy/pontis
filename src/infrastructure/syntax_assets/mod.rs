use std::path::{Path, PathBuf};

use syntect::LoadingError;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub(crate) struct SyntaxAssets {
    pub(crate) ps: SyntaxSet,
    pub(crate) ts: ThemeSet,
}

pub(crate) fn load_syntax_assets(config_dir: Option<&Path>) -> SyntaxAssets {
    let ps = load_syntax_set(config_dir);
    let ts = load_theme_set(config_dir);
    SyntaxAssets { ps, ts }
}

fn load_syntax_set(config_dir: Option<&Path>) -> SyntaxSet {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    if let Some(config_dir) = config_dir {
        let _ = add_syntaxes_from_folder(&mut builder, config_dir.join("syntaxes"));
    }
    builder.build()
}

fn add_syntaxes_from_folder(
    builder: &mut syntect::parsing::SyntaxSetBuilder,
    folder: PathBuf,
) -> Result<(), LoadingError> {
    if !folder.is_dir() {
        return Ok(());
    }
    builder.add_from_folder(folder, true)
}

fn load_theme_set(config_dir: Option<&Path>) -> ThemeSet {
    let mut ts = ThemeSet::load_defaults();
    if let Some(config_dir) = config_dir {
        let _ = add_themes_from_folder(&mut ts, config_dir.join("themes"));
    }
    ts
}

fn add_themes_from_folder(ts: &mut ThemeSet, folder: PathBuf) -> Result<(), LoadingError> {
    if !folder.is_dir() {
        return Ok(());
    }
    ts.add_from_folder(folder)
}
