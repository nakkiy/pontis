pub fn build_screen_title(name: &str) -> String {
    format!("pontis preview: {name}")
}

pub fn render_summary(modified: usize, renamed: usize) -> String {
    let mut parts = Vec::new();
    parts.push(format!("modified={modified}"));
    parts.push(format!("renamed={renamed}"));
    parts.join(" ")
}

pub fn footer_hint() -> &'static str {
    "alt+left/right merges hunk"
}
