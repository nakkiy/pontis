pub fn difftool_command(left: &str, right: &str) -> String {
    format!("pontis git --diff {left} {right}")
}
