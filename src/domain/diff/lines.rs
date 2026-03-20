pub fn split_lines_keep_newline(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split_inclusive('\n').map(ToOwned::to_owned).collect()
}

pub fn join_lines(lines: &[String]) -> String {
    lines.concat()
}
