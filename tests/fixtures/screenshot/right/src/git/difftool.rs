pub fn difftool_command(left: &str, right: &str) -> String {
    format!(
        "pontis git --repo \"$PWD\" --diff {left} {right} --difftool-left-dir \"$LOCAL\" --difftool-right-dir \"$REMOTE\""
    )
}
