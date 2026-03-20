use syntect::highlighting::ThemeSet;

pub(super) fn pick_theme_name(ts: &ThemeSet) -> String {
    if ts.themes.contains_key("base16-ocean.dark") {
        "base16-ocean.dark".to_string()
    } else {
        ts.themes
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "InspiredGitHub".to_string())
    }
}
