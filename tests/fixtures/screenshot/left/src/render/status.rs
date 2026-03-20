pub fn status_header() -> &'static str {
    "Files"
}

pub fn format_loading_state(loaded: bool) -> &'static str {
    if loaded {
        "ready"
    } else {
        "loading..."
    }
}

pub fn format_dirty_marker(is_dirty: bool) -> &'static str {
    if is_dirty {
        "*"
    } else {
        ""
    }
}
