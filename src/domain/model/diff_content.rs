#[derive(Debug, Clone)]
pub struct DiffContent {
    pub left_text: String,
    pub right_text: String,
    pub left_bytes: usize,
    pub right_bytes: usize,
    pub left_is_binary: bool,
    pub right_is_binary: bool,
    pub is_binary: bool,
    pub has_unsupported_encoding: bool,
    pub left_has_unsupported_encoding: bool,
    pub right_has_unsupported_encoding: bool,
    pub left_has_utf8_bom: bool,
    pub right_has_utf8_bom: bool,
    pub highlight_limited: bool,
}
